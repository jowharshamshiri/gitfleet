mod cli;
mod commands;
mod config;
mod error;
mod git;
mod output;

use clap::Parser;
use cli::{Cli, Commands};
use output::Formatter;
use std::env;
use std::path::PathBuf;

fn main() {
    let cli = Cli::parse();

    // Get repository root (current directory by default)
    let repo_root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));

    // Execute command
    let result = match cli.command {
        Commands::Status => commands::status::execute(&repo_root, cli.verbose),

        Commands::Checkout { branch, create } => {
            commands::checkout::execute(&repo_root, &branch, create, cli.dry_run, cli.verbose)
        }

        Commands::Pull { rebase } => {
            commands::pull::execute(&repo_root, rebase, cli.dry_run, cli.verbose)
        }

        Commands::Push { force_with_lease } => {
            commands::push::execute(&repo_root, force_with_lease, cli.dry_run, cli.verbose)
        }

        Commands::Create { branch, from, push } => commands::create::execute(
            &repo_root,
            &branch,
            from.as_deref(),
            push,
            cli.dry_run,
            cli.verbose,
        ),

        Commands::Commit { message, push } => {
            commands::commit::execute(&repo_root, &message, push, cli.dry_run, cli.verbose)
        }

        Commands::Sync { init, recursive } => {
            commands::sync::execute(&repo_root, init, recursive, cli.dry_run, cli.verbose)
        }

        Commands::Delete {
            branch,
            remote,
            force,
        } => commands::delete::execute(
            &repo_root,
            &branch,
            remote,
            force,
            cli.dry_run,
            cli.verbose,
        ),

        Commands::Fetch { all } => {
            commands::fetch::execute(&repo_root, all, cli.dry_run, cli.verbose)
        }

        Commands::Branch { all } => commands::branch::execute(&repo_root, all, cli.verbose),

        Commands::Init { recursive } => {
            commands::init::execute(&repo_root, recursive, cli.dry_run, cli.verbose)
        }
    };

    // Handle result
    match result {
        Ok(_) => {
            if cli.verbose {
                Formatter::print_success("Command completed successfully");
            }
        }
        Err(e) => {
            Formatter::print_error(&format!("Error: {}", e));
            std::process::exit(1);
        }
    }
}
