use crate::error::{Result, SuperGitError};
use git2::{Branch, BranchType, Repository, StatusOptions};
use std::path::Path;

pub struct GitOps;

impl GitOps {
    /// Open a repository at the given path
    pub fn open_repo<P: AsRef<Path>>(path: P) -> Result<Repository> {
        Repository::open(path).map_err(|_| SuperGitError::NotARepository)
    }

    /// Get current branch name
    pub fn current_branch(repo: &Repository) -> Result<String> {
        let head = repo.head()?;

        if !head.is_branch() {
            return Err(SuperGitError::Other("Detached HEAD".to_string()));
        }

        Ok(head
            .shorthand()
            .ok_or_else(|| SuperGitError::Other("Invalid branch name".to_string()))?
            .to_string())
    }

    /// Check if working directory is clean
    pub fn is_clean(repo: &Repository) -> Result<bool> {
        let statuses = repo.statuses(Some(StatusOptions::new().include_untracked(true)))?;
        Ok(statuses.is_empty())
    }

    /// Get list of uncommitted files
    pub fn get_uncommitted_files(repo: &Repository) -> Result<Vec<String>> {
        let mut files = Vec::new();
        let statuses = repo.statuses(Some(StatusOptions::new().include_untracked(true)))?;

        for entry in statuses.iter() {
            if let Some(path) = entry.path() {
                files.push(path.to_string());
            }
        }

        Ok(files)
    }

    /// Check if branch exists locally
    pub fn branch_exists(repo: &Repository, branch_name: &str) -> Result<bool> {
        match repo.find_branch(branch_name, BranchType::Local) {
            Ok(_) => Ok(true),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Check if remote branch exists
    pub fn remote_branch_exists(repo: &Repository, branch_name: &str) -> Result<bool> {
        let remote_branch = format!("origin/{}", branch_name);
        match repo.find_branch(&remote_branch, BranchType::Remote) {
            Ok(_) => Ok(true),
            Err(e) if e.code() == git2::ErrorCode::NotFound => Ok(false),
            Err(e) => Err(e.into()),
        }
    }

    /// Get ahead/behind counts for current branch
    pub fn get_ahead_behind(repo: &Repository) -> Result<(usize, usize)> {
        let head = repo.head()?;
        let head_oid = head.target().ok_or_else(|| {
            SuperGitError::Other("Could not get HEAD target".to_string())
        })?;

        // Get upstream branch
        let branch = Branch::wrap(head);
        let upstream = match branch.upstream() {
            Ok(u) => u,
            Err(_) => return Ok((0, 0)), // No upstream, return 0,0
        };

        let upstream_oid = upstream.get().target().ok_or_else(|| {
            SuperGitError::Other("Could not get upstream target".to_string())
        })?;

        let (ahead, behind) = repo.graph_ahead_behind(head_oid, upstream_oid)?;
        Ok((ahead, behind))
    }

    /// Fetch from remote
    pub fn fetch(repo: &Repository, remote_name: &str) -> Result<()> {
        let mut remote = repo.find_remote(remote_name)?;

        let mut fetch_options = git2::FetchOptions::new();
        fetch_options.download_tags(git2::AutotagOption::All);

        remote.fetch(&[] as &[&str], Some(&mut fetch_options), None)?;
        Ok(())
    }

    /// Check if repository is in detached HEAD state
    pub fn is_detached_head(repo: &Repository) -> Result<bool> {
        Ok(repo.head_detached()?)
    }

    /// Get list of all local branches
    pub fn get_local_branches(repo: &Repository) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        for branch in repo.branches(Some(BranchType::Local))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }
        Ok(branches)
    }

    /// Get list of all remote branches
    pub fn get_remote_branches(repo: &Repository) -> Result<Vec<String>> {
        let mut branches = Vec::new();
        for branch in repo.branches(Some(BranchType::Remote))? {
            let (branch, _) = branch?;
            if let Some(name) = branch.name()? {
                branches.push(name.to_string());
            }
        }
        Ok(branches)
    }

    /// Get the default branch name (main or master)
    pub fn get_default_branch(repo: &Repository) -> Result<String> {
        // Try common default branches
        for branch in &["main", "master"] {
            if Self::branch_exists(repo, branch)? {
                return Ok(branch.to_string());
            }
        }

        // If neither exists, try to get the first branch
        let branches = Self::get_local_branches(repo)?;
        branches
            .first()
            .cloned()
            .ok_or_else(|| SuperGitError::Other("No branches found".to_string()))
    }
}
