use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "gitfleet")]
#[command(about = "A Git submodule manager for synchronized superbranches", long_about = None)]
#[command(version)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Dry run mode (show what would happen without executing)
    #[arg(short = 'n', long, global = true)]
    pub dry_run: bool,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Show status of all submodules
    Status,

    /// Checkout a branch across all submodules
    Checkout {
        /// Branch name to checkout
        branch: String,

        /// Create the branch if it doesn't exist
        #[arg(short, long)]
        create: bool,
    },

    /// Pull latest changes in all submodules
    Pull {
        /// Use rebase instead of merge
        #[arg(short, long)]
        rebase: bool,
    },

    /// Push changes in all submodules
    Push {
        /// Force push with lease
        #[arg(short, long)]
        force_with_lease: bool,
    },

    /// Create a new branch across all submodules
    Create {
        /// Branch name to create
        branch: String,

        /// Base branch to create from
        #[arg(short, long)]
        from: Option<String>,

        /// Push the new branch immediately
        #[arg(short, long)]
        push: bool,
    },

    /// Commit changes across all submodules
    Commit {
        /// Commit message
        #[arg(short, long)]
        message: String,

        /// Push after committing
        #[arg(short, long)]
        push: bool,
    },

    /// Synchronize all submodules based on .gitmodules
    Sync {
        /// Initialize missing submodules
        #[arg(short, long)]
        init: bool,

        /// Recursive sync for nested submodules
        #[arg(short, long)]
        recursive: bool,
    },

    /// Delete a branch from all submodules
    Delete {
        /// Branch name to delete
        branch: String,

        /// Also delete from remote
        #[arg(short, long)]
        remote: bool,

        /// Force delete
        #[arg(short, long)]
        force: bool,
    },

    /// Fetch updates for all submodules
    Fetch {
        /// Fetch from all remotes
        #[arg(short, long)]
        all: bool,
    },

    /// List branches across all submodules
    Branch {
        /// Show all branches including remotes
        #[arg(short, long)]
        all: bool,
    },

    /// Initialize all submodules
    Init {
        /// Recursively initialize nested submodules
        #[arg(short, long)]
        recursive: bool,
    },
}
