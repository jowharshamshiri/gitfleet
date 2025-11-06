use crate::config::Submodule;
use crate::error::{Result, SuperGitError};
use crate::git::operations::GitOps;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone)]
pub struct SubmoduleStatus {
    pub name: String,
    pub path: String,
    pub current_branch: Option<String>,
    pub is_clean: bool,
    pub is_detached: bool,
    pub ahead: usize,
    pub behind: usize,
    pub uncommitted_files: Vec<String>,
    pub exists: bool,
}

pub struct SubmoduleOps;

impl SubmoduleOps {
    /// Get status for a single submodule
    pub fn get_status<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
    ) -> Result<SubmoduleStatus> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        // Check if submodule exists
        if !submodule_path.exists() {
            return Ok(SubmoduleStatus {
                name: submodule.name.clone(),
                path: submodule.path.display().to_string(),
                current_branch: None,
                is_clean: true,
                is_detached: false,
                ahead: 0,
                behind: 0,
                uncommitted_files: Vec::new(),
                exists: false,
            });
        }

        let repo = GitOps::open_repo(&submodule_path)?;

        let is_detached = GitOps::is_detached_head(&repo)?;
        let current_branch = if is_detached {
            None
        } else {
            Some(GitOps::current_branch(&repo)?)
        };

        let is_clean = GitOps::is_clean(&repo)?;
        let uncommitted_files = if !is_clean {
            GitOps::get_uncommitted_files(&repo)?
        } else {
            Vec::new()
        };

        let (ahead, behind) = if !is_detached {
            GitOps::get_ahead_behind(&repo).unwrap_or((0, 0))
        } else {
            (0, 0)
        };

        Ok(SubmoduleStatus {
            name: submodule.name.clone(),
            path: submodule.path.display().to_string(),
            current_branch,
            is_clean,
            is_detached,
            ahead,
            behind,
            uncommitted_files,
            exists: true,
        })
    }

    /// Initialize submodule
    pub fn init<P: AsRef<Path>>(repo_root: P, submodule: &Submodule, recursive: bool) -> Result<()> {
        let mut cmd = Command::new("git");
        cmd.arg("submodule")
            .arg("update")
            .arg("--init");

        if recursive {
            cmd.arg("--recursive");
        }

        cmd.arg("--")
            .arg(&submodule.path)
            .current_dir(repo_root.as_ref());

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to initialize submodule '{}': {}",
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Checkout a branch in submodule
    pub fn checkout<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        branch: &str,
        create: bool,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let mut cmd = Command::new("git");
        cmd.arg("checkout");

        if create {
            cmd.arg("-b");
        }

        cmd.arg(branch).current_dir(&submodule_path);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to checkout '{}' in '{}': {}",
                branch,
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Pull latest changes in submodule
    pub fn pull<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        rebase: bool,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let mut cmd = Command::new("git");
        cmd.arg("pull");

        if rebase {
            cmd.arg("--rebase");
        }

        cmd.current_dir(&submodule_path);

        let output = cmd.output()?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            if stderr.contains("CONFLICT") {
                return Err(SuperGitError::MergeConflict(submodule.name.clone()));
            }
            return Err(SuperGitError::Other(format!(
                "Failed to pull in '{}': {}",
                submodule.name, stderr
            )));
        }

        Ok(())
    }

    /// Push changes in submodule
    pub fn push<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        force_with_lease: bool,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let mut cmd = Command::new("git");
        cmd.arg("push");

        if force_with_lease {
            cmd.arg("--force-with-lease");
        }

        // Push current branch to origin
        cmd.arg("--set-upstream")
            .arg("origin")
            .arg("HEAD")
            .current_dir(&submodule_path);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to push in '{}': {}",
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Commit changes in submodule
    pub fn commit<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        message: &str,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        // First, add all changes
        let add_output = Command::new("git")
            .arg("add")
            .arg("-A")
            .current_dir(&submodule_path)
            .output()?;

        if !add_output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to stage changes in '{}': {}",
                submodule.name,
                String::from_utf8_lossy(&add_output.stderr)
            )));
        }

        // Then commit
        let commit_output = Command::new("git")
            .arg("commit")
            .arg("-m")
            .arg(message)
            .current_dir(&submodule_path)
            .output()?;

        if !commit_output.status.success() {
            let stderr = String::from_utf8_lossy(&commit_output.stderr);
            // Ignore "nothing to commit" errors
            if !stderr.contains("nothing to commit") && !stderr.contains("nothing added") {
                return Err(SuperGitError::Other(format!(
                    "Failed to commit in '{}': {}",
                    submodule.name, stderr
                )));
            }
        }

        Ok(())
    }

    /// Create a new branch in submodule
    pub fn create_branch<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        branch: &str,
        from_branch: Option<&str>,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let mut cmd = Command::new("git");
        cmd.arg("checkout").arg("-b").arg(branch);

        if let Some(from) = from_branch {
            cmd.arg(from);
        }

        cmd.current_dir(&submodule_path);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to create branch '{}' in '{}': {}",
                branch,
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Delete a branch in submodule
    pub fn delete_branch<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        branch: &str,
        force: bool,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let mut cmd = Command::new("git");
        cmd.arg("branch");

        if force {
            cmd.arg("-D");
        } else {
            cmd.arg("-d");
        }

        cmd.arg(branch).current_dir(&submodule_path);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to delete branch '{}' in '{}': {}",
                branch,
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Delete remote branch in submodule
    pub fn delete_remote_branch<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        branch: &str,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let output = Command::new("git")
            .arg("push")
            .arg("origin")
            .arg("--delete")
            .arg(branch)
            .current_dir(&submodule_path)
            .output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to delete remote branch '{}' in '{}': {}",
                branch,
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }

    /// Fetch in submodule
    pub fn fetch<P: AsRef<Path>>(
        repo_root: P,
        submodule: &Submodule,
        all_remotes: bool,
    ) -> Result<()> {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            return Err(SuperGitError::SubmoduleNotFound(submodule.name.clone()));
        }

        let mut cmd = Command::new("git");
        cmd.arg("fetch");

        if all_remotes {
            cmd.arg("--all");
        } else {
            cmd.arg("origin");
        }

        cmd.current_dir(&submodule_path);

        let output = cmd.output()?;

        if !output.status.success() {
            return Err(SuperGitError::Other(format!(
                "Failed to fetch in '{}': {}",
                submodule.name,
                String::from_utf8_lossy(&output.stderr)
            )));
        }

        Ok(())
    }
}
