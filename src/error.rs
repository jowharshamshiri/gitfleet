use thiserror::Error;

#[derive(Error, Debug)]
pub enum SuperGitError {
    #[error("Git error: {0}")]
    Git(#[from] git2::Error),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Not a git repository")]
    NotARepository,

    #[error("No submodules found in .gitmodules")]
    NoSubmodules,

    #[error("Submodule '{0}' not found")]
    SubmoduleNotFound(String),

    #[error("Branch '{0}' not found in submodule '{1}'")]
    BranchNotFound(String, String),

    #[error("Submodule '{0}' has uncommitted changes")]
    UncommittedChanges(String),

    #[error("Submodule '{0}' is in detached HEAD state")]
    DetachedHead(String),

    #[error("Failed to parse .gitmodules: {0}")]
    GitModulesParse(String),

    #[error("Operation cancelled by user")]
    Cancelled,

    #[error("Merge conflict in submodule '{0}'")]
    MergeConflict(String),

    #[error("{0}")]
    Other(String),
}

pub type Result<T> = std::result::Result<T, SuperGitError>;
