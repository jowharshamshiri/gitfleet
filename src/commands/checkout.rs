use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use dialoguer::Confirm;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    branch: &str,
    create: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Checking out '{}' in {} submodule(s){}",
        branch,
        gitmodules.submodules.len(),
        if create { " (creating if needed)" } else { "" }
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

        if !is_clean && !dry_run {
            Formatter::print_warning(&format!(
                "Submodule '{}' has uncommitted changes",
                submodule.name
            ));

            let should_continue = Confirm::new()
                .with_prompt("Do you want to continue? (changes may be lost)")
                .default(false)
                .interact()
                .unwrap_or(false);

            if !should_continue {
                Formatter::print_info("Skipping");
                skipped += 1;
                continue;
            }
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

        if !branch_exists && !create {
            Formatter::print_error(&format!(
                "Branch '{}' does not exist in '{}'. Use --create to create it.",
                branch, submodule.name
            ));
            failed += 1;
            continue;
        }

        if dry_run {
            if branch_exists {
                Formatter::print_info(&format!("Would checkout '{}'", branch));
            } else {
                Formatter::print_info(&format!("Would create and checkout '{}'", branch));
            }
            successful += 1;
            continue;
        }

        // Perform checkout
        match SubmoduleOps::checkout(&repo_root, submodule, branch, !branch_exists && create) {
            Ok(_) => {
                if branch_exists {
                    Formatter::print_success(&format!(
                        "Checked out '{}' in '{}'",
                        branch, submodule.name
                    ));
                } else {
                    Formatter::print_success(&format!(
                        "Created and checked out '{}' in '{}'",
                        branch, submodule.name
                    ));
                }
                successful += 1;
            }
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to checkout '{}' in '{}': {}",
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
            "{} submodule(s) failed to checkout",
            failed
        )));
    }

    Ok(())
}
