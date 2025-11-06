use crate::config::GitModules;
use crate::error::Result;
use crate::git::SubmoduleOps;
use crate::output::Formatter;
use rayon::prelude::*;
use std::path::Path;

pub fn execute<P: AsRef<Path> + Sync>(repo_root: P, _verbose: bool) -> Result<()> {
    let gitmodules = GitModules::parse(&repo_root)?;

    Formatter::print_info(&format!(
        "Found {} submodule(s)",
        gitmodules.submodules.len()
    ));

    // Get status for all submodules in parallel
    let statuses: Vec<_> = gitmodules
        .submodules
        .par_iter()
        .map(|submodule| SubmoduleOps::get_status(&repo_root, submodule))
        .collect::<Result<Vec<_>>>()?;

    Formatter::print_status(&statuses);

    Ok(())
}
