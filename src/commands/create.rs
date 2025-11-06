use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::{GitOps, SubmoduleOps};
use crate::output::Formatter;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(
    repo_root: P,
    branch: &str,
    from_branch: Option<&str>,
    push: bool,
    dry_run: bool,
    verbose: bool,
) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Creating branch '{}' in superproject + {} submodule(s){}",
        branch,
        gitmodules.submodules.len(),
        if let Some(from) = from_branch {
            format!(" from '{}'", from)
        } else {
            String::new()
        }
    ));

    if dry_run {
        Formatter::print_warning("DRY RUN: No changes will be made");
    }

    let mut successful = 0;
    let mut failed = 0;
    let mut skipped = 0;

    // Create branch in superproject first
    if verbose {
        Formatter::print_submodule_header("[SUPERPROJECT]");
    }

    let super_repo = GitOps::open_repo(&repo_root)?;
    let super_branch_exists = GitOps::branch_exists(&super_repo, branch)?;

    if super_branch_exists {
        Formatter::print_warning(&format!(
            "Branch '{}' already exists in superproject, skipping creation",
            branch
        ));
        skipped += 1;
    } else {
        if !dry_run {
            use std::process::Command;
            let mut cmd = Command::new("git");
            cmd.arg("checkout").arg("-b").arg(branch);
            if let Some(from) = from_branch {
                cmd.arg(from);
            }
            cmd.current_dir(&repo_root);

            let output = cmd.output()?;
            if !output.status.success() {
                Formatter::print_error(&format!(
                    "Failed to create branch '{}' in superproject: {}",
                    branch,
                    String::from_utf8_lossy(&output.stderr)
                ));
                return Err(SuperGitError::Other("Superproject branch creation failed".to_string()));
            }

            Formatter::print_success(&format!("Created branch '{}' in superproject", branch));
            successful += 1;

            // Push if requested
            if push {
                let push_output = Command::new("git")
                    .arg("push")
                    .arg("--set-upstream")
                    .arg("origin")
                    .arg(branch)
                    .current_dir(&repo_root)
                    .output()?;

                if push_output.status.success() {
                    Formatter::print_success(&format!("Pushed '{}' in superproject", branch));
                } else {
                    Formatter::print_error(&format!(
                        "Failed to push '{}' in superproject: {}",
                        branch,
                        String::from_utf8_lossy(&push_output.stderr)
                    ));
                }
            }
        } else {
            Formatter::print_info(&format!("Would create branch '{}'", branch));
            successful += 1;
        }
    }

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

        // Check if branch already exists
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

        if branch_exists {
            Formatter::print_warning(&format!(
                "Branch '{}' already exists in '{}', skipping",
                branch, submodule.name
            ));
            skipped += 1;
            continue;
        }

        if dry_run {
            Formatter::print_info(&format!("Would create branch '{}'", branch));
            successful += 1;
            continue;
        }

        // Create branch
        match SubmoduleOps::create_branch(&repo_root, submodule, branch, from_branch) {
            Ok(_) => {
                Formatter::print_success(&format!(
                    "Created branch '{}' in '{}'",
                    branch, submodule.name
                ));
                successful += 1;

                // Push if requested
                if push {
                    match SubmoduleOps::push(&repo_root, submodule, false) {
                        Ok(_) => {
                            Formatter::print_success(&format!(
                                "Pushed '{}' in '{}'",
                                branch, submodule.name
                            ));
                        }
                        Err(e) => {
                            Formatter::print_error(&format!(
                                "Failed to push '{}' in '{}': {}",
                                branch, submodule.name, e
                            ));
                        }
                    }
                }
            }
            Err(e) => {
                Formatter::print_error(&format!(
                    "Failed to create branch '{}' in '{}': {}",
                    branch, submodule.name, e
                ));
                failed += 1;
            }
        }
    }

    if !dry_run {
        Formatter::print_summary(successful, failed, skipped);

        // Update .gitmodules to track new branch
        if successful > 0 {
            Formatter::print_info("Updating .gitmodules branch fields...");
            for submodule in &gitmodules.submodules {
                let _ = GitModules::update_branch(
                    &repo_root,
                    &submodule.path.display().to_string(),
                    branch,
                );
            }
            Formatter::print_success("Updated .gitmodules");
        }
    }

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} submodule(s) failed to create branch",
            failed
        )));
    }

    Ok(())
}
