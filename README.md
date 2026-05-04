# jjit - AI-powered jj Version Management

[![Crates.io](https://img.shields.io/crates/v/jjit)](https://crates.io/crates/jjit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English](README.md) | [中文](README.zh.md)

jjit is a command-line tool that brings AI assistance to [jj](https://jj-vcs.github.io/jj/) version control. It uses the DeepSeek API to understand your codebase changes and automatically generate commit messages, navigate commit history, and organize your commits.

## Table of Contents

- [Features](#features)
- [Requirements](#requirements)
- [Installation](#installation)
- [Quick Start](#quick-start)
- [Commands](#commands)
- [Global Options](#global-options)
- [Configuration](#configuration)
- [Architecture](#architecture)
- [Development](#development)
- [License](#license)

## Features

### Automatic Commit Messages

Generate conventional commit messages from your working copy changes. The AI analyzes diffs and produces structured commit descriptions following the Conventional Commits specification.

### Intelligent Commit Navigation

Find and checkout commits using natural language descriptions. Instead of memorizing commit hashes, describe what you are looking for in plain language.

### Smart Commit Packing

Combine multiple related commits into a single well-described commit. Useful for cleaning up your history before sharing work.

### Full Observability

- View the LLM's thinking process in real-time
- Inspect prompts sent to the AI
- See detailed debug information about API calls and jj command execution

### Multi-language Support

Configure the output language for AI-generated content. Default is `auto`, which automatically detects the language from the `LANG` environment variable, with support for any language the LLM understands.

## Requirements

- Rust 1.91 or later
- jj (Jujutsu) version control system installed
- DeepSeek API key

## Installation

### One-line Install (Recommended)

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/LaneSun/jjit/main/install.sh | sh
```

This automatically detects your platform, downloads the latest binary from GitHub releases, and installs it.

**Supported platforms:**
- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)

The script will install to `/usr/local/bin` (if you have write permission) or `~/.local/bin` (otherwise).

### Using Cargo

```bash
cargo install jjit
```

Or from source:

```bash
cargo install --path .
```

This installs to `~/.cargo/bin`. Ensure this directory is in your PATH.

### Using Make

```bash
make install
```

### Manual Build

```bash
cargo build --release
cp target/release/jjit ~/.local/bin/  # or any directory in PATH
```

## Quick Start

1. Set up your API key:

```bash
jjit config set api_key sk-your-api-key-here
```

Or use the environment variable:

```bash
export DEEPSEEK_API_KEY=sk-your-api-key-here
```

2. Try auto-commit:

```bash
# Make some changes to your code
jjit commit              # Generate and apply commit message
jjit commit --dry-run    # Preview without applying
```

## Commands

### `jjit commit`

Analyzes working copy changes and generates a commit message using AI.

```bash
jjit commit              # Create commit with AI-generated message
jjit commit --dry-run    # Preview the message without committing
```

The commit message follows Conventional Commits format (e.g., `feat:`, `fix:`, `refactor:`). The AI examines your diff to determine the appropriate type and scope.

### `jjit goto`

Finds and checks out a commit based on natural language description.

```bash
jjit goto "initial commit"
jjit goto "the commit that added user authentication"
jjit goto "before the major refactoring"
jjit goto "latest" --dry-run    # Preview without checking out
```

The AI searches through your commit history to find the best match for your description.

### `jjit pack`

Combines multiple commits into a single commit with a unified message.

```bash
jjit pack "all commits"                    # Combine all non-empty commits
jjit pack "today's commits"                # Combine commits from today
jjit pack "last 3 commits"                 # Combine the 3 most recent commits
jjit pack "commits related to config"      # Combine commits matching description
```

The AI determines which commits to combine and generates an appropriate unified commit message. After packing, a new empty working copy is created on top of the combined commit.

### `jjit config`

Manages configuration settings.

```bash
jjit config set api_key sk-your-key        # Set API key (project-level)
jjit config set api_key sk-your-key --global  # Set API key (user-level)
jjit config get api_key                    # Read a value
jjit config list                           # List all settings
```

Configuration is stored in:
- User-level: `~/.config/jjit/config.toml`
- Project-level: `.jjit/config.toml`

## Global Options

These options work with any command:

| Option | Description |
|--------|-------------|
| `--verbose` | Enable verbose output |
| `--show-prompt` | Display prompts sent to the LLM |
| `--debug` | Show detailed debug information |
| `--no-thinking` | Hide the LLM thinking process |
| `--no-color` | Disable colored output |

Examples:

```bash
jjit --show-prompt commit     # See the full system and user prompts
jjit --debug goto "latest"    # See API request details and jj commands
jjit --no-thinking pack "all" # Hide reasoning output
```

## Configuration

### Available Settings

| Setting | Description | Default |
|---------|-------------|---------|
| `api_key` | DeepSeek API key | Required |
| `model` | LLM model to use | `deepseek-v4-flash` |
| `base_url` | API endpoint URL | `https://api.deepseek.com` |
| `show_thinking` | Display AI reasoning | `true` |
| `language` | Output language for AI content | `auto` |
| `verbose` | Verbose output | `false` |
| `debug` | Debug mode | `false` |

### Configuration Precedence

Settings are resolved in this order (later overrides earlier):

1. Default values
2. User-level config (`~/.config/jjit/config.toml`)
3. Project-level config (`.jjit/config.toml`)
4. Environment variables (`DEEPSEEK_API_KEY`)
5. Command-line flags

### Language Configuration

Set the language for AI-generated content. The default is `auto`, which automatically detects the language from the `LANG` environment variable:

```bash
jjit config set language auto  # Auto-detect from LANG environment variable (default)
jjit config set language zh    # Chinese
jjit config set language en    # English
jjit config set language ja    # Japanese
```

When set to `auto`, jjit reads the `LANG` environment variable (e.g., `zh_CN.UTF-8` -> `zh`, `en_US.UTF-8` -> `en`) and falls back to `zh` if detection fails.

## Architecture

jjit consists of several components working together:

### CLI Layer (`src/cli.rs`)

Uses clap for command-line argument parsing. Defines the structure for `commit`, `goto`, `pack`, and `config` commands along with global flags.

### Command Handlers (`src/commands/`)

Each command has its own module:
- `commit.rs`: Analyzes diffs and creates commits
- `goto.rs`: Searches history and checks out commits
- `pack.rs`: Combines multiple commits
- `config.rs`: Manages settings

### LLM Client (`src/llm.rs`)

Handles communication with the DeepSeek API:
- Streaming response processing with real-time thinking display
- Automatic retry with exponential backoff (5s -> 30s -> 5min)
- Configurable thinking mode and language hints
- Full observability via `--show-prompt` and `--debug`

### XML Parser (`src/output.rs`)

Parses structured LLM responses using quick-xml:
- Supports `<commit>`, `<goto>`, `<pack>`, and `<reply>` tags
- Handles nested content and malformed XML gracefully
- Extracts structured data for command execution

### jj Utilities (`src/jj_util.rs`)

Wraps jj commands via the vcs-runner crate:
- Custom log templates with full commit IDs for reliable resolution
- Diff and status querying
- Commit creation, checkout, and squashing operations

### Configuration (`src/config.rs`)

Manages layered configuration:
- User-level and project-level TOML files
- Environment variable overrides
- Default values

## Development

### Building

```bash
cargo build --release
```

### Testing

```bash
cargo test
```

### Key Design Decisions

- **LLM returns structured data only**: The AI provides XML-structured data; the application constructs and executes all jj commands
- **Strict XML schema**: Each command has a defined XML format (`<commit>`, `<goto>`, `<pack>`, `<reply>`)
- **Full commit IDs**: Log templates use full 40-character commit IDs to ensure reliable revset resolution
- **No interactive confirmation**: jj provides undo capabilities, so commands execute without prompts
- **Streaming output**: LLM thinking process is displayed in real-time using gray ANSI color

## License

MIT License. See [LICENSE](LICENSE) for details.
