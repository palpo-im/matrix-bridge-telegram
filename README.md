# matrix-bridge-telegram

[![CI](https://github.com/palpo-im/matrix-bridge-telegram/actions/workflows/ci.yml/badge.svg)](https://github.com/palpo-im/matrix-bridge-telegram/actions/workflows/ci.yml)
[![License: Apache-2.0](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://opensource.org/licenses/Apache-2.0)

A high-performance Matrix-Telegram bridge written in Rust. It enables bidirectional messaging between [Matrix](https://matrix.org/) rooms and [Telegram](https://telegram.org/) chats using the Matrix Application Service API and the Telegram Bot API.

## Features

- **Bidirectional messaging** -- text messages are relayed in both directions
- **Media support** -- photos, videos, audio, documents, voice messages, video notes, animations (GIFs), stickers, contacts, and locations
- **Message formatting** -- Markdown and HTML formatting preserved across platforms
- **Puppeting** -- Telegram users appear as individual Matrix ghost users with display names and avatars
- **Portal rooms** -- Telegram chats are automatically mapped to Matrix rooms
- **Bot commands** -- `!tg` commands on Matrix and `/` commands on Telegram
- **Room management** -- bridge, unbridge, sync members, and list rooms
- **Join/leave notifications** -- membership changes forwarded between platforms
- **Chat title and topic sync** -- title changes propagated across platforms
- **Message pinning** -- pinned messages forwarded from Telegram to Matrix
- **Read receipts** -- configurable read receipt forwarding
- **Presence** -- configurable presence status synchronization
- **Encryption** -- optional end-to-bridge encryption support
- **Multiple database backends** -- PostgreSQL, SQLite, and MySQL
- **Metrics** -- optional Prometheus-compatible metrics endpoint
- **Docker support** -- ready-to-use Dockerfile and docker-compose configuration
- **CLI tools** -- registration generation, config validation, room management, and database migration

## Requirements

- **Rust 1.93+** (for building from source)
- **PostgreSQL 14+**, **SQLite 3**, or **MySQL 8+** (database backend)
- **Telegram Bot Token** (obtain from [@BotFather](https://t.me/BotFather))
- **Telegram API credentials** (`api_id` and `api_hash` from [my.telegram.org](https://my.telegram.org/))
- **Matrix homeserver** with Application Service support (e.g., [Synapse](https://github.com/element-hq/synapse), [Conduit](https://conduit.rs/), [Dendrite](https://github.com/matrix-org/dendrite))

## Telegram Setup

Before running the bridge, you need to obtain Telegram credentials.

### 1. Get API Credentials (`api_id` and `api_hash`)

1. Visit [https://my.telegram.org/](https://my.telegram.org/) and log in with your phone number
2. Click **"API development tools"**
3. Fill in the form (app title and short name can be anything, e.g. "Matrix Bridge")
4. You will receive an `api_id` (integer) and `api_hash` (string) -- save these for your config

### 2. Create a Bot via BotFather

1. Open Telegram and search for [@BotFather](https://t.me/BotFather)
2. Send `/newbot` and follow the prompts to choose a name and username
3. BotFather will reply with a **bot token** like `123456789:ABCdefGhIJKlmnOPQRstUVwxyz` -- save this for your config
4. **Recommended bot settings** (send these commands to BotFather):
   - `/setprivacy` -> select your bot -> **Disable** (allows the bot to see all messages in groups, not just commands)
   - `/setjoingroups` -> select your bot -> **Enable** (allows the bot to be added to groups)
   - `/setcommands` -> select your bot -> send the following:
     ```
     start - Show welcome message
     help - Show available commands
     bridge - Show bridge status for this chat
     ```

### 3. Add the Bot to Telegram Groups

1. Open the Telegram group you want to bridge
2. Add the bot as a member (search by the bot's username)
3. **Promote the bot to admin** (recommended) -- this allows the bot to:
   - See all messages (required if privacy mode is not disabled)
   - Delete messages (for redaction forwarding)
   - Pin messages
   - Change group info

### 4. Get the Chat ID

To bridge a room, you need the Telegram chat ID. Methods to obtain it:

- **From the bot**: After adding the bot to a group, send any message. The bridge logs will show the `chat_id`.
- **Using the `/bridge` command**: Send `/bridge` in the Telegram group after the bot is added -- the bot will display the chat ID.
- **From Telegram web**: Open [https://web.telegram.org/](https://web.telegram.org/), open the group, and check the URL. The number after `#-` (with a `-` prefix for groups) is the chat ID (e.g., URL `#-1001234567890` means chat ID `-1001234567890`).

### 5. Fill in Your Configuration

Add the credentials to your config file.

**KDL format** (`config.kdl`):

```kdl
auth {
    api_id 12345678
    api_hash "0123456789abcdef"
    bot_token "123456789:ABCdefGhIJKlmnOPQRstUVwxyz"
}
```

**YAML format** (`config.yaml`):

```yaml
auth:
  api_id: 12345678            # From step 1
  api_hash: "0123456789abcdef" # From step 1
  bot_token: "123456789:ABCdefGhIJKlmnOPQRstUVwxyz"  # From step 2
```

Or use environment variables:

```bash
export APPSERVICE_TELEGRAM_AUTH_API_ID=12345678
export APPSERVICE_TELEGRAM_AUTH_API_HASH=0123456789abcdef
export APPSERVICE_TELEGRAM_AUTH_BOT_TOKEN=123456789:ABCdefGhIJKlmnOPQRstUVwxyz
```

### 6. Bridge a Room

Once the bridge is running and the bot is in a Telegram group:

1. In your Matrix room, send: `!tg bridge -1001234567890` (replace with your chat ID)
2. Or in the Telegram group, send `/bridge` to verify the bridge is active

## Quick Start

### Build from Source

```bash
# Clone the repository
git clone https://github.com/palpo-im/matrix-bridge-telegram.git
cd matrix-bridge-telegram

# Build with PostgreSQL and SQLite support (default)
cargo build --release

# Or build with only PostgreSQL
cargo build --release --no-default-features --features postgres

# Or build with only SQLite
cargo build --release --no-default-features --features sqlite
```

The binary will be located at `target/release/matrix-bridge-telegram`.

### Docker

```bash
# Build the Docker image
docker build -t matrix-bridge-telegram .

# Or use docker-compose (includes PostgreSQL)
docker-compose up -d
```

When using Docker, mount your configuration directory to `/data`:

```bash
docker run -v ./config:/data -p 29317:29317 matrix-bridge-telegram
```

### Initial Setup

1. **Copy and edit the sample configuration:**

   ```bash
   cp config/config.sample.yaml config.yaml
   # Edit config.yaml with your settings
   ```

2. **Generate a registration file for your homeserver:**

   ```bash
   ./matrix-bridge-telegram generate-registration \
     --id telegram \
     --homeserver-url http://localhost:8008 \
     --domain example.org \
     --output telegram-registration.yaml
   ```

3. **Register the appservice with your homeserver.** For Synapse, add the registration file path to `app_service_config_files` in `homeserver.yaml` and restart Synapse.

4. **Start the bridge:**

   ```bash
   ./matrix-bridge-telegram --config config.yaml
   ```

## Configuration

The bridge is configured through a YAML or KDL file. See [`config/config.sample.yaml`](config/config.sample.yaml) for the YAML example or [`config/config.example.kdl`](config/config.example.kdl) for the KDL example.

Configuration can also be loaded via the `CONFIG_PATH` environment variable. Sensitive values support environment variable overrides:

| Environment Variable | Config Field |
|---|---|
| `APPSERVICE_TELEGRAM_AUTH_API_ID` | `auth.api_id` |
| `APPSERVICE_TELEGRAM_AUTH_API_HASH` | `auth.api_hash` |
| `APPSERVICE_TELEGRAM_AUTH_BOT_TOKEN` | `auth.bot_token` |
| `APPSERVICE_TELEGRAM_REGISTRATION_ID` | `registration.bridge_id` |
| `APPSERVICE_TELEGRAM_REGISTRATION_AS_TOKEN` | `registration.appservice_token` |
| `APPSERVICE_TELEGRAM_REGISTRATION_HS_TOKEN` | `registration.homeserver_token` |
| `APPSERVICE_TELEGRAM_REGISTRATION_SENDER_LOCALPART` | `registration.sender_localpart` |

### Configuration Sections

| Section | Description |
|---|---|
| `bridge` | Core bridge settings: domain, port, homeserver URL, feature toggles |
| `registration` | Appservice registration: tokens, namespaces, sender localpart |
| `auth` | Telegram API credentials: `api_id`, `api_hash`, `bot_token` |
| `logging` | Log level, format, file output settings |
| `database` | Database connection URL and pool settings |
| `room` | Room creation and visibility defaults |
| `portal` | Username/displayname templates for ghost users and room aliases |
| `limits` | Rate limits, delays, and size thresholds |
| `ghosts` | Ghost user naming patterns |
| `metrics` | Prometheus metrics endpoint configuration |
| `telegram` | Telegram connection settings and update handling |

## Palpo KDL Configuration

When running in the [Palpo](https://github.com/palpo-im/palpo) environment, you can use KDL format for configuration. See [`config/config.example.kdl`](config/config.example.kdl) for the full annotated example.

Key sections in KDL format:

```kdl
bridge {
    domain "example.org"
    homeserver_url "http://localhost:8008"
    port 29317
    bind_address "0.0.0.0"
    command_prefix "!tg"
    admin_mxid "@admin:example.org"
}

auth {
    api_id 12345
    api_hash "your_api_hash"
    bot_token "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11"
}

registration {
    bridge_id "telegram"
    appservice_token "your_as_token"
    homeserver_token "your_hs_token"
    sender_localpart "_telegram_"
    namespaces {
        users {
            - exclusive=true regex="@_telegram_.*:example.org"
        }
    }
}

database {
    url "postgresql://bridge:changeme@localhost:5432/telegram_bridge"
    max_connections 10
}
```

## Commands

### Matrix Commands (in bridged rooms)

All commands use the prefix `!tg` (configurable via `bridge.command_prefix`).

| Command | Description |
|---|---|
| `!tg ping` | Check if the bridge is running |
| `!tg help` | Show available commands |
| `!tg bridge <chat_id> [type]` | Bridge the current room to a Telegram chat (type: `user`, `chat`, `channel`, `supergroup`) |
| `!tg unbridge` | Remove the bridge from the current room |
| `!tg login` | Show login information |
| `!tg logout` | Show logout information |
| `!tg whoami` | Show your linked account info |
| `!tg status` | Show bridge status (bot connection, bridged rooms, active puppets) |
| `!tg sync` | Trigger member synchronization for the current room |
| `!tg list` | List all bridged rooms (admin only) |

### Telegram Commands (in bridged chats)

| Command | Description |
|---|---|
| `/start` | Show welcome message |
| `/help` | Show available commands |
| `/bridge` | Show bridge status for the current chat |

## API Endpoints

The bridge exposes an HTTP API on the configured `bridge.port` (default: `29317`).

| Method | Path | Description |
|---|---|---|
| `GET` | `/health` | Health check endpoint |
| `GET` | `/metrics` | Prometheus metrics (when enabled) |
| `PUT` | `/transactions/<txn_id>` | Matrix appservice transaction endpoint (authenticated) |
| `GET` | `/thirdparty/protocol/telegram` | Third-party protocol discovery (Matrix spec) |
| `GET` | `/thirdparty/user` | Third-party user lookup |
| `GET` | `/thirdparty/location` | Third-party location lookup |
| `GET` | `/v1/bridges` | List all bridges (provisioning API) |
| `POST` | `/v1/bridges` | Create a new bridge (provisioning API) |
| `GET` | `/v1/bridges/<room_id>` | Get bridge details for a room |
| `DELETE` | `/v1/bridges/<room_id>` | Remove a bridge from a room |
| `GET` | `/v1/portals` | List all portal rooms |
| `GET` | `/v1/users/<user_id>` | Get user info |

## CLI Commands

```bash
# Validate configuration
matrix-bridge-telegram validate-config --config config.yaml

# Generate appservice registration file
matrix-bridge-telegram generate-registration --id telegram --domain example.org

# List bridged rooms
matrix-bridge-telegram list-rooms --limit 50

# Unbridge a room
matrix-bridge-telegram unbridge --room '!roomid:example.org'

# Grant admin privileges
matrix-bridge-telegram adminme --user '@admin:example.org' --power-level 100

# Migrate database
matrix-bridge-telegram migrate --from sqlite://old.db --to postgresql://user:pass@localhost/db

# Show bridge status
matrix-bridge-telegram status
```

## Architecture

```
                    Matrix Homeserver
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

**Key components:**

- **WebServer** -- Salvo-based HTTP server handling Matrix appservice transactions, health checks, metrics, and the provisioning API.
- **BridgeCore** -- Central coordinator managing message flow between Matrix and Telegram, portal/puppet lifecycle, and media transfers.
- **MatrixAppservice** -- Interfaces with the Matrix homeserver for sending messages, managing rooms, and creating ghost users.
- **TelegramClient** -- Connects to the Telegram Bot API via teloxide, handling incoming updates and outgoing messages.
- **PortalManager** -- Tracks the mapping between Matrix rooms and Telegram chats.
- **PuppetManager** -- Manages Matrix ghost users that represent Telegram users.
- **MediaHandler** -- Transfers media files between Matrix and Telegram, including format conversion.
- **DatabaseManager** -- Provides storage for portals, puppets, message mappings, and user data across PostgreSQL, SQLite, or MySQL.

## Contributing

Contributions are welcome. Please open an issue or submit a pull request on [GitHub](https://github.com/palpo-im/matrix-bridge-telegram).

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/my-feature`)
3. Commit your changes (`git commit -am 'Add my feature'`)
4. Push to the branch (`git push origin feature/my-feature`)
5. Open a Pull Request

Please ensure your code passes `cargo clippy` and `cargo test` before submitting.

## License

This project is licensed under the [Apache License 2.0](https://www.apache.org/licenses/LICENSE-2.0).

Copyright 2024-2026 Palpo Team
