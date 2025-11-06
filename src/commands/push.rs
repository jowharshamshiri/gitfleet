use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    force_with_lease: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Pushing changes in superproject + {} submodule(s){}",
        gitmodules.submodules.len(),
        if force_with_lease {
            " (with --force-with-lease)"
        } else {
            ""
        }
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

        // Check if in detached HEAD
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
            Formatter::print_info("Would push changes");
            successful += 1;
            continue;
        }

        // Perform push
        match SubmoduleOps::push(&repo_root, submodule, force_with_lease) {
            Ok(_) => {
                Formatter::print_success(&format!("Pushed changes in '{}'", submodule.name));
                successful += 1;
            }
            Err(e) => {
                // Check if it's a "no changes" error
                let err_str = e.to_string();
                if err_str.contains("Everything up-to-date") || err_str.contains("up to date") {
                    Formatter::print_info(&format!("'{}' is up to date", submodule.name));
                    successful += 1;
                } else {
                    Formatter::print_error(&format!(
                        "Failed to push in '{}': {}",
                        submodule.name, e
                    ));
                    failed += 1;
                }
            }
        }
    }

    // Push superproject after submodules
    if !dry_run {
        use std::process::Command;

        if verbose {
            Formatter::print_submodule_header("[SUPERPROJECT]");
        }

        let mut cmd = Command::new("git");
        cmd.arg("push");
        if force_with_lease {
            cmd.arg("--force-with-lease");
        }
        cmd.arg("--set-upstream")
            .arg("origin")
            .arg("HEAD")
            .current_dir(&repo_root);

        let output = cmd.output()?;
        if output.status.success() {
            Formatter::print_success("Pushed superproject");
            successful += 1;
        } else {
            let err_str = String::from_utf8_lossy(&output.stderr);
            if err_str.contains("Everything up-to-date") || err_str.contains("up to date") {
                Formatter::print_info("Superproject is up to date");
                successful += 1;
            } else {
                Formatter::print_error(&format!(
                    "Failed to push superproject: {}",
                    err_str
                ));
                failed += 1;
            }
        }

        Formatter::print_summary(successful, failed, skipped);
    }

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} repository/submodule(s) failed to push",
            failed
        )));
    }

    Ok(())
}
