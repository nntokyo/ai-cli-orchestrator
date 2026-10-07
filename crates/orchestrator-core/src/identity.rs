use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use std::{
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IdentityValueError {
    Empty,
}

impl fmt::Display for IdentityValueError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => formatter.write_str("identity value must not be empty or whitespace"),
        }
    }
}

impl Error for IdentityValueError {}

macro_rules! string_id {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize)]
        #[serde(transparent)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, IdentityValueError> {
                let value = value.into();
                if value.trim().is_empty() {
                    return Err(IdentityValueError::Empty);
                }

                Ok(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl TryFrom<String> for $name {
            type Error = IdentityValueError;

            fn try_from(value: String) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl TryFrom<&str> for $name {
            type Error = IdentityValueError;

            fn try_from(value: &str) -> Result<Self, Self::Error> {
                Self::new(value)
            }
        }

        impl<'de> Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                let value = String::deserialize(deserializer)?;
                Self::new(value).map_err(D::Error::custom)
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
///
/// The current identity must be resolved from the filesystem rather than
/// deserialized from persisted comparison data. Persisted sessions store their
/// canonical working directory separately.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceIdentity {
    canonical_path: PathBuf,
}

impl WorkspaceIdentity {
    /// Resolves symlinks and filesystem aliases for an existing workspace.
    pub fn resolve(path: impl AsRef<Path>) -> io::Result<Self> {
        let canonical_path = fs::canonicalize(path)?;
        Ok(Self { canonical_path })
    }

    #[must_use]
    pub fn canonical_path(&self) -> &Path {
        &self.canonical_path
    }

    #[must_use]
    pub fn same_location(&self, other: &Self) -> bool {
        self.canonical_path == other.canonical_path
    }

    /// Compares against a path that was already canonicalized when the session
    /// metadata was captured.
    ///
    /// This deliberately does not implement its own Unicode case folding. If
    /// platform canonicalization cannot prove the paths are the same, the
    /// safer outcome is a false negative that starts a new session.
    #[must_use]
    pub fn matches_canonical_path(&self, path: &Path) -> bool {
        self.canonical_path == path
    }
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
    fn ids_reject_empty_and_whitespace_values() {
        assert_eq!(WorkspaceId::new(""), Err(IdentityValueError::Empty));
        assert_eq!(TaskId::new("   "), Err(IdentityValueError::Empty));
        assert_eq!(ProviderId::new("\t"), Err(IdentityValueError::Empty));
        assert_eq!(SessionId::new("\n"), Err(IdentityValueError::Empty));
        assert_eq!(NativeSessionId::new("  "), Err(IdentityValueError::Empty));
    }

    #[test]
    fn deserialization_rejects_empty_identity_values() {
        use serde::de::value::{Error as ValueError, StringDeserializer};

        let deserializer = StringDeserializer::<ValueError>::new("   ".to_owned());

        assert!(WorkspaceId::deserialize(deserializer).is_err());
    }

    #[test]
    fn ids_preserve_non_empty_values() {
        let id = TaskId::new("task-1").expect("valid task id");

        assert_eq!(id.as_str(), "task-1");
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

        let before_identity =
            WorkspaceIdentity::resolve(&before).expect("workspace should resolve");
        fs::rename(&before, &after).expect("workspace should be renamed");
        let after_identity =
            WorkspaceIdentity::resolve(&after).expect("renamed workspace should resolve");

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
    fn windows_canonicalization_handles_case_aliases() {
        let root = temporary_workspace("case");
        let mixed = root.join("MixedCaseWorkspace");
        fs::create_dir_all(&mixed).expect("workspace should be created");

        let alternate = root.join("mixedcaseworkspace");
        let mixed_identity = WorkspaceIdentity::resolve(&mixed).expect("workspace should resolve");
        let alternate_identity =
            WorkspaceIdentity::resolve(&alternate).expect("case alias should resolve");

        assert!(mixed_identity.same_location(&alternate_identity));

        fs::remove_dir_all(root).expect("temporary workspace should be removed");
    }
}
