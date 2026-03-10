# matrix-bridge-telegram

[![CI](https://github.com/palpo-im/matrix-bridge-telegram/actions/workflows/ci.yml/badge.svg)](https://github.com/palpo-im/matrix-bridge-telegram/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

一个使用 Rust 编写的高性能 Matrix-Telegram 桥接服务。通过 Matrix Application Service API 和 Telegram Bot API，实现 [Matrix](https://matrix.org/) 房间与 [Telegram](https://telegram.org/) 聊天之间的双向消息互通。

## 功能特性

- **双向消息传递** -- 文本消息在两个平台之间实时转发
- **媒体支持** -- 图片、视频、音频、文件、语音消息、视频留言、动画（GIF）、贴纸、联系人和位置信息
- **消息格式** -- 跨平台保留 Markdown 和 HTML 格式
- **用户映射（Puppeting）** -- Telegram 用户在 Matrix 侧显示为独立的虚拟用户，包含显示名称和头像
- **Portal 房间** -- Telegram 聊天自动映射为 Matrix 房间
- **Bot 命令** -- Matrix 侧支持 `!tg` 命令，Telegram 侧支持 `/` 命令
- **房间管理** -- 桥接、取消桥接、同步成员、列出房间
- **加入/退出通知** -- 成员变动在两个平台之间同步
- **聊天标题与话题同步** -- 标题变更跨平台传播
- **消息置顶** -- Telegram 置顶消息转发至 Matrix
- **已读回执** -- 可配置的已读回执转发
- **在线状态** -- 可配置的在线状态同步
- **加密** -- 可选的端到桥加密支持
- **多数据库后端** -- 支持 PostgreSQL、SQLite 和 MySQL
- **监控指标** -- 可选的 Prometheus 兼容指标端点
- **Docker 支持** -- 提供 Dockerfile 和 docker-compose 配置
- **CLI 工具** -- 注册文件生成、配置验证、房间管理和数据库迁移

## 环境要求

- **Rust 1.93+**（从源码构建时需要）
- **PostgreSQL 14+**、**SQLite 3** 或 **MySQL 8+**（数据库后端）
- **Telegram Bot Token**（从 [@BotFather](https://t.me/BotFather) 获取）
- **Telegram API 凭证**（从 [my.telegram.org](https://my.telegram.org/) 获取 `api_id` 和 `api_hash`）
- **Matrix 服务器**，需支持 Application Service（例如 [Synapse](https://github.com/element-hq/synapse)、[Conduit](https://conduit.rs/)、[Dendrite](https://github.com/matrix-org/dendrite)）

## Telegram 配置

在运行桥接服务之前，需要先获取 Telegram 凭证。

### 1. 获取 API 凭证（`api_id` 和 `api_hash`）

1. 访问 [https://my.telegram.org/](https://my.telegram.org/)，使用手机号登录
2. 点击 **"API development tools"**
3. 填写表单（应用名称和短名称随意填写，例如 "Matrix Bridge"）
4. 系统会提供 `api_id`（整数）和 `api_hash`（字符串）-- 保存这些信息用于配置

### 2. 通过 BotFather 创建 Bot

1. 打开 Telegram，搜索 [@BotFather](https://t.me/BotFather)
2. 发送 `/newbot`，按提示选择 Bot 名称和用户名
3. BotFather 会回复一个 **Bot Token**，格式如 `123456789:ABCdefGhIJKlmnOPQRstUVwxyz` -- 保存此 Token
4. **推荐的 Bot 设置**（向 BotFather 发送以下命令）：
   - `/setprivacy` -> 选择你的 Bot -> **Disable**（允许 Bot 看到群组中的所有消息，而不仅仅是命令）
   - `/setjoingroups` -> 选择你的 Bot -> **Enable**（允许 Bot 被添加到群组）
   - `/setcommands` -> 选择你的 Bot -> 发送以下内容：
     ```
     start - 显示欢迎消息
     help - 显示可用命令
     bridge - 显示当前聊天的桥接状态
     ```

### 3. 将 Bot 添加到 Telegram 群组

1. 打开要桥接的 Telegram 群组
2. 将 Bot 添加为群组成员（通过 Bot 用户名搜索）
3. **将 Bot 提升为管理员**（推荐）-- 这样 Bot 可以：
   - 查看所有消息（未关闭隐私模式时必需）
   - 删除消息（用于撤回转发）
   - 置顶消息
   - 修改群组信息

### 4. 获取 Chat ID

桥接房间时需要 Telegram 的 Chat ID，获取方法：

- **通过 Bot**：将 Bot 添加到群组后发送任意消息，桥接日志中会显示 `chat_id`。
- **使用 `/bridge` 命令**：在添加了 Bot 的 Telegram 群组中发送 `/bridge`，Bot 会显示 Chat ID。
- **通过 Telegram Web**：打开 [https://web.telegram.org/](https://web.telegram.org/)，进入群组，查看 URL。`#-` 后面的数字（群组带 `-` 前缀）即为 Chat ID（例如 URL `#-1001234567890` 表示 Chat ID 为 `-1001234567890`）。

### 5. 填写配置

将凭证添加到 `config.yaml`：

```yaml
auth:
  api_id: 12345678            # 来自步骤 1
  api_hash: "0123456789abcdef" # 来自步骤 1
  bot_token: "123456789:ABCdefGhIJKlmnOPQRstUVwxyz"  # 来自步骤 2
```

或使用环境变量：

```bash
export APPSERVICE_TELEGRAM_AUTH_API_ID=12345678
export APPSERVICE_TELEGRAM_AUTH_API_HASH=0123456789abcdef
export APPSERVICE_TELEGRAM_AUTH_BOT_TOKEN=123456789:ABCdefGhIJKlmnOPQRstUVwxyz
```

### 6. 桥接房间

桥接服务运行且 Bot 已加入 Telegram 群组后：

1. 在 Matrix 房间中发送：`!tg bridge -1001234567890`（替换为你的 Chat ID）
2. 或在 Telegram 群组中发送 `/bridge` 验证桥接是否生效

## 快速开始

### 从源码构建

```bash
# 克隆仓库
git clone https://github.com/palpo-im/matrix-bridge-telegram.git
cd matrix-bridge-telegram

# 使用 PostgreSQL 和 SQLite 支持构建（默认）
cargo build --release

# 或仅使用 PostgreSQL 构建
cargo build --release --no-default-features --features postgres

# 或仅使用 SQLite 构建
cargo build --release --no-default-features --features sqlite
```

构建完成后，二进制文件位于 `target/release/matrix-bridge-telegram`。

### Docker

```bash
# 构建 Docker 镜像
docker build -t matrix-bridge-telegram .

# 或使用 docker-compose（包含 PostgreSQL）
docker-compose up -d
```

使用 Docker 时，将配置目录挂载到 `/data`：

```bash
docker run -v ./config:/data -p 29317:29317 matrix-bridge-telegram
```

### 初始配置

1. **复制并编辑示例配置文件：**

   ```bash
   cp config/config.sample.yaml config.yaml
   # 编辑 config.yaml，填入你的配置
   ```

2. **为 Matrix 服务器生成注册文件：**

   ```bash
   ./matrix-bridge-telegram generate-registration \
     --id telegram \
     --homeserver-url http://localhost:8008 \
     --domain example.org \
     --output telegram-registration.yaml
   ```

3. **在 Matrix 服务器中注册 appservice。** 以 Synapse 为例，将注册文件路径添加到 `homeserver.yaml` 的 `app_service_config_files` 列表中，然后重启 Synapse。

4. **启动桥接服务：**

   ```bash
   ./matrix-bridge-telegram --config config.yaml
   ```

## 配置说明

桥接服务通过 YAML 文件进行配置。完整的带注释示例请参见 [`config/config.sample.yaml`](config/config.sample.yaml)。

配置文件也可以通过 `CONFIG_PATH` 环境变量指定路径加载。敏感信息支持环境变量覆盖：

| 环境变量 | 对应配置项 |
|---|---|
| `APPSERVICE_TELEGRAM_AUTH_API_ID` | `auth.api_id` |
| `APPSERVICE_TELEGRAM_AUTH_API_HASH` | `auth.api_hash` |
| `APPSERVICE_TELEGRAM_AUTH_BOT_TOKEN` | `auth.bot_token` |
| `APPSERVICE_TELEGRAM_REGISTRATION_ID` | `registration.bridge_id` |
| `APPSERVICE_TELEGRAM_REGISTRATION_AS_TOKEN` | `registration.appservice_token` |
| `APPSERVICE_TELEGRAM_REGISTRATION_HS_TOKEN` | `registration.homeserver_token` |
| `APPSERVICE_TELEGRAM_REGISTRATION_SENDER_LOCALPART` | `registration.sender_localpart` |

### 配置章节

| 章节 | 说明 |
|---|---|
| `bridge` | 核心桥接设置：域名、端口、服务器 URL、功能开关 |
| `registration` | Appservice 注册信息：令牌、命名空间、发送者本地部分 |
| `auth` | Telegram API 凭证：`api_id`、`api_hash`、`bot_token` |
| `logging` | 日志级别、格式和文件输出设置 |
| `database` | 数据库连接 URL 和连接池设置 |
| `room` | 房间创建和可见性默认值 |
| `portal` | 虚拟用户的用户名/显示名模板及房间别名模板 |
| `limits` | 速率限制、延迟和大小阈值 |
| `ghosts` | 虚拟用户命名模式 |
| `metrics` | Prometheus 指标端点配置 |
| `telegram` | Telegram 连接设置和更新处理 |

## 命令

### Matrix 命令（在桥接房间中使用）

所有命令使用前缀 `!tg`（可通过 `bridge.command_prefix` 配置）。

| 命令 | 说明 |
|---|---|
| `!tg ping` | 检查桥接服务是否运行 |
| `!tg help` | 显示可用命令列表 |
| `!tg bridge <chat_id> [type]` | 将当前房间桥接到 Telegram 聊天（type 可选：`user`、`chat`、`channel`、`supergroup`） |
| `!tg unbridge` | 取消当前房间的桥接 |
| `!tg login` | 显示登录信息 |
| `!tg logout` | 显示登出信息 |
| `!tg whoami` | 显示已关联的账户信息 |
| `!tg status` | 显示桥接状态（Bot 连接、已桥接房间数、活跃虚拟用户数） |
| `!tg sync` | 触发当前房间的成员同步 |
| `!tg list` | 列出所有已桥接房间（仅管理员） |

### Telegram 命令（在桥接聊天中使用）

| 命令 | 说明 |
|---|---|
| `/start` | 显示欢迎消息 |
| `/help` | 显示可用命令列表 |
| `/bridge` | 显示当前聊天的桥接状态 |

## API 端点

桥接服务在配置的 `bridge.port`（默认：`29317`）上暴露 HTTP API。

| 方法 | 路径 | 说明 |
|---|---|---|
| `GET` | `/health` | 健康检查端点 |
| `GET` | `/metrics` | Prometheus 监控指标（启用时可用） |
| `PUT` | `/transactions/<txn_id>` | Matrix appservice 事务端点（需认证） |
| `GET` | `/thirdparty/protocol/telegram` | 第三方协议发现（Matrix 规范） |
| `GET` | `/thirdparty/user` | 第三方用户查询 |
| `GET` | `/thirdparty/location` | 第三方位置查询 |
| `GET` | `/v1/bridges` | 列出所有桥接（Provisioning API） |
| `POST` | `/v1/bridges` | 创建新桥接（Provisioning API） |
| `GET` | `/v1/bridges/<room_id>` | 获取指定房间的桥接详情 |
| `DELETE` | `/v1/bridges/<room_id>` | 移除指定房间的桥接 |
| `GET` | `/v1/portals` | 列出所有 Portal 房间 |
| `GET` | `/v1/users/<user_id>` | 获取用户信息 |

## CLI 命令

```bash
# 验证配置文件
matrix-bridge-telegram validate-config --config config.yaml

# 生成 appservice 注册文件
matrix-bridge-telegram generate-registration --id telegram --domain example.org

# 列出已桥接房间
matrix-bridge-telegram list-rooms --limit 50

# 取消房间桥接
matrix-bridge-telegram unbridge --room '!roomid:example.org'

# 授予管理员权限
matrix-bridge-telegram adminme --user '@admin:example.org' --power-level 100

# 数据库迁移
matrix-bridge-telegram migrate --from sqlite://old.db --to postgresql://user:pass@localhost/db

# 显示桥接状态
matrix-bridge-telegram status
```

## 架构概览

```
                    Matrix 服务器
                         |
                  [Appservice API]
                         |
              +----------+----------+
              |    WebServer (Salvo) |
              |   /transactions     |
              |   /health, /metrics |
              |   /v1/* provisioning|
              +----------+----------+
                         |
                  +------+------+
                  |  BridgeCore |
                  +------+------+
                  /      |      \
         +-------+  +----+----+  +--------+
         |Matrix |  | Portal  |  |Telegram|
         |Client |  | Manager |  | Client |
         +-------+  +---------+  +--------+
              |          |            |
         +----+----+ +---+---+  +-----+-----+
         | Event   | |Puppet |  |  Update   |
         | Handler | |Manager|  |  Handler  |
         +---------+ +-------+  +-----------+
              |          |            |
         +----+----------+------------+----+
         |         DatabaseManager         |
         |  (PostgreSQL / SQLite / MySQL)   |
         +---------------------------------+
```

**核心组件：**

- **WebServer** -- 基于 Salvo 的 HTTP 服务器，处理 Matrix appservice 事务、健康检查、指标监控和 Provisioning API。
- **BridgeCore** -- 中央协调器，管理 Matrix 与 Telegram 之间的消息流、Portal/Puppet 生命周期和媒体传输。
- **MatrixAppservice** -- 与 Matrix 服务器交互，负责发送消息、管理房间和创建虚拟用户。
- **TelegramClient** -- 通过 teloxide 连接 Telegram Bot API，处理接收的更新和发送的消息。
- **PortalManager** -- 跟踪 Matrix 房间与 Telegram 聊天之间的映射关系。
- **PuppetManager** -- 管理代表 Telegram 用户的 Matrix 虚拟用户。
- **MediaHandler** -- 在 Matrix 与 Telegram 之间传输媒体文件，包括格式转换。
- **DatabaseManager** -- 为 Portal、Puppet、消息映射和用户数据提供存储，支持 PostgreSQL、SQLite 或 MySQL。

## 贡献

欢迎贡献代码。请在 [GitHub](https://github.com/palpo-im/matrix-bridge-telegram) 上提交 Issue 或 Pull Request。

1. Fork 本仓库
2. 创建功能分支（`git checkout -b feature/my-feature`）
3. 提交你的更改（`git commit -am 'Add my feature'`）
4. 推送到分支（`git push origin feature/my-feature`）
5. 创建 Pull Request

提交前请确保代码通过 `cargo clippy` 和 `cargo test` 检查。

## 许可证

本项目采用 [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0) 许可证。

Copyright 2024-2026 Palpo Team
