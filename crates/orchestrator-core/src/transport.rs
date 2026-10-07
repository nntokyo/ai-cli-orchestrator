use std::{
    collections::VecDeque,
    error::Error,
    ffi::OsString,
    fmt,
    io::{self, Read, Write},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, Command, ExitStatus, Stdio},
    sync::{
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver, SyncSender, TrySendError},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub const DEFAULT_MAX_FRAME_BYTES: usize = 1024 * 1024;
pub const DEFAULT_EVENT_CAPACITY: usize = 128;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum TransportKind {
    NativeProtocol,
    PersistentStructured,
    HeadlessStructured,
    PtyFallback,
}

impl TransportKind {
    #[must_use]
    pub const fn preference(self) -> u8 {
        match self {
            Self::NativeProtocol => 0,
            Self::PersistentStructured => 1,
            Self::HeadlessStructured => 2,
            Self::PtyFallback => 3,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EnvironmentPolicy {
    Inherit {
        set: Vec<(OsString, OsString)>,
        remove: Vec<OsString>,
    },
    Clear {
        set: Vec<(OsString, OsString)>,
    },
}

impl EnvironmentPolicy {
    #[must_use]
    pub fn inherit() -> Self {
        Self::Inherit {
            set: Vec::new(),
            remove: Vec::new(),
        }
    }

    #[must_use]
    pub fn clear() -> Self {
        Self::Clear { set: Vec::new() }
    }

    #[must_use]
    pub fn with_variable(mut self, key: impl Into<OsString>, value: impl Into<OsString>) -> Self {
        match &mut self {
            Self::Inherit { set, .. } | Self::Clear { set } => {
                set.push((key.into(), value.into()));
            }
        }
        self
    }

    #[must_use]
    pub fn without_variable(mut self, key: impl Into<OsString>) -> Self {
        if let Self::Inherit { remove, .. } = &mut self {
            remove.push(key.into());
        }
        self
    }

    fn apply(&self, command: &mut Command) {
        match self {
            Self::Inherit { set, remove } => {
                for key in remove {
                    command.env_remove(key);
                }
                for (key, value) in set {
                    command.env(key, value);
                }
            }
            Self::Clear { set } => {
                command.env_clear();
                for (key, value) in set {
                    command.env(key, value);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CommandSpec {
    pub program: PathBuf,
    pub args: Vec<OsString>,
    pub cwd: PathBuf,
    pub environment: EnvironmentPolicy,
}

impl CommandSpec {
    #[must_use]
    pub fn new(
        program: impl Into<PathBuf>,
        cwd: impl Into<PathBuf>,
        environment: EnvironmentPolicy,
    ) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            cwd: cwd.into(),
            environment,
        }
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<OsString>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<OsString>,
    {
        self.args.extend(args.into_iter().map(Into::into));
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputStream {
    Stdout,
    Stderr,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportEvent {
    Frame {
        stream: OutputStream,
        bytes: Vec<u8>,
    },
    FrameTooLarge {
        stream: OutputStream,
        max_bytes: usize,
    },
    ReaderError {
        stream: OutputStream,
        message: String,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProcessExit {
    pub success: bool,
    pub code: Option<i32>,
}

impl From<ExitStatus> for ProcessExit {
    fn from(status: ExitStatus) -> Self {
        Self {
            success: status.success(),
            code: status.code(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownOutcome {
    Exited(ProcessExit),
    Terminated(ProcessExit),
}

#[derive(Debug)]
pub enum TransportError {
    EmptyProgram,
    InvalidWorkingDirectory(PathBuf),
    Spawn(io::Error),
    MissingPipe(&'static str),
    InputClosed,
    Write(io::Error),
    Wait(io::Error),
    Kill(io::Error),
    WorkerPanicked,
}

impl fmt::Display for TransportError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyProgram => formatter.write_str("transport program must not be empty"),
            Self::InvalidWorkingDirectory(path) => {
                write!(formatter, "transport working directory is invalid: {}", path.display())
            }
            Self::Spawn(error) => write!(formatter, "failed to spawn transport process: {error}"),
            Self::MissingPipe(name) => write!(formatter, "transport process is missing {name} pipe"),
            Self::InputClosed => formatter.write_str("transport stdin is already closed"),
            Self::Write(error) => write!(formatter, "failed to write transport input: {error}"),
            Self::Wait(error) => write!(formatter, "failed to inspect transport process: {error}"),
            Self::Kill(error) => write!(formatter, "failed to terminate transport process: {error}"),
            Self::WorkerPanicked => formatter.write_str("transport reader worker panicked"),
        }
    }
}

impl Error for TransportError {}

#[derive(Debug, Clone, PartialEq, Eq)]
enum DecodedFrame {
    Frame(Vec<u8>),
    TooLarge,
}

#[derive(Debug)]
struct LineFrameDecoder {
    max_frame_bytes: usize,
    buffer: Vec<u8>,
    dropping_oversized: bool,
}

impl LineFrameDecoder {
    fn new(max_frame_bytes: usize) -> Self {
        Self {
            max_frame_bytes: max_frame_bytes.max(1),
            buffer: Vec::new(),
            dropping_oversized: false,
        }
    }

    fn push(&mut self, chunk: &[u8]) -> Vec<DecodedFrame> {
        let mut frames = Vec::new();

        for &byte in chunk {
            if self.dropping_oversized {
                if byte == b'\n' {
                    self.dropping_oversized = false;
                }
                continue;
            }

            if byte == b'\n' {
                if self.buffer.last() == Some(&b'\r') {
                    self.buffer.pop();
                }
                frames.push(DecodedFrame::Frame(std::mem::take(&mut self.buffer)));
                continue;
            }

            if self.buffer.len() == self.max_frame_bytes {
                self.buffer.clear();
                self.dropping_oversized = true;
                frames.push(DecodedFrame::TooLarge);
                continue;
            }

            self.buffer.push(byte);
        }

        frames
    }

    fn finish(&mut self) -> Option<DecodedFrame> {
        if self.dropping_oversized || self.buffer.is_empty() {
            self.buffer.clear();
            return None;
        }

        if self.buffer.last() == Some(&b'\r') {
            self.buffer.pop();
        }

        Some(DecodedFrame::Frame(std::mem::take(&mut self.buffer)))
    }
}

pub trait Transport {
    fn kind(&self) -> TransportKind;
    fn process_id(&self) -> Option<u32>;
    fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError>;
    fn send_line(&mut self, bytes: &[u8]) -> Result<(), TransportError>;
    fn try_event(&mut self) -> Option<TransportEvent>;
    fn close_input(&mut self);
    fn try_wait(&mut self) -> Result<Option<ProcessExit>, TransportError>;
    fn shutdown(&mut self, timeout: Duration) -> Result<ShutdownOutcome, TransportError>;
}

pub struct StdioProcessTransport {
    kind: TransportKind,
    child: Child,
    stdin: Option<ChildStdin>,
    events: Receiver<TransportEvent>,
    buffered_events: VecDeque<TransportEvent>,
    stop_readers: Arc<AtomicBool>,
    readers: Vec<JoinHandle<()>>,
    shutdown_complete: bool,
}

impl StdioProcessTransport {
    pub fn spawn(
        kind: TransportKind,
        spec: &CommandSpec,
        max_frame_bytes: usize,
        event_capacity: usize,
    ) -> Result<Self, TransportError> {
        if spec.program.as_os_str().is_empty() {
            return Err(TransportError::EmptyProgram);
        }
        if !spec.cwd.is_dir() {
            return Err(TransportError::InvalidWorkingDirectory(spec.cwd.clone()));
        }

        let mut command = Command::new(&spec.program);
        command
            .args(&spec.args)
            .current_dir(&spec.cwd)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped());
        spec.environment.apply(&mut command);

        let mut child = command.spawn().map_err(TransportError::Spawn)?;
        let stdin = child.stdin.take().ok_or(TransportError::MissingPipe("stdin"))?;
        let stdout = child
            .stdout
            .take()
            .ok_or(TransportError::MissingPipe("stdout"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or(TransportError::MissingPipe("stderr"))?;

        let (sender, receiver) = mpsc::sync_channel(event_capacity.max(1));
        let stop_readers = Arc::new(AtomicBool::new(false));

        let readers = vec![
            spawn_reader(
                stdout,
                OutputStream::Stdout,
                sender.clone(),
                Arc::clone(&stop_readers),
                max_frame_bytes,
            ),
            spawn_reader(
                stderr,
                OutputStream::Stderr,
                sender,
                Arc::clone(&stop_readers),
                max_frame_bytes,
            ),
        ];

        Ok(Self {
            kind,
            child,
            stdin: Some(stdin),
            events: receiver,
            buffered_events: VecDeque::new(),
            stop_readers,
            readers,
            shutdown_complete: false,
        })
    }

    fn stop_and_join_readers(&mut self) -> Result<(), TransportError> {
        self.stop_readers.store(true, Ordering::Release);

        for reader in self.readers.drain(..) {
            if reader.join().is_err() {
                return Err(TransportError::WorkerPanicked);
            }
        }

        Ok(())
    }

    fn wait_until(&mut self, deadline: Instant) -> Result<Option<ProcessExit>, TransportError> {
        loop {
            if let Some(exit) = self.try_wait()? {
                return Ok(Some(exit));
            }
            if Instant::now() >= deadline {
                return Ok(None);
            }

            while let Ok(event) = self.events.try_recv() {
                self.buffered_events.push_back(event);
            }

            thread::sleep(Duration::from_millis(10));
        }
    }
}

impl Transport for StdioProcessTransport {
    fn kind(&self) -> TransportKind {
        self.kind
    }

    fn process_id(&self) -> Option<u32> {
        if self.shutdown_complete {
            None
        } else {
            Some(self.child.id())
        }
    }

    fn send(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        let stdin = self.stdin.as_mut().ok_or(TransportError::InputClosed)?;
        stdin.write_all(bytes).map_err(TransportError::Write)?;
        stdin.flush().map_err(TransportError::Write)
    }

    fn send_line(&mut self, bytes: &[u8]) -> Result<(), TransportError> {
        self.send(bytes)?;
        self.send(b"\n")
    }

    fn try_event(&mut self) -> Option<TransportEvent> {
        self.buffered_events
            .pop_front()
            .or_else(|| self.events.try_recv().ok())
    }

    fn close_input(&mut self) {
        self.stdin.take();
    }

    fn try_wait(&mut self) -> Result<Option<ProcessExit>, TransportError> {
        self.child
            .try_wait()
            .map(|status| status.map(ProcessExit::from))
            .map_err(TransportError::Wait)
    }

    fn shutdown(&mut self, timeout: Duration) -> Result<ShutdownOutcome, TransportError> {
        self.close_input();

        if let Some(exit) = self.wait_until(Instant::now() + timeout)? {
            self.shutdown_complete = true;
            self.stop_and_join_readers()?;
            return Ok(ShutdownOutcome::Exited(exit));
        }

        self.stop_readers.store(true, Ordering::Release);
        self.child.kill().map_err(TransportError::Kill)?;
        let status = self.child.wait().map_err(TransportError::Wait)?;
        self.shutdown_complete = true;
        self.stop_and_join_readers()?;

        Ok(ShutdownOutcome::Terminated(status.into()))
    }
}

impl Drop for StdioProcessTransport {
    fn drop(&mut self) {
        if !self.shutdown_complete {
            self.close_input();
            self.stop_readers.store(true, Ordering::Release);
            let _ = self.child.kill();
            let _ = self.child.wait();
            let _ = self.stop_and_join_readers();
            self.shutdown_complete = true;
        }
    }
}

fn spawn_reader<R>(
    mut reader: R,
    stream: OutputStream,
    sender: SyncSender<TransportEvent>,
    stop: Arc<AtomicBool>,
    max_frame_bytes: usize,
) -> JoinHandle<()>
where
    R: Read + Send + 'static,
{
    thread::spawn(move || {
        let mut decoder = LineFrameDecoder::new(max_frame_bytes);
        let mut chunk = [0_u8; 8192];

        loop {
            if stop.load(Ordering::Acquire) {
                return;
            }

            match reader.read(&mut chunk) {
                Ok(0) => {
                    if let Some(frame) = decoder.finish() {
                        send_decoded(stream, frame, &sender, &stop);
                    }
                    return;
                }
                Ok(length) => {
                    for frame in decoder.push(&chunk[..length]) {
                        if !send_decoded(stream, frame, &sender, &stop) {
                            return;
                        }
                    }
                }
                Err(error) if error.kind() == io::ErrorKind::Interrupted => {}
                Err(error) => {
                    let _ = send_with_backpressure(
                        &sender,
                        TransportEvent::ReaderError {
                            stream,
                            message: error.to_string(),
                        },
                        &stop,
                    );
                    return;
                }
            }
        }
    })
}

fn send_decoded(
    stream: OutputStream,
    frame: DecodedFrame,
    sender: &SyncSender<TransportEvent>,
    stop: &AtomicBool,
) -> bool {
    let event = match frame {
        DecodedFrame::Frame(bytes) => TransportEvent::Frame { stream, bytes },
        DecodedFrame::TooLarge => TransportEvent::FrameTooLarge {
            stream,
            max_bytes: DEFAULT_MAX_FRAME_BYTES,
        },
    };

    send_with_backpressure(sender, event, stop)
}

fn send_with_backpressure(
    sender: &SyncSender<TransportEvent>,
    mut event: TransportEvent,
    stop: &AtomicBool,
) -> bool {
    loop {
        if stop.load(Ordering::Acquire) {
            return false;
        }

        match sender.try_send(event) {
            Ok(()) => return true,
            Err(TrySendError::Full(returned)) => {
                event = returned;
                thread::sleep(Duration::from_millis(2));
            }
            Err(TrySendError::Disconnected(_)) => return false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        env,
        io::{BufRead, BufReader},
        process,
    };

    #[test]
    fn transport_priority_prefers_machine_readable_protocols() {
        assert!(
            TransportKind::NativeProtocol.preference()
                < TransportKind::PersistentStructured.preference()
        );
        assert!(
            TransportKind::PersistentStructured.preference()
                < TransportKind::HeadlessStructured.preference()
        );
        assert!(
            TransportKind::HeadlessStructured.preference() < TransportKind::PtyFallback.preference()
        );
    }

    #[test]
    fn decoder_reassembles_partial_frames_and_strips_crlf() {
        let mut decoder = LineFrameDecoder::new(32);

        assert!(decoder.push(b"hel").is_empty());
        assert_eq!(
            decoder.push(b"lo\r\nworld\n"),
            vec![
                DecodedFrame::Frame(b"hello".to_vec()),
                DecodedFrame::Frame(b"world".to_vec()),
            ]
        );
    }

    #[test]
    fn decoder_reports_and_skips_oversized_frame() {
        let mut decoder = LineFrameDecoder::new(4);

        assert_eq!(
            decoder.push(b"12345ignored\nok\n"),
            vec![
                DecodedFrame::TooLarge,
                DecodedFrame::Frame(b"ok".to_vec()),
            ]
        );
    }

    #[test]
    fn decoder_flushes_final_frame_without_newline() {
        let mut decoder = LineFrameDecoder::new(32);
        assert!(decoder.push(b"final").is_empty());
        assert_eq!(
            decoder.finish(),
            Some(DecodedFrame::Frame(b"final".to_vec()))
        );
    }

    #[test]
    fn command_spec_keeps_arguments_separate() {
        let spec = CommandSpec::new(
            "program",
            env::current_dir().expect("current directory"),
            EnvironmentPolicy::clear(),
        )
        .arg("hello world")
        .arg("$(not-a-shell)");

        assert_eq!(spec.args.len(), 2);
        assert_eq!(spec.args[0], OsString::from("hello world"));
        assert_eq!(spec.args[1], OsString::from("$(not-a-shell)"));
    }

    #[test]
    fn stdio_transport_supports_persistent_multi_turn_and_stderr_separation() {
        let mut transport = spawn_helper("echo", 128);

        transport.send_line(b"one").expect("first turn should send");
        transport.send_line(b"two").expect("second turn should send");
        transport.send_line(b"quit").expect("quit should send");

        let deadline = Instant::now() + Duration::from_secs(10);
        let mut stdout = Vec::new();
        let mut stderr = Vec::new();

        while Instant::now() < deadline && stdout.len() < 2 {
            if let Some(TransportEvent::Frame { stream, bytes }) = transport.try_event() {
                if bytes.starts_with(b"out:") {
                    stdout.push(bytes);
                } else if bytes.starts_with(b"err:") && stream == OutputStream::Stderr {
                    stderr.push(bytes);
                }
            } else {
                thread::sleep(Duration::from_millis(10));
            }
        }

        assert_eq!(stdout, vec![b"out:one".to_vec(), b"out:two".to_vec()]);
        assert!(stderr.iter().any(|frame| frame == b"err:one"));
        assert!(stderr.iter().any(|frame| frame == b"err:two"));

        let outcome = transport
            .shutdown(Duration::from_secs(5))
            .expect("helper should shutdown");
        assert!(matches!(outcome, ShutdownOutcome::Exited(_)));
    }

    #[test]
    fn shutdown_escalates_to_termination_after_timeout() {
        let mut transport = spawn_helper("sleep", 16);

        let outcome = transport
            .shutdown(Duration::from_millis(50))
            .expect("shutdown should force termination");

        assert!(matches!(outcome, ShutdownOutcome::Terminated(_)));
        assert_eq!(transport.process_id(), None);
    }

    #[test]
    fn invalid_working_directory_is_rejected_before_spawn() {
        let spec = CommandSpec::new(
            env::current_exe().expect("test executable"),
            Path::new("definitely-not-a-real-directory"),
            EnvironmentPolicy::clear(),
        );

        let error = StdioProcessTransport::spawn(
            TransportKind::HeadlessStructured,
            &spec,
            DEFAULT_MAX_FRAME_BYTES,
            DEFAULT_EVENT_CAPACITY,
        )
        .expect_err("invalid cwd must fail");

        assert!(matches!(error, TransportError::InvalidWorkingDirectory(_)));
    }

    fn spawn_helper(mode: &str, capacity: usize) -> StdioProcessTransport {
        let spec = CommandSpec::new(
            env::current_exe().expect("test executable"),
            env::current_dir().expect("current directory"),
            EnvironmentPolicy::inherit()
                .with_variable("AI_CLI_ORCHESTRATOR_TRANSPORT_HELPER", mode),
        )
        .args([
            "--exact",
            "transport::tests::transport_helper_process",
            "--ignored",
            "--nocapture",
            "--test-threads=1",
        ]);

        StdioProcessTransport::spawn(
            TransportKind::PersistentStructured,
            &spec,
            DEFAULT_MAX_FRAME_BYTES,
            capacity,
        )
        .expect("helper transport should spawn")
    }

    #[test]
    #[ignore = "invoked as a subprocess by transport tests"]
    fn transport_helper_process() {
        let mode = env::var("AI_CLI_ORCHESTRATOR_TRANSPORT_HELPER").unwrap_or_default();

        match mode.as_str() {
            "echo" => {
                let stdin = io::stdin();
                for line in BufReader::new(stdin.lock()).lines() {
                    let line = line.expect("helper input");
                    if line == "quit" {
                        break;
                    }
                    println!("out:{line}");
                    eprintln!("err:{line}");
                    io::stdout().flush().expect("stdout flush");
                    io::stderr().flush().expect("stderr flush");
                }
            }
            "sleep" => thread::sleep(Duration::from_secs(30)),
            other => {
                eprintln!("unknown helper mode: {other}");
                process::exit(2);
            }
        }
    }
}
