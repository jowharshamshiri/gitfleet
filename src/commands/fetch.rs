use crate::config::GitModules;
use crate::error::{Result, SuperGitError};
use crate::git::SubmoduleOps;
use crate::output::Formatter;
use rayon::prelude::*;
use std::path::Path;

pub fn execute<P: AsRef<Path> + Sync>(repo_root: P, all: bool, dry_run: bool, verbose: bool) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Fetching updates for {} submodule(s){}",
        gitmodules.submodules.len(),
        if all { " (all remotes)" } else { "" }
    ));

    if dry_run {
        Formatter::print_warning("DRY RUN: No changes will be made");
        return Ok(());
    }

    // Fetch in parallel for speed
    let results: Vec<_> = gitmodules
        .submodules
        .par_iter()
        .map(|submodule| {
            let submodule_path = repo_root.as_ref().join(&submodule.path);

            if !submodule_path.exists() {
                return (submodule.name.clone(), Err("not initialized".to_string()));
            }

            match SubmoduleOps::fetch(&repo_root, submodule, all) {
                Ok(_) => (submodule.name.clone(), Ok(())),
                Err(e) => (submodule.name.clone(), Err(e.to_string())),
            }
        })
        .collect();

    let mut successful = 0;
    let mut failed = 0;
    let mut skipped = 0;

    for (name, result) in results {
        match result {
            Ok(_) => {
                if verbose {
                    Formatter::print_success(&format!("Fetched '{}'", name));
                }
                successful += 1;
            }
            Err(e) => {
                if e == "not initialized" {
                    if verbose {
                        Formatter::print_warning(&format!("'{}' not initialized, skipping", name));
                    }
                    skipped += 1;
                } else {
                    Formatter::print_error(&format!("Failed to fetch '{}': {}", name, e));
                    failed += 1;
                }
            }
        }
    }

    Formatter::print_summary(successful, failed, skipped);

    if failed > 0 {
        return Err(SuperGitError::Other(format!(
            "{} submodule(s) failed to fetch",
            failed
        )));
    }

    Ok(())
}
