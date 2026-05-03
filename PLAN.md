# jjit 实现计划

## 项目概述

基于 jj 的版本管理 CLI 工具，通过 vcs-runner 检查 jj 状态，通过 DeepSeek API 进行自动化处理。

## 技术栈

- **语言**: Rust (edition 2021)
- **CLI 解析**: clap 4.5 (derive 特性)
- **HTTP 客户端**: reqwest 0.12 (json 特性)
- **序列化**: serde + serde_json + toml
- **XML 解析**: quick-xml 0.36 (serialize 特性)
- **异步运行时**: tokio 1.40 (full 特性)
- **目录管理**: directories 5.0
- **时间处理**: chrono 0.4
- **错误处理**: anyhow 1.0
- **VCS 操作**: vcs-runner 0.12.1

## 项目结构

```
src/
├── main.rs          # 程序入口、命令分发、重试逻辑
├── cli.rs           # clap derive 命令定义
├── config.rs        # 配置管理（用户级/项目级/环境变量）
├── llm.rs           # DeepSeek API 封装
├── output.rs        # XML 解析、格式化输出
├── jj_util.rs       # jj 操作封装（基于 vcs-runner）
└── commands/
    ├── commit.rs    # commit 命令实现
    ├── goto.rs      # goto 命令实现
    ├── pack.rs      # pack 命令实现
    └── config.rs    # config set/get 命令实现
```

## 命令清单

### 已实现

- `jjit commit` - 检查变动，调用 LLM 生成描述并提交
- `jjit goto <要求>` - 根据要求调用 LLM 获取目标 commit 并检出
- `jjit pack <要求>` - 根据要求调用 LLM 获得范围并组合 commits
- `jjit config set <key> <value>` - 设置配置
- `jjit config get <key>` - 获取配置

### 暂不实现

- `jjit split [@-N] [要求]` - 拆分提交
- `jjit push` - 提交后自动整理
- `jjit do <要求>` - Agent 形式自动化处理
- `jjit select` - 交互式选择 commits

## 配置系统

### 配置优先级（从高到低）

1. 环境变量 `DEEPSEEK_API_KEY`
2. 项目级配置 `.jjit/config.toml`
3. 用户级配置 `~/.config/jjit/config.toml`
4. 无配置时首次命令会提示输入并持久化

### 配置项

- `api_key` - DeepSeek API Key
- `model` - LLM 模型（默认 `deepseek-v4-flash`）
- `base_url` - API 基础地址（默认 `https://api.deepseek.com`）
- `verbose` - 是否显示详细输出（默认 `false`）

### 配置命令

```bash
jjit config set api_key <key>        # 默认项目级
jjit config set api_key <key> --global   # 用户级
jjit config get api_key
jjit config list
```

## LLM 输出规范（XML Schema）

### 通用结构

所有类型根标签区分，包含 `<summary>` 字段。

### commit → `<commit>`

```xml
<commit>
  <summary>检测到 3 个文件变更，按约定式提交规范生成</summary>
  <message>feat(auth): 添加用户登录验证模块</message>
  <body>新增用户名密码验证逻辑</body>
</commit>
```

**必填字段**: `summary`, `message`
**可选字段**: `body`

### goto → `<goto>`

```xml
<goto>
  <summary>找到修改 src/main.rs 前的版本</summary>
  <target>abc123def456</target>
</goto>
```

**必填字段**: `summary`, `target`

### pack → `<pack>`

```xml
<pack>
  <summary>组合今天 3 个提交为 1 个功能提交</summary>
  <range>abc123:def456</range>
  <message>feat: 实现完整的用户认证流程</message>
  <body>包含登录、注册、密码重置三个功能</body>
</pack>
```

**必填字段**: `summary`, `range`, `message`
**可选字段**: `body`

### reply → `<reply>`（LLM 拒绝执行）

```xml
<reply>
  <message>当前工作副本没有未提交的变更</message>
</reply>
```

**必填字段**: `message`

## 容错策略

### XML 解析容错

- 标签大小写不敏感（统一转小写处理）
- 未知标签：忽略
- 多余文本/注释：忽略
- 必填字段缺失：提取已有信息，缺失部分触发重试
- XML 解析失败：尝试正则提取关键字段，仍失败则按重试策略处理

### 重试策略

指数退避重试：5s → 30s → 5min（共 3 次重试）
仅对以下错误重试：
- 网络超时
- HTTP 429（Rate Limit）
- API 服务端错误（5xx）
- XML 解析失败

重试全部失败后程序退出并报错。

## 各命令实现流程

### commit 命令

1. 执行 `jj status` 检查是否有变更
2. 执行 `jj diff --summary` 获取变更文件列表
3. 执行 `jj diff` 获取完整 diff
4. 构建 prompt，调用 LLM
5. 解析 XML 响应
6. 如为 `<commit>`，执行 `jj describe -m "message"`（含 body 则 `jj describe -m "message\n\nbody"`）
7. 如为 `<reply>`，输出 message 并退出

### goto 命令

1. 执行 `jj log -r 'all()'` 获取完整提交历史
2. 构建 prompt（包含用户要求和历史记录），调用 LLM
3. 解析 XML 响应
4. 如为 `<goto>`，执行 `jj new <target>`
5. 如为 `<reply>`，输出 message 并退出

### pack 命令

1. 执行 `jj log` 获取提交历史
2. 构建 prompt（包含用户要求和历史记录），调用 LLM
3. 解析 XML 响应
4. 如为 `<pack>`，执行 `jj squash -r <range> -m "message"`
5. 如为 `<reply>`，输出 message 并退出

### config 命令

1. `set`: 写入对应配置文件（用户级或项目级）
2. `get`: 按优先级读取配置
3. `list`: 列出所有配置项

## Prompt 设计原则

1. 每个命令有独立的 system prompt
2. system prompt 说明 XML Schema 格式要求
3. user prompt 包含 jj 命令输出（diff、log 等）和用户要求
4. 要求 LLM 严格按 XML 格式返回

## 输出风格

- 默认简洁：只输出 `<summary>` 或 `<message>`
- verbose 模式：输出完整交互信息
- 可通过 `--verbose` 参数或 `verbose` 配置项控制

## 安全考虑

- API Key 不硬编码，通过配置或环境变量获取
- 不在日志中打印 API Key
- 使用标准目录存储配置（directories crate）

## 示例用法

```bash
# 提交
jjit commit

# 检出
jjit goto 上次 src/main.rs 被修改前的版本

# 组合
jjit pack 今天所有的提交

# 配置
jjit config set api_key sk-xxx
jjit config get api_key
```
