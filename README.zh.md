# jjit - AI 驱动的 jj 版本管理工具

[![Crates.io](https://img.shields.io/crates/v/jjit)](https://crates.io/crates/jjit)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](https://opensource.org/licenses/MIT)

[English](README.md) | [中文](README.zh.md)

jjit 是一款命令行工具，为 [jj](https://jj-vcs.github.io/jj/) 版本控制系统提供 AI 辅助功能。它通过 DeepSeek API 理解代码变更，自动生成提交信息、导航提交历史、整理提交记录。

## 目录

- [功能特性](#功能特性)
- [环境要求](#环境要求)
- [安装](#安装)
- [快速开始](#快速开始)
- [命令说明](#命令说明)
- [全局选项](#全局选项)
- [配置](#配置)
- [架构设计](#架构设计)
- [开发指南](#开发指南)
- [许可证](#许可证)

## 功能特性

### 自动生成提交信息

根据工作区变更自动生成符合 [约定式提交](https://www.conventionalcommits.org/) 规范的提交信息。AI 分析 diff 内容，判断变更类型和范围。

### 智能提交导航

使用自然语言描述查找并检出任一提交。无需记忆提交哈希，用日常语言描述你要找的提交即可。

### 智能提交合并

将多个相关提交合并为一个描述清晰的提交。适合在分享工作前整理提交历史。

### 完整可观测性

- 实时查看 AI 的思考过程（灰色显示）
- 查看发送给 AI 的完整提示词
- 查看 API 调用和 jj 命令执行的详细调试信息

### 多语言支持

支持配置 AI 输出的语言。默认值为 `auto`，自动从 `LANG` 环境变量检测语言，支持任何 LLM 能理解的语言。

## 环境要求

- Rust 1.91 或更高版本
- 已安装 [jj](https://jj-vcs.github.io/jj/latest/install-and-setup/) (Jujutsu)
- DeepSeek API 密钥

## 安装

### 方式一：一键安装（推荐）

```bash
curl --proto '=https' --tlsv1.2 -sSf https://raw.githubusercontent.com/LaneSun/jjit/main/install.sh | sh
```

脚本会自动检测你的平台，从 GitHub releases 下载最新二进制文件并安装。

**支持的平台：**
- Linux (x86_64, aarch64)
- macOS (Intel, Apple Silicon)

脚本会优先安装到 `/usr/local/bin`（如果你有写入权限），否则安装到 `~/.local/bin`。

### 方式二：使用 Cargo

```bash
cargo install jjit
```

或从源码安装：

```bash
cargo install --path .
```

这会将 jjit 安装到 `~/.cargo/bin`，请确保该目录在 PATH 中。

### 方式三：使用 Make

```bash
make install
```

### 方式四：手动构建

```bash
cargo build --release
cp target/release/jjit ~/.local/bin/  # 或 PATH 中的任意目录
```

## 快速开始

1. 配置 API 密钥：

```bash
jjit config set api_key sk-your-api-key-here
```

或通过环境变量：

```bash
export DEEPSEEK_API_KEY=sk-your-api-key-here
```

2. 试用自动提交：

```bash
# 修改一些代码
jjit commit              # 生成并应用提交信息
jjit commit --dry-run    # 预览而不应用
```

## 命令说明

### `jjit commit`

分析工作区变更，使用 AI 生成提交信息。

```bash
jjit commit              # 创建带有 AI 生成信息的提交
jjit commit --dry-run    # 预览提交信息而不提交
```

提交信息遵循约定式提交格式（如 `feat:`、`fix:`、`refactor:`）。AI 会检查 diff 来确定合适的类型和范围。

### `jjit goto`

基于自然语言描述查找并检出提交。

```bash
jjit goto "初始提交"
jjit goto "添加了用户认证的那个提交"
jjit goto "重构之前的提交"
jjit goto "最新" --dry-run    # 预览而不检出
```

AI 会搜索提交历史，找到与你描述最匹配的提交。

### `jjit pack`

将多个提交合并为一个带有统一信息的提交。

```bash
jjit pack "所有提交"                    # 合并所有非空提交
jjit pack "今天的提交"                  # 合并今天的提交
jjit pack "最近 3 个提交"                # 合并最近 3 个提交
jjit pack "与配置相关的提交"             # 合并描述匹配的提交
```

AI 会确定要合并哪些提交，并生成适当的统一提交信息。合并后会在合并提交上方创建一个新的空工作区提交。

### `jjit config`

管理配置设置。

```bash
jjit config set api_key sk-your-key        # 设置 API 密钥（项目级）
jjit config set api_key sk-your-key --global  # 设置 API 密钥（用户级）
jjit config get api_key                    # 读取配置值
jjit config list                           # 列出所有设置
```

配置文件存储位置：
- 用户级：`~/.config/jjit/config.toml`
- 项目级：`.jjit/config.toml`

## 全局选项

这些选项适用于所有命令：

| 选项 | 说明 |
|------|------|
| `--verbose` | 启用详细输出 |
| `--show-prompt` | 显示发送给 LLM 的提示词 |
| `--debug` | 显示详细调试信息 |
| `--no-thinking` | 隐藏 AI 思考过程 |
| `--no-color` | 禁用彩色输出 |

示例：

```bash
jjit --show-prompt commit     # 查看完整的系统和用户提示词
jjit --debug goto "最新"      # 查看 API 请求详情和 jj 命令
jjit --no-thinking pack "所有" # 隐藏推理输出
```

## 配置

### 可用设置

| 设置 | 说明 | 默认值 |
|------|------|--------|
| `api_key` | DeepSeek API 密钥 | 必填 |
| `model` | 使用的 LLM 模型 | `deepseek-v4-flash` |
| `base_url` | API 端点地址 | `https://api.deepseek.com` |
| `show_thinking` | 显示 AI 推理过程 | `true` |
| `language` | AI 内容输出语言 | `auto` |
| `verbose` | 详细输出 | `false` |
| `debug` | 调试模式 | `false` |

### 配置优先级

设置按以下顺序解析（后面的覆盖前面的）：

1. 默认值
2. 用户级配置（`~/.config/jjit/config.toml`）
3. 项目级配置（`.jjit/config.toml`）
4. 环境变量（`DEEPSEEK_API_KEY`）
5. 命令行参数

### 语言配置

设置 AI 生成内容的语言。默认值为 `auto`，自动从 `LANG` 环境变量检测语言：

```bash
jjit config set language auto  # 从 LANG 环境变量自动检测（默认）
jjit config set language zh    # 中文
jjit config set language en    # 英文
jjit config set language ja    # 日文
```

设置为 `auto` 时，jjit 会读取 `LANG` 环境变量（例如 `zh_CN.UTF-8` -> `zh`，`en_US.UTF-8` -> `en`），如果检测失败则回退到 `zh`。

## 架构设计

jjit 由多个组件协同工作：

### CLI 层（`src/cli.rs`）

使用 clap 进行命令行参数解析。定义了 `commit`、`goto`、`pack` 和 `config` 命令的结构以及全局标志。

### 命令处理器（`src/commands/`）

每个命令有独立的模块：
- `commit.rs`：分析 diff 并创建提交
- `goto.rs`：搜索历史并检出提交
- `pack.rs`：合并多个提交
- `config.rs`：管理设置

### LLM 客户端（`src/llm.rs`）

处理与 DeepSeek API 的通信：
- 流式响应处理，实时显示思考过程
- 自动重试，指数退避（5秒 -> 30秒 -> 5分钟）
- 可配置的思考模式和语言提示
- 通过 `--show-prompt` 和 `--debug` 实现完整可观测性

### XML 解析器（`src/output.rs`）

使用 quick-xml 解析结构化的 LLM 响应：
- 支持 `<commit>`、`<goto>`、`<pack>` 和 `<reply>` 标签
- 优雅处理嵌套内容和格式错误的 XML
- 提取结构化数据用于命令执行

### jj 工具封装（`src/jj_util.rs`）

通过 vcs-runner 库包装 jj 命令：
- 自定义日志模板，使用完整提交 ID 以确保可靠解析
- diff 和状态查询
- 提交创建、检出和压缩操作

### 配置管理（`src/config.rs`）

管理分层配置：
- 用户级和项目级 TOML 文件
- 环境变量覆盖
- 默认值

## 开发指南

### 构建

```bash
cargo build --release
```

### 测试

```bash
cargo test
```

### 关键设计决策

- **LLM 只返回结构化数据**：AI 提供 XML 结构化数据；应用程序构建并执行所有 jj 命令
- **严格的 XML 模式**：每个命令都有定义的 XML 格式（`<commit>`、`<goto>`、`<pack>`、`<reply>`）
- **完整提交 ID**：日志模板使用完整的 40 字符提交 ID 以确保可靠的 revset 解析
- **无交互式确认**：jj 提供撤销功能，因此命令无需提示直接执行
- **流式输出**：LLM 思考过程使用灰色 ANSI 颜色实时显示

## 许可证

MIT 许可证。详见 [LICENSE](LICENSE)。
