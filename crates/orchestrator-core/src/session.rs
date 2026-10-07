use crate::identity::{
    NativeSessionId, ProviderId, SessionId, TaskId, WorkspaceId, WorkspaceIdentity,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceFingerprint(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GitSnapshot {
    pub branch: Option<String>,
    pub head: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SessionStatus {
    Active,
    Resumable,
    Stale,
    Missing,
    Closed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ResumeCapability {
    Unsupported,
    ById,
    ContinueMostRecent,
    ByIdAndFork,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderCompatibility {
    Compatible,
    RequiresRevalidation,
    Incompatible,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProviderSessionMetadata {
    pub session_id: SessionId,
    pub native_session_id: NativeSessionId,
    pub workspace_id: WorkspaceId,
    pub task_id: TaskId,
    pub provider_id: ProviderId,
    pub canonical_cwd: PathBuf,
    pub provider_version: Option<String>,
    pub capability_snapshot: Option<String>,
    pub model: Option<String>,
    pub effort: Option<String>,
    pub workspace_fingerprint: Option<WorkspaceFingerprint>,
    pub git_snapshot: Option<GitSnapshot>,
    pub permission_revision: Option<String>,
    pub resume_capability: ResumeCapability,
    pub status: SessionStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResumeContext {
    pub workspace_id: WorkspaceId,
    pub task_id: TaskId,
    pub provider_id: ProviderId,
    pub workspace: WorkspaceIdentity,
    pub workspace_fingerprint: Option<WorkspaceFingerprint>,
    pub git_snapshot: Option<GitSnapshot>,
    pub permission_revision: Option<String>,
    pub provider_compatibility: ProviderCompatibility,
    pub native_session_exists: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityMismatch {
    Workspace,
    Task,
    Provider,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StartNewReason {
    WorkspaceLocationChanged,
    NativeSessionMissing,
    ResumeUnsupported,
    SessionMissing,
    SessionClosed,
    ProviderIncompatible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResumeDrift {
    SessionMarkedStale,
    WorkspaceFingerprintChanged,
    GitContextChanged,
    PermissionPolicyChanged,
    ProviderRequiresRevalidation,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ResumeDecision {
    Resume,
    ResumeWithDrift { reasons: Vec<ResumeDrift> },
    StartNew { reason: StartNewReason },
    Reject { reason: IdentityMismatch },
}

/// Validates whether a native provider session can be resumed for the current
/// Task and Workspace.
///
/// Identity mismatches are rejected rather than converted into a silent
/// "start new" decision. This prevents a session belonging to another Task,
/// Workspace, or Provider from being consumed accidentally.
#[must_use]
pub fn validate_resume(
    session: &ProviderSessionMetadata,
    current: &ResumeContext,
) -> ResumeDecision {
    if session.workspace_id != current.workspace_id {
        return ResumeDecision::Reject {
            reason: IdentityMismatch::Workspace,
        };
    }

    if session.task_id != current.task_id {
        return ResumeDecision::Reject {
            reason: IdentityMismatch::Task,
        };
    }

    if session.provider_id != current.provider_id {
        return ResumeDecision::Reject {
            reason: IdentityMismatch::Provider,
        };
    }

    if !current.workspace.matches_canonical_path(&session.canonical_cwd) {
        return ResumeDecision::StartNew {
            reason: StartNewReason::WorkspaceLocationChanged,
        };
    }

    match session.status {
        SessionStatus::Missing => {
            return ResumeDecision::StartNew {
                reason: StartNewReason::SessionMissing,
            };
        }
        SessionStatus::Closed => {
            return ResumeDecision::StartNew {
                reason: StartNewReason::SessionClosed,
            };
        }
        SessionStatus::Active | SessionStatus::Resumable | SessionStatus::Stale => {}
    }

    if !current.native_session_exists {
        return ResumeDecision::StartNew {
            reason: StartNewReason::NativeSessionMissing,
        };
    }

    if session.resume_capability == ResumeCapability::Unsupported {
        return ResumeDecision::StartNew {
            reason: StartNewReason::ResumeUnsupported,
        };
    }

    if current.provider_compatibility == ProviderCompatibility::Incompatible {
        return ResumeDecision::StartNew {
            reason: StartNewReason::ProviderIncompatible,
        };
    }

    let mut reasons = Vec::new();

    if session.status == SessionStatus::Stale {
        reasons.push(ResumeDrift::SessionMarkedStale);
    }

    if session.workspace_fingerprint != current.workspace_fingerprint {
        reasons.push(ResumeDrift::WorkspaceFingerprintChanged);
    }

    if session.git_snapshot != current.git_snapshot {
        reasons.push(ResumeDrift::GitContextChanged);
    }

    if session.permission_revision != current.permission_revision {
        reasons.push(ResumeDrift::PermissionPolicyChanged);
    }

    if current.provider_compatibility == ProviderCompatibility::RequiresRevalidation {
        reasons.push(ResumeDrift::ProviderRequiresRevalidation);
    }

    if reasons.is_empty() {
        ResumeDecision::Resume
    } else {
        ResumeDecision::ResumeWithDrift { reasons }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::identity::WorkspaceIdentity;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_workspace(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after unix epoch")
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "ai-cli-orchestrator-session-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary workspace should be created");
        path
    }

    fn workspace_id(value: &str) -> WorkspaceId {
        WorkspaceId::new(value).expect("valid workspace id")
    }

    fn task_id(value: &str) -> TaskId {
        TaskId::new(value).expect("valid task id")
    }

    fn provider_id(value: &str) -> ProviderId {
        ProviderId::new(value).expect("valid provider id")
    }

    fn session_id(value: &str) -> SessionId {
        SessionId::new(value).expect("valid session id")
    }

    fn native_session_id(value: &str) -> NativeSessionId {
        NativeSessionId::new(value).expect("valid native session id")
    }

    struct Fixture {
        root: PathBuf,
        session: ProviderSessionMetadata,
        current: ResumeContext,
    }

    impl Fixture {
        fn new() -> Self {
            let root = temporary_workspace("fixture");
            let workspace =
                WorkspaceIdentity::resolve(&root).expect("temporary workspace should resolve");

            let session = ProviderSessionMetadata {
                session_id: session_id("session-1"),
                native_session_id: native_session_id("native-1"),
                workspace_id: workspace_id("workspace-1"),
                task_id: task_id("task-1"),
                provider_id: provider_id("provider-1"),
                canonical_cwd: workspace.canonical_path().to_path_buf(),
                provider_version: Some("1.0.0".into()),
                capability_snapshot: Some("resume-by-id".into()),
                model: Some("model-a".into()),
                effort: Some("medium".into()),
                workspace_fingerprint: Some(WorkspaceFingerprint("fingerprint-a".into())),
                git_snapshot: None,
                permission_revision: Some("permission-v1".into()),
                resume_capability: ResumeCapability::ById,
                status: SessionStatus::Resumable,
            };

            let current = ResumeContext {
                workspace_id: workspace_id("workspace-1"),
                task_id: task_id("task-1"),
                provider_id: provider_id("provider-1"),
                workspace,
                workspace_fingerprint: Some(WorkspaceFingerprint("fingerprint-a".into())),
                git_snapshot: None,
                permission_revision: Some("permission-v1".into()),
                provider_compatibility: ProviderCompatibility::Compatible,
                native_session_exists: true,
            };

            Self {
                root,
                session,
                current,
            }
        }
    }

    impl Drop for Fixture {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.root);
        }
    }

    #[test]
    fn resumes_when_identity_and_state_match() {
        let fixture = Fixture::new();

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::Resume
        );
    }

    #[test]
    fn rejects_cross_task_session_even_for_continue_most_recent() {
        let mut fixture = Fixture::new();
        fixture.session.resume_capability = ResumeCapability::ContinueMostRecent;
        fixture.current.task_id = task_id("task-2");

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::Reject {
                reason: IdentityMismatch::Task
            }
        );
    }

    #[test]
    fn rejects_wrong_workspace() {
        let mut fixture = Fixture::new();
        fixture.current.workspace_id = workspace_id("workspace-2");

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::Reject {
                reason: IdentityMismatch::Workspace
            }
        );
    }

    #[test]
    fn rejects_wrong_provider() {
        let mut fixture = Fixture::new();
        fixture.current.provider_id = provider_id("provider-2");

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::Reject {
                reason: IdentityMismatch::Provider
            }
        );
    }

    #[test]
    fn missing_stored_session_starts_new() {
        let mut fixture = Fixture::new();
        fixture.session.status = SessionStatus::Missing;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::StartNew {
                reason: StartNewReason::SessionMissing
            }
        );
    }

    #[test]
    fn closed_stored_session_starts_new() {
        let mut fixture = Fixture::new();
        fixture.session.status = SessionStatus::Closed;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::StartNew {
                reason: StartNewReason::SessionClosed
            }
        );
    }

    #[test]
    fn starts_new_when_native_session_was_deleted() {
        let mut fixture = Fixture::new();
        fixture.current.native_session_exists = false;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::StartNew {
                reason: StartNewReason::NativeSessionMissing
            }
        );
    }

    #[test]
    fn starts_new_when_resume_is_unsupported() {
        let mut fixture = Fixture::new();
        fixture.session.resume_capability = ResumeCapability::Unsupported;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::StartNew {
                reason: StartNewReason::ResumeUnsupported
            }
        );
    }

    #[test]
    fn incompatible_provider_version_starts_new() {
        let mut fixture = Fixture::new();
        fixture.current.provider_compatibility = ProviderCompatibility::Incompatible;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::StartNew {
                reason: StartNewReason::ProviderIncompatible
            }
        );
    }

    #[test]
    fn provider_revalidation_is_reported_as_drift() {
        let mut fixture = Fixture::new();
        fixture.current.provider_compatibility = ProviderCompatibility::RequiresRevalidation;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::ResumeWithDrift {
                reasons: vec![ResumeDrift::ProviderRequiresRevalidation]
            }
        );
    }

    #[test]
    fn fingerprint_change_is_reported_as_drift() {
        let mut fixture = Fixture::new();
        fixture.current.workspace_fingerprint =
            Some(WorkspaceFingerprint("fingerprint-b".into()));

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::ResumeWithDrift {
                reasons: vec![ResumeDrift::WorkspaceFingerprintChanged]
            }
        );
    }

    #[test]
    fn git_change_is_reported_as_drift_but_git_is_optional() {
        let mut fixture = Fixture::new();
        fixture.current.git_snapshot = Some(GitSnapshot {
            branch: Some("feature".into()),
            head: Some("abc123".into()),
        });

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::ResumeWithDrift {
                reasons: vec![ResumeDrift::GitContextChanged]
            }
        );
    }

    #[test]
    fn permission_revision_change_is_reported_as_drift() {
        let mut fixture = Fixture::new();
        fixture.current.permission_revision = Some("permission-v2".into());

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::ResumeWithDrift {
                reasons: vec![ResumeDrift::PermissionPolicyChanged]
            }
        );
    }

    #[test]
    fn moved_workspace_starts_new_instead_of_silent_resume() {
        let mut fixture = Fixture::new();
        let moved = fixture.root.with_extension("moved");

        fs::rename(&fixture.root, &moved).expect("workspace should be moved");
        fixture.current.workspace =
            WorkspaceIdentity::resolve(&moved).expect("moved workspace should resolve");
        fixture.root = moved;

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::StartNew {
                reason: StartNewReason::WorkspaceLocationChanged
            }
        );
    }

    #[test]
    fn stale_session_accumulates_multiple_drift_reasons() {
        let mut fixture = Fixture::new();
        fixture.session.status = SessionStatus::Stale;
        fixture.current.workspace_fingerprint =
            Some(WorkspaceFingerprint("fingerprint-b".into()));
        fixture.current.permission_revision = Some("permission-v2".into());

        assert_eq!(
            validate_resume(&fixture.session, &fixture.current),
            ResumeDecision::ResumeWithDrift {
                reasons: vec![
                    ResumeDrift::SessionMarkedStale,
                    ResumeDrift::WorkspaceFingerprintChanged,
                    ResumeDrift::PermissionPolicyChanged,
                ]
            }
        );
    }
}
