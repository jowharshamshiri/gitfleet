use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    rebase: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Pulling latest changes in {} submodule(s){}",
        gitmodules.submodules.len(),
        if rebase { " (with rebase)" } else { "" }
    ));

    if dry_run {
        Formatter::print_warning("DRY RUN: No changes will be made");
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

        // Check for uncommitted changes
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

        let is_clean = match GitOps::is_clean(&repo) {
            Ok(c) => c,
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to check status of '{}': {}",
                    submodule.name, e
                ));
                failed += 1;
                continue;
            }
        };

        if !is_clean {
            Formatter::print_warning(&format!(
                "Submodule '{}' has uncommitted changes, skipping",
                submodule.name
            ));
            skipped += 1;
            continue;
        }

        // Check if in detached HEAD
        let is_detached = match GitOps::is_detached_head(&repo) {
            Ok(d) => d,
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to check HEAD state of '{}': {}",
                    submodule.name, e
                ));
                failed += 1;
                continue;
            }
        };

        if is_detached {
            Formatter::print_warning(&format!(
                "Submodule '{}' is in detached HEAD state, skipping",
                submodule.name
            ));
            skipped += 1;
            continue;
        }

        if dry_run {
            Formatter::print_info("Would pull latest changes");
            successful += 1;
            continue;
        }

        // Perform pull
        match SubmoduleOps::pull(&repo_root, submodule, rebase) {
            Ok(_) => {
                Formatter::print_success(&format!("Pulled latest changes in '{}'", submodule.name));
                successful += 1;
            }
            Err(SuperGitError::MergeConflict(_)) => {
                Formatter::print_error(&format!(
                    "Merge conflict in '{}'. Resolve conflicts and try again.",
                    submodule.name
                ));
                failed += 1;
            }
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to pull in '{}': {}",
                    submodule.name, e
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
            "{} submodule(s) failed to pull",
            failed
        )));
    }

    Ok(())
}
