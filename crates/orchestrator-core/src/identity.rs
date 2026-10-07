use serde::{Deserialize, Serialize};
use std::{
    fs,
    io,
    path::{Path, PathBuf},
};

macro_rules! string_id {
    ($name:ident) => {
        #[derive(
            Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize,
        )]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            #[must_use]
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }
    };
}

string_id!(WorkspaceId);
string_id!(TaskId);
string_id!(ProviderId);
string_id!(SessionId);
string_id!(NativeSessionId);

/// Filesystem identity used to prevent a provider session from being resumed
/// in a different workspace by accident.
///
/// The identity is intentionally path-based. A moved or renamed workspace is
/// treated as a different location until a later persistence/UI flow performs
/// an explicit relink. This is safer than silently reusing a session.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WorkspaceIdentity {
    canonical_path: PathBuf,
    comparison_key: String,
}

impl WorkspaceIdentity {
    /// Resolves symlinks and filesystem aliases for an existing workspace.
    pub fn resolve(path: impl AsRef<Path>) -> io::Result<Self> {
        let canonical_path = fs::canonicalize(path)?;
        let comparison_key = comparison_key(&canonical_path);

        Ok(Self {
            canonical_path,
            comparison_key,
        })
    }

    #[must_use]
    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    #[must_use]
    pub fn same_location(&self, other: &Self) -> bool {
        self.comparison_key == other.comparison_key
    }

    #[must_use]
    pub fn matches_canonical_path(&self, path: &Path) -> bool {
        self.comparison_key == comparison_key(path)
    }
}

#[cfg(windows)]
fn comparison_key(path: &Path) -> String {
    let mut value = path.to_string_lossy().replace('/', "\");

    if let Some(rest) = value.strip_prefix(r"\\?\UNC\") {
        value = format!(r"\\{rest}");
    } else if let Some(rest) = value.strip_prefix(r"\\?\") {
        value = rest.to_owned();
    }

    value.to_lowercase()
}

#[cfg(not(windows))]
fn comparison_key(path: &Path) -> String {
    path.to_string_lossy().into_owned()
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        fs,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temporary_workspace(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("clock must be after unix epoch")
            .as_nanos();

        let path = std::env::temp_dir().join(format!(
            "ai-cli-orchestrator-{name}-{}-{nonce}",
            std::process::id()
        ));
        fs::create_dir_all(&path).expect("temporary workspace should be created");
        path
    }

    #[test]
    fn resolves_spaces_and_unicode() {
        let root = temporary_workspace("identity");
        let workspace = root.join("workspace with spaces 日本語");
        fs::create_dir_all(&workspace).expect("workspace should be created");

        let identity = WorkspaceIdentity::resolve(&workspace).expect("workspace should resolve");

        assert!(identity.canonical_path().is_absolute());

        fs::remove_dir_all(root).expect("temporary workspace should be removed");
    }

    #[test]
    fn rename_is_treated_as_a_new_location() {
        let root = temporary_workspace("rename");
        let before = root.join("before");
        let after = root.join("after");
        fs::create_dir_all(&before).expect("workspace should be created");

        let before_identity = WorkspaceIdentity::resolve(&before).expect("workspace should resolve");
        fs::rename(&before, &after).expect("workspace should be renamed");
        let after_identity = WorkspaceIdentity::resolve(&after).expect("renamed workspace should resolve");

        assert!(!before_identity.same_location(&after_identity));

        fs::remove_dir_all(root).expect("temporary workspace should be removed");
    }

    #[cfg(unix)]
    #[test]
    fn symlink_and_target_share_identity() {
        use std::os::unix::fs::symlink;

        let root = temporary_workspace("symlink");
        let target = root.join("target");
        let alias = root.join("alias");
        fs::create_dir_all(&target).expect("target should be created");
        symlink(&target, &alias).expect("symlink should be created");

        let target_identity = WorkspaceIdentity::resolve(&target).expect("target should resolve");
        let alias_identity = WorkspaceIdentity::resolve(&alias).expect("symlink should resolve");

        assert!(target_identity.same_location(&alias_identity));

        fs::remove_dir_all(root).expect("temporary workspace should be removed");
    }

    #[cfg(windows)]
    #[test]
    fn windows_comparison_key_is_case_insensitive() {
        let upper = comparison_key(Path::new(r"C:\Users\Example\Workspace"));
        let lower = comparison_key(Path::new(r"c:\users\example\workspace"));

        assert_eq!(upper, lower);
    }

    #[cfg(windows)]
    #[test]
    fn windows_comparison_key_strips_extended_prefix() {
        let extended = comparison_key(Path::new(r"\\?\C:\Users\Example\Workspace"));
        let normal = comparison_key(Path::new(r"C:\Users\Example\Workspace"));

        assert_eq!(extended, normal);
    }
}
