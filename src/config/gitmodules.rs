use crate::error::{Result, SuperGitError};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct Submodule {
    pub name: String,
    pub path: PathBuf,
    pub url: String,
    pub branch: Option<String>,
}

pub struct GitModules {
    pub submodules: Vec<Submodule>,
}

impl GitModules {
    /// Parse .gitmodules file from repository root
    pub fn parse<P: AsRef<Path>>(repo_root: P) -> Result<Self> {
        let gitmodules_path = repo_root.as_ref().join(".gitmodules");

        if !gitmodules_path.exists() {
            return Err(SuperGitError::NoSubmodules);
        }

        let content = std::fs::read_to_string(&gitmodules_path)
            .map_err(|e| SuperGitError::GitModulesParse(e.to_string()))?;

        let mut submodules = Vec::new();
        let mut current_name: Option<String> = None;
        let mut current_path: Option<String> = None;
        let mut current_url: Option<String> = None;
        let mut current_branch: Option<String> = None;

        for line in content.lines() {
            let line = line.trim();

            // Check for section header [submodule "name"]
            if line.starts_with("[submodule") {
                // Save previous submodule if exists
                if let (Some(name), Some(path), Some(url)) = (current_name.take(), current_path.take(), current_url.take()) {
                    submodules.push(Submodule {
                        name,
                        path: PathBuf::from(path),
                        url,
                        branch: current_branch.take(),
                    });
                }

                // Extract submodule name from [submodule "name"]
                if let Some(start) = line.find('"') {
                    if let Some(end) = line[start + 1..].find('"') {
                        current_name = Some(line[start + 1..start + 1 + end].to_string());
                    }
                }
            } else if let Some((key, value)) = line.split_once('=') {
                let key = key.trim();
                let value = value.trim();

                match key {
                    "path" => current_path = Some(value.to_string()),
                    "url" => current_url = Some(value.to_string()),
                    "branch" => current_branch = Some(value.to_string()),
                    _ => {}
                }
            }
        }

        // Save last submodule
        if let (Some(name), Some(path), Some(url)) = (current_name, current_path, current_url) {
            submodules.push(Submodule {
                name,
                path: PathBuf::from(path),
                url,
                branch: current_branch,
            });
        }

        if submodules.is_empty() {
            return Err(SuperGitError::NoSubmodules);
        }

        Ok(GitModules { submodules })
    }

    /// Update branch field for a submodule in .gitmodules
    pub fn update_branch<P: AsRef<Path>>(
        repo_root: P,
        submodule_path: &str,
        branch: &str,
    ) -> Result<()> {
        let gitmodules_path = repo_root.as_ref().join(".gitmodules");

        let content = std::fs::read_to_string(&gitmodules_path)
            .map_err(|e| SuperGitError::GitModulesParse(e.to_string()))?;

        let mut new_content = String::new();
        let mut in_target_section = false;
        let mut branch_updated = false;

        for line in content.lines() {
            let trimmed = line.trim();

            // Check if we're entering a new section
            if trimmed.starts_with("[submodule") {
                in_target_section = false;
            }

            // Check if this is our target section
            if trimmed.starts_with("[submodule") {
                // Read ahead to check if path matches
                let check_lines: Vec<&str> = content.lines().collect();
                if let Some(current_idx) = check_lines.iter().position(|&l| l == line) {
                    for check_line in &check_lines[current_idx + 1..] {
                        let check_trimmed = check_line.trim();
                        if check_trimmed.starts_with("[") {
                            break;
                        }
                        if let Some((key, value)) = check_trimmed.split_once('=') {
                            if key.trim() == "path" && value.trim() == submodule_path {
                                in_target_section = true;
                                break;
                            }
                        }
                    }
                }
            }

            // Update or add branch line
            if in_target_section {
                if let Some((key, _)) = trimmed.split_once('=') {
                    if key.trim() == "branch" {
                        new_content.push_str(&format!("\tbranch = {}\n", branch));
                        branch_updated = true;
                        continue;
                    }
                }
                // If we're at the end of this section and haven't added branch, add it
                if trimmed.is_empty() || trimmed.starts_with("[") {
                    if !branch_updated && in_target_section {
                        new_content.push_str(&format!("\tbranch = {}\n", branch));
                        branch_updated = true;
                    }
                }
            }

            new_content.push_str(line);
            new_content.push('\n');
        }

        // Add branch at end if still not added
        if in_target_section && !branch_updated {
            new_content.push_str(&format!("\tbranch = {}\n", branch));
        }

        std::fs::write(&gitmodules_path, new_content)
            .map_err(|e| SuperGitError::GitModulesParse(e.to_string()))?;

        Ok(())
    }
}
