use crate::git::SubmoduleStatus;
use colored::*;

pub struct Formatter;

impl Formatter {
    pub fn print_status(statuses: &[SubmoduleStatus]) {
        println!("\n{}", "Submodule Status:".bold().underline());
        println!();

        for status in statuses {
            if !status.exists {
                println!(
                    "  {} {} {}",
                    "●".red(),
                    status.name.bold(),
                    "[NOT INITIALIZED]".red()
                );
                continue;
            }

            let indicator = if status.is_clean && status.ahead == 0 && status.behind == 0 {
                "●".green()
            } else if status.is_detached {
                "●".yellow()
            } else {
                "●".yellow()
            };

            print!("  {} {}", indicator, status.name.bold());

            if let Some(ref branch) = status.current_branch {
                print!(" ({})", branch.cyan());
            } else if status.is_detached {
                print!(" {}", "[DETACHED HEAD]".red());
            }

            println!();

            if !status.is_clean {
                println!(
                    "    {} {} uncommitted file(s)",
                    "!".yellow(),
                    status.uncommitted_files.len()
                );
            }

            if status.ahead > 0 {
                println!("    {} {} commit(s) ahead", "↑".green(), status.ahead);
            }

            if status.behind > 0 {
                println!("    {} {} commit(s) behind", "↓".red(), status.behind);
            }
        }

        println!();
    }

    pub fn print_success(message: &str) {
        println!("{} {}", "✓".green(), message);
    }

    pub fn print_error(message: &str) {
        eprintln!("{} {}", "✗".red(), message);
    }

    pub fn print_warning(message: &str) {
        println!("{} {}", "!".yellow(), message);
    }

    pub fn print_info(message: &str) {
        println!("{} {}", "ℹ".cyan(), message);
    }

    pub fn print_submodule_header(name: &str) {
        println!("\n{} {}...", "→".cyan(), name.bold());
    }

    pub fn print_summary(successful: usize, failed: usize, skipped: usize) {
        println!("\n{}", "Summary:".bold().underline());
        println!(
            "  {} successful, {} failed, {} skipped",
            successful.to_string().green(),
            failed.to_string().red(),
            skipped.to_string().yellow()
        );
    }

    pub fn print_branches(submodule_name: &str, branches: &[String], current: Option<&str>) {
        println!("\n{}", submodule_name.bold());
        for branch in branches {
            if Some(branch.as_str()) == current {
                println!("  {} {}", "*".green(), branch.green());
            } else {
                println!("    {}", branch);
            }
        }
    }
}
