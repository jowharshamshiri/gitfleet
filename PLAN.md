# SuperGit - Comprehensive Plan

## Overview
SuperGit is a Rust CLI tool that treats Git submodules as synchronized "superbranches" - allowing unified branch management, commits, and operations across all submodules at once.

## Core Concept
- Maintain consistent branch names across all submodules
- Drive submodule state from `.gitmodules` branch declarations
- Provide simple commands to operate on all submodules simultaneously
- Handle edge cases gracefully with clear user feedback

## Commands

### 1. `supergit status`
Shows status of all submodules in a unified view.
- Current branch for each submodule
- Uncommitted changes
- Ahead/behind tracking branch
- Detached HEAD warnings
- Uninitialized submodules

### 2. `supergit checkout <branch> [--create]`
Checkout the same branch across all submodules.
- Checks if branch exists in each submodule
- Creates branch if `--create` flag is used
- Handles dirty working directories (stash prompt)
- Falls back gracefully if branch missing
- Updates superproject to track new branch

### 3. `supergit pull [--rebase]`
Pull latest changes for current branch in all submodules.
- Fetches and pulls each submodule
- Reports conflicts clearly
- Option to rebase instead of merge
- Skips submodules with uncommitted changes (with warning)

### 4. `supergit push [--force-with-lease]`
Push current branch for all submodules.
- Pushes each submodule's current branch
- Handles authentication
- Reports success/failure per submodule
- Supports force-with-lease for safety

### 5. `supergit create <branch> [--from <base-branch>]`
Create new branch across all submodules.
- Creates branch from current HEAD or specified base
- Sets up tracking to origin
- Updates .gitmodules branch field
- Can push new branch immediately with `--push`

### 6. `supergit commit -m "message" [--push]`
Commit changes across all submodules.
- Commits only submodules with changes
- Uses same message for all
- Updates superproject to reference new commits
- Optionally pushes immediately

### 7. `supergit sync`
Synchronize all submodules based on .gitmodules.
- Reads branch field from .gitmodules for each submodule
- Fetches all remotes
- Checks out specified branch
- Pulls latest changes
- Initializes missing submodules

### 8. `supergit delete <branch> [--remote]`
Delete branch from all submodules.
- Deletes local branch
- Optionally deletes from remote with `--remote`
- Safety check: prevents deleting current branch
- Confirms before remote deletion

### 9. `supergit fetch [--all]`
Fetch updates for all submodules.
- Fetches from origin by default
- `--all` fetches from all remotes
- Parallel execution for speed

### 10. `supergit branch [--all]`
List branches across all submodules.
- Shows current branch for each submodule
- `--all` shows all branches including remotes
- Highlights inconsistencies

### 11. `supergit init`
Initialize all submodules recursively.
- Clones missing submodules
- Updates existing ones
- Checkouts branch from .gitmodules if specified

## Edge Cases & Handling

### 1. **Submodule Doesn't Have Target Branch**
- Attempt to create branch from default branch (main/master)
- Prompt user for action
- Report which submodules failed
- Option to skip that submodule

### 2. **Uncommitted Changes**
- Detect dirty working directory
- Offer to stash changes
- Prevent destructive operations without user consent
- List affected files

### 3. **Detached HEAD State**
- Clear warning when detected
- Offer to create branch from current commit
- Prevent operations that would lose commits

### 4. **Branch Doesn't Exist on Remote**
- For push: offer to create remote branch
- For pull: skip with warning
- For checkout: offer to create from local branch

### 5. **Merge Conflicts**
- Stop on first conflict
- Provide clear instructions
- Don't proceed to other submodules until resolved
- Option to abort and reset

### 6. **Network Failures**
- Retry with exponential backoff
- Report which operations failed
- Continue with successful submodules
- Provide recovery commands

### 7. **Permission Issues**
- Check SSH keys / credentials before batch operations
- Report authentication failures clearly
- Support credential helpers

### 8. **Nested Submodules**
- Recursively handle nested submodules
- `--recursive` flag for deep operations
- Clear hierarchy in output

### 9. **Uninitialized Submodules**
- Auto-initialize when needed
- Report what's being initialized
- Skip if `--no-init` flag used

### 10. **Diverged Branches**
- Detect local vs remote divergence
- Warn before potentially destructive operations
- Suggest rebase or merge options

## Architecture

### Module Structure
```
supergit/
├── src/
│   ├── main.rs              # CLI entry point
│   ├── cli.rs               # Command-line parsing (clap)
│   ├── git/
│   │   ├── mod.rs           # Git module
│   │   ├── operations.rs    # Git command wrappers
│   │   ├── submodule.rs     # Submodule-specific operations
│   │   └── status.rs        # Status parsing
│   ├── config/
│   │   ├── mod.rs           # Config module
│   │   └── gitmodules.rs    # .gitmodules parser
│   ├── commands/
│   │   ├── mod.rs           # Commands module
│   │   ├── status.rs        # Status command
│   │   ├── checkout.rs      # Checkout command
│   │   ├── pull.rs          # Pull command
│   │   ├── push.rs          # Push command
│   │   ├── create.rs        # Create command
│   │   ├── commit.rs        # Commit command
│   │   ├── sync.rs          # Sync command
│   │   ├── delete.rs        # Delete command
│   │   ├── fetch.rs         # Fetch command
│   │   ├── branch.rs        # Branch command
│   │   └── init.rs          # Init command
│   ├── output/
│   │   ├── mod.rs           # Output module
│   │   └── formatter.rs     # Colorized output
│   ├── error.rs             # Error types
│   └── utils.rs             # Utility functions
├── Cargo.toml
└── README.md
```

### Key Dependencies
- `clap` - CLI parsing with derive macros
- `git2` - Rust Git library (alternative to shell commands)
- `colored` - Colorized terminal output
- `anyhow` - Error handling
- `serde` - Configuration parsing
- `ini` - .gitmodules parsing
- `indicatif` - Progress bars
- `rayon` - Parallel operations
- `dialoguer` - Interactive prompts

## Features

### Safety Features
1. **Dry-run mode**: `--dry-run` flag shows what would happen
2. **Interactive mode**: Prompt before destructive operations
3. **Force flag required**: For potentially dangerous operations
4. **Backup suggestions**: Suggest stashing before risky operations
5. **Atomic operations**: Roll back on failure where possible

### User Experience
1. **Colorized output**: Green for success, red for errors, yellow for warnings
2. **Progress bars**: For long-running operations
3. **Clear error messages**: With suggested fixes
4. **Summary reports**: Show what succeeded/failed
5. **Verbose mode**: `-v` for detailed output

### Performance
1. **Parallel operations**: Fetch/pull multiple submodules concurrently
2. **Lazy loading**: Only parse what's needed
3. **Caching**: Cache .gitmodules parsing
4. **Efficient Git operations**: Use libgit2 when possible

## Testing Strategy

### Unit Tests
- .gitmodules parser
- Git command builders
- Error handling
- Status parsing

### Integration Tests
- Create test repository with multiple submodules
- Test each command with various scenarios
- Test edge cases (conflicts, missing branches, etc.)
- Test nested submodules

### Manual Test Scenarios
1. Fresh clone with multiple submodules
2. Switching between branches
3. Creating new feature branches across all
4. Handling merge conflicts
5. Network failure recovery
6. Permission denied scenarios
7. Detached HEAD recovery
8. Mixed state submodules

## Configuration

### .supergit.toml (optional)
```toml
[defaults]
parallel = true
max_parallel = 4
auto_stash = false
confirm_destructive = true

[sync]
auto_init = true
recursive = true

[output]
color = true
progress = true
verbose = false
```

## Documentation

### README.md Sections
1. Installation
2. Quick Start
3. Commands Reference
4. Common Workflows
5. Troubleshooting
6. Contributing

### Examples
- Setting up superbranches for the first time
- Daily workflow (create, commit, push)
- Switching between feature branches
- Syncing team changes
- Handling conflicts

## Success Criteria
✅ Can manage 10+ submodules efficiently
✅ Handles all common Git workflows
✅ Clear error messages with recovery steps
✅ Comprehensive test coverage (>80%)
✅ Works on Linux, macOS, Windows
✅ Performance: <5s for operations on 10 submodules
✅ Zero data loss scenarios
✅ Production-ready error handling
