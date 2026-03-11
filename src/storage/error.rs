use std::fmt;
use std::io;

/// All possible errors from the storage layer.
/// Every variant carries the file path that caused the error
/// so callers always have context for diagnostics.
#[derive(Debug)]
pub enum StorageError {
    /// Cannot create the data directory.
    DirectoryCreateFail {
        path: String,
        source: io::Error,
    },
    /// Expected file does not exist.
    FileNotFound {
        path: String,
    },
    /// Cannot read file (permissions, I/O).
    FileReadError {
        path: String,
        source: io::Error,
    },
    /// Cannot write file (disk full, permissions).
    FileWriteError {
        path: String,
        source: io::Error,
    },
    /// JSON is malformed.
    ParseError {
        path: String,
        message: String,
    },
    /// Cannot serialize data to JSON.
    SerializationError {
        message: String,
    },
    /// Backup file is also invalid during recovery.
    BackupRestoreError {
        path: String,
        message: String,
    },
}

impl fmt::Display for StorageError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StorageError::DirectoryCreateFail { path, source } => {
                write!(f, "Failed to create directory '{}': {}", path, source)
            }
            StorageError::FileNotFound { path } => {
                write!(f, "File not found: '{}'", path)
            }
            StorageError::FileReadError { path, source } => {
                write!(f, "Failed to read file '{}': {}", path, source)
            }
            StorageError::FileWriteError { path, source } => {
                write!(f, "Failed to write file '{}': {}", path, source)
            }
            StorageError::ParseError { path, message } => {
                write!(f, "Failed to parse '{}': {}", path, message)
            }
            StorageError::SerializationError { message } => {
                write!(f, "Serialization error: {}", message)
            }
            StorageError::BackupRestoreError { path, message } => {
                write!(f, "Backup restore failed for '{}': {}", path, message)
            }
        }
    }
}

impl std::error::Error for StorageError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            StorageError::DirectoryCreateFail { source, .. } => Some(source),
            StorageError::FileReadError { source, .. } => Some(source),
            StorageError::FileWriteError { source, .. } => Some(source),
            _ => None,
        }
    }
}
