use crate::config::GitModules;
use crate::error::Result;
use crate::git::{GitOps, SubmoduleOps, SubmoduleStatus};
use crate::output::Formatter;
use rayon::prelude::*;
use std::path::Path;

pub fn execute<P: AsRef<Path> + Sync>(repo_root: P, _verbose: bool) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Superproject + {} submodule(s)",
        gitmodules.submodules.len()
    ));

    // Get status for superproject
    let repo = GitOps::open_repo(&repo_root)?;
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

    let super_status = SubmoduleStatus {
        name: "[SUPERPROJECT]".to_string(),
        path: ".".to_string(),
        current_branch,
        is_clean,
        is_detached,
        ahead,
        behind,
        uncommitted_files,
        exists: true,
    };

    // Get status for all submodules in parallel
    let mut statuses: Vec<_> = gitmodules
        .submodules
        .par_iter()
        .map(|submodule| SubmoduleOps::get_status(&repo_root, submodule))
        .collect::<Result<Vec<_>>>()?;

    // Prepend superproject status
    statuses.insert(0, super_status);

    Formatter::print_status(&statuses);

    Ok(())
}
