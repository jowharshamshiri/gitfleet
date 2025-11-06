use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    init: bool,
    recursive: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Synchronizing {} submodule(s) based on .gitmodules",
        gitmodules.submodules.len()
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

        // Initialize if needed
        if !submodule_path.exists() {
            if !init {
                Formatter::print_warning(&format!(
                    "Submodule '{}' not initialized. Use --init to initialize.",
                    submodule.name
                ));
                skipped += 1;
                continue;
            }

            if dry_run {
                Formatter::print_info("Would initialize submodule");
            } else {
                Formatter::print_info(&format!("Initializing '{}'...", submodule.name));
                match SubmoduleOps::init(&repo_root, submodule, recursive) {
                    Ok(_) => {
                        Formatter::print_success(&format!("Initialized '{}'", submodule.name));
                    }
                    Err(e) => {
                        Formatter::print_error(&format!(
                            "Failed to initialize '{}': {}",
                            submodule.name, e
                        ));
                        failed += 1;
                        continue;
                    }
                }
            }
        }

        if dry_run && submodule_path.exists() {
            Formatter::print_info("Would fetch and sync");
            successful += 1;
            continue;
        }

        if !submodule_path.exists() {
            continue;
        }

        // Fetch latest
        if verbose {
            Formatter::print_info("Fetching...");
        }
        if let Err(e) = SubmoduleOps::fetch(&repo_root, submodule, false) {
            Formatter::print_error(&format!("Failed to fetch '{}': {}", submodule.name, e));
            failed += 1;
            continue;
        }

        // Get target branch from .gitmodules or current branch
        let target_branch = if let Some(ref branch) = submodule.branch {
            branch.clone()
        } else {
            // Try to get current branch
            match GitOps::open_repo(&submodule_path) {
                Ok(repo) => {
                    match GitOps::current_branch(&repo) {
                        Ok(b) => b,
                        Err(_) => {
                            Formatter::print_warning(&format!(
                                "No branch specified in .gitmodules and cannot detect current branch for '{}', skipping",
                                submodule.name
                            ));
                            skipped += 1;
                            continue;
                        }
                    }
                }
                Err(e) => {
                    Formatter::print_error(&format!(
                        "Failed to open '{}': {}",
                        submodule.name, e
                    ));
                    failed += 1;
                    continue;
                }
            }
        };

        // Checkout target branch
        if verbose {
            Formatter::print_info(&format!("Checking out '{}'...", target_branch));
        }
        if let Err(e) = SubmoduleOps::checkout(&repo_root, submodule, &target_branch, false) {
            Formatter::print_error(&format!(
                "Failed to checkout '{}' in '{}': {}",
                target_branch, submodule.name, e
            ));
            failed += 1;
            continue;
        }

        // Pull latest
        if verbose {
            Formatter::print_info("Pulling...");
        }
        match SubmoduleOps::pull(&repo_root, submodule, false) {
            Ok(_) => {
                Formatter::print_success(&format!(
                    "Synced '{}' on branch '{}'",
                    submodule.name, target_branch
                ));
                successful += 1;
            }
            Err(e) => {
                Formatter::print_error(&format!("Failed to pull '{}': {}", submodule.name, e));
                failed += 1;
            }
        }
    }

    if !dry_run {
        Formatter::print_summary(successful, failed, skipped);
    }

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} submodule(s) failed to sync",
            failed
        )));
    }

    Ok(())
}
