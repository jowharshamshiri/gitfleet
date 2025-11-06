use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use std::path::Path;
use std::process::Command;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    message: &str,
    push: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Committing changes in {} submodule(s)",
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

        // Check if there are changes to commit
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

        if is_clean {
            if verbose {
                Formatter::print_info(&format!("No changes in '{}'", submodule.name));
            }
            skipped += 1;
            continue;
        }

        if dry_run {
            Formatter::print_info("Would commit changes");
            successful += 1;
            continue;
        }

        // Commit changes
        match SubmoduleOps::commit(&repo_root, submodule, message) {
            Ok(_) => {
                Formatter::print_success(&format!("Committed changes in '{}'", submodule.name));
                successful += 1;

                // Push if requested
                if push {
                    match SubmoduleOps::push(&repo_root, submodule, false) {
                        Ok(_) => {
                            Formatter::print_success(&format!("Pushed changes in '{}'", submodule.name));
                        }
                        Err(e) => {
                            Formatter::print_error(&format!(
                                "Failed to push in '{}': {}",
                                submodule.name, e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to commit in '{}': {}",
                    submodule.name, e
                ));
                failed += 1;
            }
        }
    }

    if !dry_run {
        Formatter::print_summary(successful, failed, skipped);

        // Update superproject to reference new commits
        if successful > 0 {
            Formatter::print_info("Updating superproject...");
            let add_output = Command::new("git")
                .arg("add")
                .arg(".")
                .current_dir(&repo_root)
                .output()?;

            if add_output.status.success() {
                let commit_output = Command::new("git")
                    .arg("commit")
                    .arg("-m")
                    .arg(format!("Update submodules: {}", message))
                    .current_dir(&repo_root)
                    .output()?;

                if commit_output.status.success() {
                    Formatter::print_success("Updated superproject");
                } else {
                    let stderr = String::from_utf8_lossy(&commit_output.stderr);
                    if !stderr.contains("nothing to commit") {
                        Formatter::print_warning(&format!(
                            "Failed to update superproject: {}",
                            stderr
                        ));
                    }
                }
            }
        }
    }

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} submodule(s) failed to commit",
            failed
        )));
    }

    Ok(())
}
