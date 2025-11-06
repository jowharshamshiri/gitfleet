use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use dialoguer::Confirm;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    branch: &str,
    remote: bool,
    force: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Deleting branch '{}' from superproject + {} submodule(s){}",
        branch,
        gitmodules.submodules.len(),
        if remote { " (including remote)" } else { "" }
    ));

    if dry_run {
        Formatter::print_warning("DRY RUN: No changes will be made");
    }

    // Confirmation for remote deletion
    if remote && !dry_run {
        let confirmed = Confirm::new()
            .with_prompt(format!(
                "Are you sure you want to delete '{}' from remote?",
                branch
            ))
            .default(false)
            .interact()
            .unwrap_or(false);

        if !confirmed {
            return Err(SuperGitError::Cancelled);
        }
    }

    let mut successful = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for submodule in &gitmodules.submodules {
        if verbose {
            Formatter::print_submodule_header(&submodule.name);
        }

        let submodule_path = repo_root.as_ref().join(&submodule.path);

        // Check if submodule exists
        if !submodule_path.exists() {
            Formatter::print_warning(&format!(
                "Submodule '{}' not initialized, skipping",
                submodule.name
            ));
            skipped += 1;
            continue;
        }

        let repo = match GitOps::open_repo(&submodule_path) {
            Ok(r) => r,
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to open '{}': {}",
                    submodule.name, e
                ));
                failed += 1;
                continue;
            }
        };

        // Check if currently on the branch to delete
        let current_branch = match GitOps::current_branch(&repo) {
            Ok(b) => Some(b),
            Err(_) => None,
        };

        if current_branch.as_deref() == Some(branch) {
            Formatter::print_error(&format!(
                "Cannot delete '{}' in '{}': currently checked out. Switch to another branch first.",
                branch, submodule.name
            ));
            failed += 1;
            continue;
        }

        // Check if branch exists
        let branch_exists = match GitOps::branch_exists(&repo, branch) {
            Ok(exists) => exists,
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to check branch in '{}': {}",
                    submodule.name, e
                ));
                failed += 1;
                continue;
            }
        };

        if !branch_exists {
            Formatter::print_warning(&format!(
                "Branch '{}' does not exist in '{}', skipping",
                branch, submodule.name
            ));
            skipped += 1;
            continue;
        }

        if dry_run {
            Formatter::print_info(&format!("Would delete branch '{}'", branch));
            if remote {
                Formatter::print_info(&format!("Would delete remote branch '{}'", branch));
            }
            successful += 1;
            continue;
        }

        // Delete local branch
        match SubmoduleOps::delete_branch(&repo_root, submodule, branch, force) {
            Ok(_) => {
                Formatter::print_success(&format!(
                    "Deleted branch '{}' in '{}'",
                    branch, submodule.name
                ));
                successful += 1;

                // Delete remote branch if requested
                if remote {
                    match SubmoduleOps::delete_remote_branch(&repo_root, submodule, branch) {
                        Ok(_) => {
                            Formatter::print_success(&format!(
                                "Deleted remote branch '{}' in '{}'",
                                branch, submodule.name
                            ));
                        }
                        Err(e) => {
                            Formatter::print_error(&format!(
                                "Failed to delete remote branch '{}' in '{}': {}",
                                branch, submodule.name, e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to delete branch '{}' in '{}': {}",
                    branch, submodule.name, e
                ));
                failed += 1;
            }
        }
    }

    if !dry_run {
        Formatter::print_summary(successful, failed, skipped);
    }

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} submodule(s) failed to delete branch",
            failed
        )));
    }

    Ok(())
}
