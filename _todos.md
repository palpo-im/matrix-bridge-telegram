# Matrix-Bridge-Telegram 功能完善清单

基于对比 `matrix-bridge-discord` 和 `references/telegram` 项目，列出以下功能实现计划。

## Phase 1: 项目基础架构 (Project Foundation)

- [x] 1.1 初始化 Cargo 项目
  - [x] 创建 `Cargo.toml`
  - [x] 配置依赖 (salvo, matrix-bot-sdk, teloxide, diesel, tokio, etc.)
  - [x] 创建 `src/main.rs`

- [x] 1.2 创建基础模块结构
  - [x] 创建 `src/lib.rs`
  - [x] 创建 `src/cli.rs` - 命令行参数处理
  - [x] 创建 `src/config/` - 配置模块
  - [x] 创建 `src/utils/` - 工具模块

- [x] 1.3 配置文件
  - [x] 创建 `config/config.sample.yaml` 配置模板
  - [x] 实现配置解析模块

## Phase 2: 数据库设计 (Database Design)

- [x] 2.1 数据库 Schema
  - [x] 创建 `user_mappings` 表 - 用户映射
  - [x] 创建 `portal` 表 - Portal 映射
  - [x] 创建 `message_mappings` 表 - 消息映射
  - [x] 创建 `reaction_mappings` 表 - 反应映射
  - [x] 创建 `telegram_files` 表 - 文件缓存

- [x] 2.2 数据库管理器
  - [x] 创建 `src/db/mod.rs`
  - [x] 创建 `src/db/manager.rs` - 数据库管理器
  - [x] 创建 `src/db/models.rs` - 数据模型
  - [x] 支持 PostgreSQL
  - [x] 支持 MySQL
  - [x] 支持 SQLite

- [x] 2.3 数据存储 (Stores)
  - [x] 创建 `src/db/stores/mod.rs`
  - [x] 创建 UserStore - 用户存储
  - [x] 创建 PortalStore - Portal 存储
  - [x] 创建 MessageStore - 消息存储
  - [x] 创建 ReactionStore - 反应存储
  - [x] 创建 TelegramFileStore - 文件存储

## Phase 3: Telegram 客户端 (Telegram Client)

- [x] 3.1 Telegram Bot 客户端
  - [x] 创建 `src/telegram/mod.rs`
  - [x] 创建 `src/telegram/client.rs` - 客户端封装
  - [x] 创建 `src/telegram/handler.rs` - 事件处理器

- [ ] 3.2 Telegram 消息处理
  - [ ] 文本消息处理
  - [ ] 图片消息处理
  - [ ] 视频消息处理
  - [ ] 音频消息处理
  - [ ] 文件消息处理
  - [ ] 贴纸消息处理
  - [ ] 回复消息处理
  - [ ] 编辑消息处理
  - [ ] 删除消息处理

## Phase 4: Matrix 客户端 (Matrix Client)

- [x] 4.1 Matrix Appservice 客户端
  - [x] 创建 `src/matrix/mod.rs`
  - [x] 创建 `src/matrix/event_handler.rs` - 事件处理器

- [x] 4.2 Matrix 命令处理
  - [x] 创建 `src/matrix/command_handler.rs`

- [ ] 4.3 Matrix 事件处理
  - [ ] 房间消息事件
  - [ ] 消息编辑事件
  - [ ] 消息红action事件
  - [ ] 房间状态事件
  - [ ] 成员事件

## Phase 5: 消息解析与转换 (Message Parsing & Conversion)

- [x] 5.1 创建 `src/parsers` 模块
  - [x] 创建 `src/parsers/mod.rs`
  - [x] 创建 `src/parsers/telegram_parser.rs` - Telegram 消息解析器
  - [x] 创建 `src/parsers/matrix_parser.rs` - Matrix 消息解析器
  - [x] 创建 `src/parsers/common.rs` - 通用消息类型定义

- [ ] 5.2 Telegram -> Matrix 消息转换
  - [ ] 文本消息转换 (支持 Markdown/HTML)
  - [ ] 媒体消息转换
  - [ ] @提及 转换
  - [ ] 回复消息转换
  - [ ] 表情反应转换

- [ ] 5.3 Matrix -> Telegram 消息转换
  - [ ] HTML 转 Telegram Markdown
  - [ ] Matrix @mention 转 Telegram @
  - [ ] Matrix 媒体转 Telegram 媒体

## Phase 6: 媒体处理 (Media Handling)

- [x] 6.1 创建 `src/media.rs` 模块
  - [ ] 下载 Matrix 媒体文件
  - [ ] 上传媒体到 Matrix
  - [ ] 从 Telegram 下载媒体
  - [ ] 上传媒体到 Telegram
  - [ ] MIME 类型检测
  - [ ] 文件大小限制检查

- [ ] 6.2 贴纸转换
  - [ ] TGS 贴纸转 PNG/GIF
  - [ ] WebM 动画贴纸转换

## Phase 7: 桥接核心 (Bridge Core)

- [x] 7.1 创建 `src/bridge.rs` 模块
  - [x] 创建 `src/bridge/mod.rs`
  - [x] 创建 `src/bridge/portal.rs` - Portal 管理
  - [x] 创建 `src/bridge/puppet.rs` - Puppet 用户管理
  - [x] 创建 `src/bridge/user_sync.rs` - 用户同步
  - [x] 创建 `src/bridge/message_flow.rs` - 消息流转

- [ ] 7.2 Portal 功能
  - [ ] Portal 创建
  - [ ] Portal 桥接
  - [ ] Portal 解除
  - [ ] 房间元数据同步

- [ ] 7.3 Puppet 功能
  - [ ] Puppet 用户创建
  - [ ] Puppet 用户信息同步
  - [ ] Double Puppet 支持

## Phase 8: 命令系统 (Command System)

- [ ] 8.1 Matrix 端命令
  - [ ] 实现 `!tg ping` 命令
  - [ ] 实现 `!tg bridge <chat_id>` 命令
  - [ ] 实现 `!tg unbridge` 命令
  - [ ] 实现 `!tg login` 命令
  - [ ] 实现 `!tg logout` 命令
  - [ ] 权限检查

- [ ] 8.2 Telegram 端命令
  - [ ] 实现 `/start` 命令
  - [ ] 实现 `/bridge` 命令
  - [ ] 实现 `/help` 命令

## Phase 9: Web 服务 (Web Service)

- [x] 9.1 创建 `src/web` 模块
  - [x] 创建 `src/web/mod.rs`
  - [x] 创建 `src/web/health.rs` - 健康检查端点
  - [x] 创建 `src/web/metrics.rs` - Prometheus 指标端点

- [x] 9.2 Provisioning API
  - [x] 创建 `src/web/provisioning.rs`
  - [ ] GET /_matrix/app/v1/rooms - 列出桥接房间
  - [ ] POST /_matrix/app/v1/bridges - 创建桥接
  - [ ] GET /_matrix/app/v1/bridges/{id} - 获取桥接信息
  - [ ] DELETE /_matrix/app/v1/bridges/{id} - 删除桥接

- [x] 9.3 第三方协议 API
  - [x] 创建 `src/web/thirdparty.rs`
  - [ ] GET /_matrix/app/v1/thirdparty/protocol
  - [ ] GET /_matrix/app/v1/thirdparty/network
  - [ ] GET /_matrix/app/v1/thirdparty/user
  - [ ] GET /_matrix/app/v1/thirdparty/location

## Phase 10: 高级功能 (Advanced Features)

- [ ] 10.1 消息编辑支持
  - [ ] Matrix 消息编辑转发到 Telegram
  - [ ] Telegram 消息编辑转发到 Matrix

- [ ] 10.2 消息撤回支持
  - [ ] Matrix redaction 转发到 Telegram (删除)
  - [ ] Telegram 删除消息转发到 Matrix

- [ ] 10.3 反应 (Reactions) 支持
  - [ ] Matrix 反应转发到 Telegram
  - [ ] Telegram 反应转发到 Matrix

- [ ] 10.4 在线状态
  - [ ] Telegram 在线状态转发到 Matrix
  - [ ] 打字状态同步

- [ ] 10.5 消息回填
  - [ ] 创建 `src/backfill.rs`
  - [ ] 历史消息回填支持

## Phase 11: 缓存系统 (Caching System)

- [x] 11.1 创建 `src/cache.rs` 模块
  - [x] 实现 TimedCache (TTL 缓存)
  - [ ] 实现 AsyncTimedCache (异步 TTL 缓存)
  - [ ] Portal 映射缓存
  - [ ] 用户信息缓存

## Phase 12: 工具模块 (Utility Modules)

- [x] 12.1 创建 `src/utils` 模块
  - [x] 创建 `src/utils/mod.rs`
  - [x] 创建 `src/utils/error.rs` - 自定义错误类型
  - [x] 创建 `src/utils/formatting.rs` - 格式化工具
  - [x] 创建 `src/utils/logging.rs` - 日志初始化

## Phase 13: 部署与文档 (Deployment & Documentation)

- [ ] 13.1 Docker 支持
  - [ ] 创建 `Dockerfile`
  - [ ] 创建 `docker-compose.yaml`
  - [ ] 创建 `.dockerignore`

- [ ] 13.2 文档
  - [ ] 创建 `README.md`
  - [ ] 创建 `README_CN.md`
  - [ ] 添加配置示例说明
  - [ ] 添加 API 文档

- [ ] 13.3 CI/CD
  - [ ] 创建 `.github/workflows/ci.yml`
  - [ ] 创建 `.github/workflows/release.yml`

---

## 优先级说明

- **Phase 1-2**: 基础架构，必须完成 ✅
- **Phase 3-7**: 核心功能，必须完成 (进行中)
- **Phase 8-9**: 高级功能，建议完成
- **Phase 10-12**: 增强功能，可选完成
- **Phase 13**: 部署文档，最后完成
