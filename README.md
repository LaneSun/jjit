# jjit - AI-powered jj version management

jjit is a command-line tool that automates version control workflows using [jj](https://jj-vcs.github.io/jj/) and LLM (via DeepSeek API).

## Features

- **Auto-commit**: Generate commit messages from diffs using AI
- **Smart checkout**: Find and checkout commits by natural language description
- **Pack commits**: Combine multiple commits with AI-generated descriptions
- **Full observability**: Debug prompts, thinking process, and command execution

## Prerequisites

- [Rust](https://rustup.rs/) (1.75+)
- [jj](https://jj-vcs.github.io/jj/latest/install-and-setup/) installed
- DeepSeek API key

## Installation

### Method 1: Using install script (Recommended)

```bash
git clone https://github.com/yourusername/jjit
cd jjit
./install.sh
```

### Method 2: Using cargo

```bash
cargo install --path .
```

This installs to `~/.cargo/bin`. Make sure it's in your PATH.

### Method 3: Using Make

```bash
make install
```

### Method 4: Manual installation

```bash
cargo build --release
cp target/release/jjit ~/.local/bin/  # or any directory in PATH
```

## Configuration

Set your DeepSeek API key:

```bash
jjit config set api_key <your-api-key>
```

Or use environment variable:

```bash
export DEEPSEEK_API_KEY=sk-...
```

## Usage

### Auto-commit

```bash
jjit commit              # Generate and apply commit message
jjit commit --dry-run    # Preview without applying
```

### Smart checkout

```bash
jjit goto "initial commit"
jjit goto "the commit that added auth"
jjit goto "before the refactoring"
```

### Pack commits

```bash
jjit pack "all commits"
jjit pack "today's commits"
jjit pack "last 3 commits"
```

### Debug mode

```bash
jjit --show-prompt commit    # Show prompts sent to LLM
jjit --debug commit          # Show detailed execution info
jjit --no-thinking commit    # Hide LLM thinking process
jjit --no-color commit       # Disable colored output
```

## Configuration Options

| Option | Description | Default |
|--------|-------------|---------|
| `api_key` | DeepSeek API key | - |
| `model` | LLM model | `deepseek-v4-flash` |
| `base_url` | API base URL | `https://api.deepseek.com` |
| `show_thinking` | Show LLM thinking process | `true` |
| `verbose` | Verbose output | `false` |
| `debug` | Debug mode | `false` |

Set globally:
```bash
jjit config set show_thinking false --global
```

## Project Structure

```
jjit/
├── src/              # Source code
├── tests/            # Tests
├── docs/             # Reference documents
├── AGENTS.md         # Development guidelines
├── PLAN.md           # Implementation plan
├── TURN1-8.md        # Iteration reports
├── install.sh        # Install script
└── Makefile          # Build automation
```

## Development

See [AGENTS.md](AGENTS.md) for development guidelines and workflow.

## License

MIT OR Apache-2.0
