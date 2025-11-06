use crate::config::GitModules;
use crate::error::Result;
use crate::git::GitOps;
use crate::output::Formatter;
use std::path::Path;

pub fn execute<P: AsRef<Path>>(repo_root: P, all: bool, _verbose: bool) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Listing branches for superproject + {} submodule(s)",
        gitmodules.submodules.len()
    ));

    // List branches for superproject first
    let repo = match GitOps::open_repo(&repo_root) {
        Ok(r) => r,
        Err(e) => {
            Formatter::print_error(&format!("Failed to open superproject: {}", e));
            return Err(e);
        }
    };

    let current_branch = GitOps::current_branch(&repo).ok();

    let branches = if all {
        let mut local = GitOps::get_local_branches(&repo).unwrap_or_default();
        let remote = GitOps::get_remote_branches(&repo).unwrap_or_default();
        local.extend(remote);
        local
    } else {
        GitOps::get_local_branches(&repo).unwrap_or_default()
    };

    Formatter::print_branches("[SUPERPROJECT]", &branches, current_branch.as_deref());

    for submodule in &gitmodules.submodules {
        let submodule_path = repo_root.as_ref().join(&submodule.path);

        if !submodule_path.exists() {
            Formatter::print_warning(&format!("'{}' not initialized", submodule.name));
            continue;
        }

        let repo = match GitOps::open_repo(&submodule_path) {
            Ok(r) => r,
            Err(e) => {
                Formatter::print_error(&format!("Failed to open '{}': {}", submodule.name, e));
                continue;
            }
        };

        let current_branch = GitOps::current_branch(&repo).ok();

        let branches = if all {
            let mut local = GitOps::get_local_branches(&repo).unwrap_or_default();
            let remote = GitOps::get_remote_branches(&repo).unwrap_or_default();
            local.extend(remote);
            local
        } else {
            GitOps::get_local_branches(&repo).unwrap_or_default()
        };

        Formatter::print_branches(&submodule.name, &branches, current_branch.as_deref());
    }

    Ok(())
}
