use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::SubmoduleOps;
use crate::output::Formatter;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(repo_root: P, recursive: bool, dry_run: bool, verbose: bool) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Initializing {} submodule(s){}",
        gitmodules.submodules.len(),
        if recursive { " (recursively)" } else { "" }
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

        if submodule_path.exists() {
            if verbose {
                Formatter::print_info(&format!("'{}' already initialized", submodule.name));
            }
            skipped += 1;
            continue;
        }

        if dry_run {
            Formatter::print_info(&format!("Would initialize '{}'", submodule.name));
            successful += 1;
            continue;
        }

        match SubmoduleOps::init(&repo_root, submodule, recursive) {
            Ok(_) => {
                Formatter::print_success(&format!("Initialized '{}'", submodule.name));
                successful += 1;
            }
            Err(e) => {
                Formatter::print_error(&format!("Failed to initialize '{}': {}", submodule.name, e));
                failed += 1;
            }
        }
    }

    if !dry_run {
        Formatter::print_summary(successful, failed, skipped);
    }

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} submodule(s) failed to initialize",
            failed
        )));
    }

    Ok(())
}
