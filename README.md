# SuperGit

A command-line tool for executing Git operations across multiple submodules simultaneously.

## Overview

SuperGit executes Git commands (checkout, pull, push, commit, etc.) across all submodules in a repository at once, maintaining consistent branch states. It reads submodule configuration from `.gitmodules` and provides a single interface for bulk operations.

## Features

- Execute Git operations on all submodules simultaneously
- Parallel fetch/status operations for performance
- Dry-run mode for testing operations
- Interactive prompts for destructive operations
- Reads branch configuration from `.gitmodules`

## Installation

### From Source

```bash
git clone https://github.com/yourusername/gitfleet.git
cd gitfleet
cargo build --release
sudo cp target/release/gitfleet /usr/local/bin/
```

### Verify Installation

```bash
gitfleet --version
```

## Quick Start

### 1. Initialize Your Submodules

```bash
gitfleet init
```

### 2. Check Status Across All Submodules

```bash
gitfleet status
```

### 3. Create a New "Superbranch"

```bash
gitfleet create feature-x
```

This creates the `feature-x` branch in all submodules and checks it out.

### 4. Make Changes and Commit

```bash
# Make your changes in various submodules
gitfleet commit -m "Implement feature X"
```

### 5. Push Everything

```bash
gitfleet push
```

## Commands

### `gitfleet status`

Shows the status of all submodules in a unified view.

```bash
gitfleet status
```

**Output:**
- Current branch for each submodule
- Uncommitted changes count
- Ahead/behind tracking information
- Detached HEAD warnings
- Uninitialized submodules

### `gitfleet checkout <branch>`

Checkout the same branch across all submodules.

```bash
gitfleet checkout main
gitfleet checkout -c feature-new  # Create and checkout new branch
```

**Options:**
- `-c, --create`: Create the branch if it doesn't exist

**Features:**
- Checks for uncommitted changes before switching
- Interactive prompts for safety
- Skips uninitialized submodules with warnings

### `gitfleet create <branch>`

Create a new branch across all submodules.

```bash
gitfleet create feature-auth
gitfleet create feature-auth --from main --push
```

**Options:**
- `-f, --from <branch>`: Base branch to create from
- `-p, --push`: Push the new branch immediately after creation

**Features:**
- Automatically updates `.gitmodules` branch fields
- Can create from a specific base branch
- Optionally pushes to remote immediately

### `gitfleet pull`

Pull latest changes for the current branch in all submodules.

```bash
gitfleet pull
gitfleet pull --rebase
```

**Options:**
- `-r, --rebase`: Use rebase instead of merge

**Features:**
- Skips submodules with uncommitted changes
- Detects and reports merge conflicts
- Skips detached HEAD states

### `gitfleet push`

Push current branch for all submodules.

```bash
gitfleet push
gitfleet push --force-with-lease
```

**Options:**
- `-f, --force-with-lease`: Force push with lease (safer than force)

**Features:**
- Sets up upstream tracking automatically
- Reports success/failure per submodule
- Skips detached HEAD states

### `gitfleet commit`

Commit changes across all submodules with the same message.

```bash
gitfleet commit -m "Fix authentication bug"
gitfleet commit -m "Add new feature" --push
```

**Options:**
- `-m, --message <msg>`: Commit message (required)
- `-p, --push`: Push immediately after committing

**Features:**
- Only commits submodules with changes
- Automatically updates superproject
- Can push immediately after committing

### `gitfleet sync`

Synchronize all submodules based on `.gitmodules` branch declarations.

```bash
gitfleet sync
gitfleet sync --init --recursive
```

**Options:**
- `-i, --init`: Initialize missing submodules
- `-r, --recursive`: Recursively sync nested submodules

**Features:**
- Reads target branch from `.gitmodules`
- Fetches, checks out, and pulls in one operation
- Can initialize uninitialized submodules

### `gitfleet delete <branch>`

Delete a branch from all submodules.

```bash
gitfleet delete old-feature
gitfleet delete old-feature --remote --force
```

**Options:**
- `-r, --remote`: Also delete from remote
- `-f, --force`: Force delete (delete even if not fully merged)

**Features:**
- Safety checks prevent deleting current branch
- Interactive confirmation for remote deletion
- Skips submodules that don't have the branch

### `gitfleet fetch`

Fetch updates for all submodules.

```bash
gitfleet fetch
gitfleet fetch --all
```

**Options:**
- `-a, --all`: Fetch from all remotes (not just origin)

**Features:**
- Parallel execution for speed
- Reports failures per submodule

### `gitfleet branch`

List branches across all submodules.

```bash
gitfleet branch
gitfleet branch --all
```

**Options:**
- `-a, --all`: Show all branches including remotes

**Features:**
- Highlights current branch
- Shows branch consistency across submodules

### `gitfleet init`

Initialize all submodules.

```bash
gitfleet init
gitfleet init --recursive
```

**Options:**
- `-r, --recursive`: Recursively initialize nested submodules

## Global Options

All commands support these global options:

- `-v, --verbose`: Enable verbose output
- `-n, --dry-run`: Show what would happen without executing
- `-h, --help`: Show help information
- `--version`: Show version information

## Common Workflows

### Setting Up a New Project Clone

```bash
# Clone your superproject
git clone https://github.com/your/project.git
cd project

# Initialize and sync all submodules
gitfleet init
gitfleet sync
```

### Starting a New Feature

```bash
# Create feature branch across all submodules
gitfleet create feature-new-ui --from main

# Check status
gitfleet status

# Make your changes...

# Commit and push
gitfleet commit -m "Add new UI components" --push
```

### Daily Development

```bash
# Start of day: sync with team
gitfleet pull

# Check what changed
gitfleet status

# Switch to feature branch
gitfleet checkout feature-analytics

# Make changes, commit frequently
gitfleet commit -m "WIP: analytics dashboard"

# End of day: push your work
gitfleet push
```

### Keeping Branches in Sync

```bash
# Update .gitmodules to specify branches
# [submodule "pdfczar"]
#   path = pdfczar
#   url = https://github.com/user/pdfczar.git
#   branch = dev

# Sync all submodules to their declared branches
gitfleet sync --init
```

### Code Review Workflow

```bash
# Fetch latest from all remotes
gitfleet fetch

# Check out review branch
gitfleet checkout review/pr-123

# Review, make changes if needed
gitfleet commit -m "Address review comments"

# Push changes
gitfleet push
```

## Configuration

### .gitmodules Branch Fields

SuperGit uses the `branch` field in `.gitmodules` to determine which branch each submodule should track:

```ini
[submodule "api"]
    path = api
    url = https://github.com/your/api.git
    branch = main

[submodule "frontend"]
    path = frontend
    url = https://github.com/your/frontend.git
    branch = develop
```

Use `gitfleet sync` to checkout and pull the specified branches.

## Edge Cases and Error Handling

SuperGit handles many common edge cases gracefully:

### Uncommitted Changes
- Detects dirty working directories
- Prompts before destructive operations
- Suggests stashing changes

### Detached HEAD
- Warns when detected
- Skips operations that require a branch
- Offers to create branch from current commit

### Branch Doesn't Exist
- Reports which submodules are missing the branch
- Offers to create it with `--create` flag
- Can create from a base branch

### Merge Conflicts
- Stops on first conflict
- Provides clear instructions
- Doesn't proceed until resolved

### Network Failures
- Reports which operations failed
- Continues with successful submodules
- Provides recovery commands

### Mixed States
- Shows clear status of each submodule
- Highlights inconsistencies
- Provides actionable suggestions

## Safety Features

1. **Dry-Run Mode**: Test operations without making changes
   ```bash
   gitfleet checkout feature-x --dry-run
   ```

2. **Interactive Prompts**: Confirmation for destructive operations
   ```bash
   gitfleet delete old-branch --remote
   # Prompts: "Are you sure you want to delete 'old-branch' from remote?"
   ```

3. **Force Flags Required**: Dangerous operations need explicit flags
   ```bash
   gitfleet push --force-with-lease
   ```

4. **Clear Error Messages**: Every error includes suggested fixes
   ```
   ✗ Error: Branch 'feature-x' does not exist in 'api'. Use --create to create it.
   ```

## Troubleshooting

### "No submodules found in .gitmodules"

Ensure you're running gitfleet from the repository root and that `.gitmodules` exists.

### "Submodule has uncommitted changes"

Commit or stash your changes before switching branches:
```bash
git stash  # in the submodule
gitfleet checkout other-branch
```

### "Failed to push in '<submodule>': authentication failed"

Ensure your SSH keys or credentials are set up correctly:
```bash
ssh -T git@github.com  # Test GitHub SSH
```

### Mixed branch states

Use `gitfleet status` to see which submodules are on different branches, then:
```bash
gitfleet checkout main  # Align all to main
```

## Performance

- Fetch and status operations execute in parallel
- Uses libgit2 for repository access
- Operations on 10 submodules typically complete in under 5 seconds

## License

MIT

## Related Projects

- [git-subrepo](https://github.com/ingydotnet/git-subrepo)
- [git-subtree](https://github.com/git/git/tree/master/contrib/subtree)
- [gitslave](http://gitslave.sourceforge.net/)

## Rationale

Standard Git submodule commands operate on one submodule at a time. For projects with multiple submodules, this requires repeating operations for each submodule. SuperGit automates this by executing commands across all submodules in a single invocation.
