use std::sync::Arc;

#[cfg(any(feature = "postgres", feature = "mysql", feature = "sqlite"))]
use diesel::RunQueryDsl;
#[cfg(feature = "mysql")]
use diesel::mysql::MysqlConnection;
#[cfg(feature = "postgres")]
use diesel::pg::PgConnection;
#[cfg(any(feature = "postgres", feature = "mysql"))]
use diesel::r2d2::{self, ConnectionManager};

use crate::config::{DatabaseConfig as ConfigDatabaseConfig, DbType as ConfigDbType};
use crate::db::stores::{
    InMemoryMessageStore, InMemoryPortalStore, InMemoryReactionStore, InMemoryTelegramFileStore,
    InMemoryUserStore,
};
use crate::db::{
    DatabaseError, MessageStore, PortalStore, ReactionStore, TelegramFileStore, UserStore,
};

#[cfg(feature = "postgres")]
pub type Pool = r2d2::Pool<ConnectionManager<PgConnection>>;
#[cfg(feature = "mysql")]
pub type MysqlPool = r2d2::Pool<ConnectionManager<MysqlConnection>>;

#[cfg(feature = "sqlite")]
use diesel::Connection;
#[cfg(feature = "sqlite")]
use diesel::sqlite::SqliteConnection;

#[derive(Clone)]
pub struct DatabaseManager {
    #[cfg(feature = "postgres")]
    postgres_pool: Option<Pool>,
    #[cfg(feature = "mysql")]
    mysql_pool: Option<MysqlPool>,
    #[cfg(feature = "sqlite")]
    sqlite_path: Option<String>,
    user_store: Arc<dyn UserStore>,
    portal_store: Arc<dyn PortalStore>,
    message_store: Arc<dyn MessageStore>,
    reaction_store: Arc<dyn ReactionStore>,
    telegram_file_store: Arc<dyn TelegramFileStore>,
    db_type: DbType,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DbType {
    Postgres,
    Sqlite,
    Mysql,
}

impl From<ConfigDbType> for DbType {
    fn from(value: ConfigDbType) -> Self {
        match value {
            ConfigDbType::Postgres => DbType::Postgres,
            ConfigDbType::Sqlite => DbType::Sqlite,
            ConfigDbType::Mysql => DbType::Mysql,
        }
    }
}

impl DatabaseManager {
    pub async fn new(config: &ConfigDatabaseConfig) -> Result<Self, DatabaseError> {
        let db_type = DbType::from(config.db_type());

        match db_type {
            #[cfg(feature = "postgres")]
            DbType::Postgres => {
                let connection_string = config.connection_string();
                let max_connections = config.max_connections();
                let min_connections = config.min_connections();

                let manager = ConnectionManager::<PgConnection>::new(connection_string);

                let builder = r2d2::Pool::builder()
                    .max_size(max_connections.unwrap_or(10))
                    .min_idle(Some(min_connections.unwrap_or(1)));

                let pool = builder
                    .build(manager)
                    .map_err(|e| DatabaseError::Connection(e.to_string()))?;

                let user_store = Arc::new(InMemoryUserStore::new());
                let portal_store = Arc::new(InMemoryPortalStore::new());
                let message_store = Arc::new(InMemoryMessageStore::new());
                let reaction_store = Arc::new(InMemoryReactionStore::new());
                let telegram_file_store = Arc::new(InMemoryTelegramFileStore::new());

                Ok(Self {
                    postgres_pool: Some(pool),
                    #[cfg(feature = "mysql")]
                    mysql_pool: None,
                    #[cfg(feature = "sqlite")]
                    sqlite_path: None,
                    user_store,
                    portal_store,
                    message_store,
                    reaction_store,
                    telegram_file_store,
                    db_type,
                })
            }
            #[cfg(feature = "sqlite")]
            DbType::Sqlite => {
                let path = config.sqlite_path().unwrap();
                let _path_arc = Arc::new(path.clone());

                let user_store = Arc::new(InMemoryUserStore::new());
                let portal_store = Arc::new(InMemoryPortalStore::new());
                let message_store = Arc::new(InMemoryMessageStore::new());
                let reaction_store = Arc::new(InMemoryReactionStore::new());
                let telegram_file_store = Arc::new(InMemoryTelegramFileStore::new());

                Ok(Self {
                    #[cfg(feature = "postgres")]
                    postgres_pool: None,
                    #[cfg(feature = "mysql")]
                    mysql_pool: None,
                    sqlite_path: Some(path),
                    user_store,
                    portal_store,
                    message_store,
                    reaction_store,
                    telegram_file_store,
                    db_type,
                })
            }
            #[cfg(feature = "mysql")]
            DbType::Mysql => {
                let connection_string = config.connection_string();
                let max_connections = config.max_connections();
                let min_connections = config.min_connections();

                let manager = ConnectionManager::<MysqlConnection>::new(connection_string);

                let builder = r2d2::Pool::builder()
                    .max_size(max_connections.unwrap_or(10))
                    .min_idle(Some(min_connections.unwrap_or(1)));

                let pool = builder
                    .build(manager)
                    .map_err(|e| DatabaseError::Connection(e.to_string()))?;

                let user_store = Arc::new(InMemoryUserStore::new());
                let portal_store = Arc::new(InMemoryPortalStore::new());
                let message_store = Arc::new(InMemoryMessageStore::new());
                let reaction_store = Arc::new(InMemoryReactionStore::new());
                let telegram_file_store = Arc::new(InMemoryTelegramFileStore::new());

                Ok(Self {
                    #[cfg(feature = "postgres")]
                    postgres_pool: None,
                    mysql_pool: Some(pool),
                    #[cfg(feature = "sqlite")]
                    sqlite_path: None,
                    user_store,
                    portal_store,
                    message_store,
                    reaction_store,
                    telegram_file_store,
                    db_type,
                })
            }
            #[cfg(not(feature = "postgres"))]
            DbType::Postgres => {
                return Err(DatabaseError::Connection(
                    "PostgreSQL feature not enabled".to_string(),
                ));
            }
            #[cfg(not(feature = "sqlite"))]
            DbType::Sqlite => {
                return Err(DatabaseError::Connection(
                    "SQLite feature not enabled".to_string(),
                ));
            }
            #[cfg(not(feature = "mysql"))]
            DbType::Mysql => {
                return Err(DatabaseError::Connection(
                    "MySQL feature not enabled".to_string(),
                ));
            }
        }
    }

    #[cfg(feature = "sqlite")]
    pub fn new_in_memory() -> Result<Self, DatabaseError> {
        use std::sync::Arc;

        let user_store = Arc::new(InMemoryUserStore::new());
        let portal_store = Arc::new(InMemoryPortalStore::new());
        let message_store = Arc::new(InMemoryMessageStore::new());
        let reaction_store = Arc::new(InMemoryReactionStore::new());
        let telegram_file_store = Arc::new(InMemoryTelegramFileStore::new());

        Ok(Self {
            #[cfg(feature = "postgres")]
            postgres_pool: None,
            #[cfg(feature = "mysql")]
            mysql_pool: None,
            sqlite_path: Some(":memory:".to_string()),
            user_store,
            portal_store,
            message_store,
            reaction_store,
            telegram_file_store,
            db_type: DbType::Sqlite,
        })
    }

    pub async fn migrate(&self) -> Result<(), DatabaseError> {
        match self.db_type {
            #[cfg(feature = "postgres")]
            DbType::Postgres => {
                let pool = self.postgres_pool.as_ref().unwrap();
                return Self::migrate_postgres(pool).await;
            }
            #[cfg(feature = "sqlite")]
            DbType::Sqlite => {
                let path = self.sqlite_path.as_ref().unwrap();
                return Self::migrate_sqlite(path).await;
            }
            #[cfg(feature = "mysql")]
            DbType::Mysql => {
                let pool = self.mysql_pool.as_ref().unwrap();
                return Self::migrate_mysql(pool).await;
            }
            #[cfg(not(feature = "postgres"))]
            DbType::Postgres => {
                return Err(DatabaseError::Migration(
                    "PostgreSQL feature not enabled".to_string(),
                ));
            }
            #[cfg(not(feature = "sqlite"))]
            DbType::Sqlite => {
                return Err(DatabaseError::Migration(
                    "SQLite feature not enabled".to_string(),
                ));
            }
            #[cfg(not(feature = "mysql"))]
            DbType::Mysql => {
                return Err(DatabaseError::Migration(
                    "MySQL feature not enabled".to_string(),
                ));
            }
        }
    }

    #[cfg(feature = "postgres")]
    async fn migrate_postgres(pool: &Pool) -> Result<(), DatabaseError> {
        let pool = pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| DatabaseError::Connection(e.to_string()))?;

            let statements = [
                r#"
                CREATE TABLE IF NOT EXISTS user_mappings (
                    id BIGSERIAL PRIMARY KEY,
                    matrix_user_id TEXT NOT NULL UNIQUE,
                    telegram_user_id BIGINT NOT NULL UNIQUE,
                    telegram_username TEXT,
                    telegram_first_name TEXT,
                    telegram_last_name TEXT,
                    telegram_phone TEXT,
                    telegram_avatar TEXT,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS portal (
                    id BIGSERIAL PRIMARY KEY,
                    matrix_room_id TEXT NOT NULL UNIQUE,
                    telegram_chat_id BIGINT NOT NULL UNIQUE,
                    telegram_chat_type TEXT NOT NULL,
                    telegram_chat_title TEXT,
                    telegram_chat_username TEXT,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS message_mappings (
                    id BIGSERIAL PRIMARY KEY,
                    telegram_message_id BIGINT NOT NULL,
                    telegram_chat_id BIGINT NOT NULL,
                    matrix_room_id TEXT NOT NULL,
                    matrix_event_id TEXT NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    updated_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    UNIQUE(telegram_chat_id, telegram_message_id)
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS reaction_mappings (
                    id BIGSERIAL PRIMARY KEY,
                    telegram_message_id BIGINT NOT NULL,
                    telegram_chat_id BIGINT NOT NULL,
                    telegram_user_id BIGINT NOT NULL,
                    reaction_emoji TEXT NOT NULL,
                    matrix_event_id TEXT NOT NULL,
                    matrix_room_id TEXT NOT NULL,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW(),
                    UNIQUE(telegram_chat_id, telegram_message_id, telegram_user_id, reaction_emoji)
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS telegram_files (
                    id BIGSERIAL PRIMARY KEY,
                    telegram_file_id TEXT NOT NULL,
                    telegram_file_unique_id TEXT NOT NULL UNIQUE,
                    mxc_url TEXT NOT NULL,
                    mime_type TEXT,
                    file_name TEXT,
                    file_size BIGINT,
                    created_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS processed_events (
                    id BIGSERIAL PRIMARY KEY,
                    event_id TEXT NOT NULL UNIQUE,
                    event_type TEXT NOT NULL,
                    source TEXT NOT NULL,
                    processed_at TIMESTAMP WITH TIME ZONE NOT NULL DEFAULT NOW()
                )
                "#,
                "CREATE INDEX IF NOT EXISTS idx_user_mappings_matrix_id ON user_mappings(matrix_user_id)",
                "CREATE INDEX IF NOT EXISTS idx_user_mappings_telegram_id ON user_mappings(telegram_user_id)",
                "CREATE INDEX IF NOT EXISTS idx_portal_matrix_room ON portal(matrix_room_id)",
                "CREATE INDEX IF NOT EXISTS idx_portal_telegram_chat ON portal(telegram_chat_id)",
                "CREATE INDEX IF NOT EXISTS idx_message_mappings_telegram ON message_mappings(telegram_chat_id, telegram_message_id)",
                "CREATE INDEX IF NOT EXISTS idx_message_mappings_matrix ON message_mappings(matrix_room_id, matrix_event_id)",
                "CREATE INDEX IF NOT EXISTS idx_reaction_mappings_telegram ON reaction_mappings(telegram_chat_id, telegram_message_id)",
                "CREATE INDEX IF NOT EXISTS idx_telegram_files_unique ON telegram_files(telegram_file_unique_id)",
                "CREATE INDEX IF NOT EXISTS idx_processed_events_event_id ON processed_events(event_id)",
            ];

            for statement in statements {
                diesel::sql_query(statement)
                    .execute(&mut conn)
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;
            }

            Ok(())
        })
        .await
        .map_err(|e| DatabaseError::Migration(format!("migration task failed: {e}")))?
    }

    #[cfg(feature = "mysql")]
    async fn migrate_mysql(pool: &MysqlPool) -> Result<(), DatabaseError> {
        let pool = pool.clone();
        tokio::task::spawn_blocking(move || {
            let mut conn = pool
                .get()
                .map_err(|e| DatabaseError::Connection(e.to_string()))?;

            let statements = [
                r#"
                CREATE TABLE IF NOT EXISTS user_mappings (
                    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                    matrix_user_id VARCHAR(255) NOT NULL UNIQUE,
                    telegram_user_id BIGINT NOT NULL UNIQUE,
                    telegram_username VARCHAR(255) NULL,
                    telegram_first_name VARCHAR(255) NULL,
                    telegram_last_name VARCHAR(255) NULL,
                    telegram_phone VARCHAR(32) NULL,
                    telegram_avatar TEXT NULL,
                    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
                    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6)
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS portal (
                    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                    matrix_room_id VARCHAR(255) NOT NULL UNIQUE,
                    telegram_chat_id BIGINT NOT NULL UNIQUE,
                    telegram_chat_type VARCHAR(32) NOT NULL,
                    telegram_chat_title VARCHAR(255) NULL,
                    telegram_chat_username VARCHAR(255) NULL,
                    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
                    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6)
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS message_mappings (
                    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                    telegram_message_id BIGINT NOT NULL,
                    telegram_chat_id BIGINT NOT NULL,
                    matrix_room_id VARCHAR(255) NOT NULL,
                    matrix_event_id VARCHAR(255) NOT NULL,
                    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
                    updated_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6) ON UPDATE CURRENT_TIMESTAMP(6),
                    UNIQUE KEY uk_telegram_message (telegram_chat_id, telegram_message_id)
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS reaction_mappings (
                    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                    telegram_message_id BIGINT NOT NULL,
                    telegram_chat_id BIGINT NOT NULL,
                    telegram_user_id BIGINT NOT NULL,
                    reaction_emoji VARCHAR(64) NOT NULL,
                    matrix_event_id VARCHAR(255) NOT NULL,
                    matrix_room_id VARCHAR(255) NOT NULL,
                    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6),
                    UNIQUE KEY uk_telegram_reaction (telegram_chat_id, telegram_message_id, telegram_user_id, reaction_emoji)
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS telegram_files (
                    id BIGINT NOT NULL AUTO_INCREMENT PRIMARY KEY,
                    telegram_file_id VARCHAR(255) NOT NULL,
                    telegram_file_unique_id VARCHAR(255) NOT NULL UNIQUE,
                    mxc_url VARCHAR(1024) NOT NULL,
                    mime_type VARCHAR(128) NULL,
                    file_name VARCHAR(255) NULL,
                    file_size BIGINT NULL,
                    created_at DATETIME(6) NOT NULL DEFAULT CURRENT_TIMESTAMP(6)
                ) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4
                "#,
            ];

            for statement in statements {
                diesel::sql_query(statement)
                    .execute(&mut conn)
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;
            }

            Ok(())
        })
        .await
        .map_err(|e| DatabaseError::Migration(format!("migration task failed: {e}")))?
    }

    #[cfg(feature = "sqlite")]
    async fn migrate_sqlite(path: &str) -> Result<(), DatabaseError> {
        let path = path.to_string();
        tokio::task::spawn_blocking(move || {
            let conn_string = format!("sqlite://{}", path);
            let mut conn = SqliteConnection::establish(&conn_string)
                .map_err(|e| DatabaseError::Connection(e.to_string()))?;

            let statements = [
                r#"
                CREATE TABLE IF NOT EXISTS user_mappings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    matrix_user_id TEXT NOT NULL UNIQUE,
                    telegram_user_id INTEGER NOT NULL UNIQUE,
                    telegram_username TEXT,
                    telegram_first_name TEXT,
                    telegram_last_name TEXT,
                    telegram_phone TEXT,
                    telegram_avatar TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS portal (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    matrix_room_id TEXT NOT NULL UNIQUE,
                    telegram_chat_id INTEGER NOT NULL UNIQUE,
                    telegram_chat_type TEXT NOT NULL,
                    telegram_chat_title TEXT,
                    telegram_chat_username TEXT,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS message_mappings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    telegram_message_id INTEGER NOT NULL,
                    telegram_chat_id INTEGER NOT NULL,
                    matrix_room_id TEXT NOT NULL,
                    matrix_event_id TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    updated_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE(telegram_chat_id, telegram_message_id)
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS reaction_mappings (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    telegram_message_id INTEGER NOT NULL,
                    telegram_chat_id INTEGER NOT NULL,
                    telegram_user_id INTEGER NOT NULL,
                    reaction_emoji TEXT NOT NULL,
                    matrix_event_id TEXT NOT NULL,
                    matrix_room_id TEXT NOT NULL,
                    created_at TEXT NOT NULL DEFAULT (datetime('now')),
                    UNIQUE(telegram_chat_id, telegram_message_id, telegram_user_id, reaction_emoji)
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS telegram_files (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    telegram_file_id TEXT NOT NULL,
                    telegram_file_unique_id TEXT NOT NULL UNIQUE,
                    mxc_url TEXT NOT NULL,
                    mime_type TEXT,
                    file_name TEXT,
                    file_size INTEGER,
                    created_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
                "#,
                r#"
                CREATE TABLE IF NOT EXISTS processed_events (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    event_id TEXT NOT NULL UNIQUE,
                    event_type TEXT NOT NULL,
                    source TEXT NOT NULL,
                    processed_at TEXT NOT NULL DEFAULT (datetime('now'))
                )
                "#,
                "CREATE INDEX IF NOT EXISTS idx_user_mappings_matrix_id ON user_mappings(matrix_user_id)",
                "CREATE INDEX IF NOT EXISTS idx_user_mappings_telegram_id ON user_mappings(telegram_user_id)",
                "CREATE INDEX IF NOT EXISTS idx_portal_matrix_room ON portal(matrix_room_id)",
                "CREATE INDEX IF NOT EXISTS idx_portal_telegram_chat ON portal(telegram_chat_id)",
                "CREATE INDEX IF NOT EXISTS idx_message_mappings_telegram ON message_mappings(telegram_chat_id, telegram_message_id)",
                "CREATE INDEX IF NOT EXISTS idx_message_mappings_matrix ON message_mappings(matrix_room_id, matrix_event_id)",
                "CREATE INDEX IF NOT EXISTS idx_reaction_mappings_telegram ON reaction_mappings(telegram_chat_id, telegram_message_id)",
                "CREATE INDEX IF NOT EXISTS idx_telegram_files_unique ON telegram_files(telegram_file_unique_id)",
                "CREATE INDEX IF NOT EXISTS idx_processed_events_event_id ON processed_events(event_id)",
            ];

            for statement in statements {
                diesel::sql_query(statement)
                    .execute(&mut conn)
                    .map_err(|e| DatabaseError::Migration(e.to_string()))?;
            }

            Ok(())
        })
        .await
        .map_err(|e| DatabaseError::Migration(format!("migration task failed: {e}")))?
    }

    pub fn user_store(&self) -> Arc<dyn UserStore> {
        self.user_store.clone()
    }

    pub fn portal_store(&self) -> Arc<dyn PortalStore> {
        self.portal_store.clone()
    }

    pub fn message_store(&self) -> Arc<dyn MessageStore> {
        self.message_store.clone()
    }

    pub fn reaction_store(&self) -> Arc<dyn ReactionStore> {
        self.reaction_store.clone()
    }

    pub fn telegram_file_store(&self) -> Arc<dyn TelegramFileStore> {
        self.telegram_file_store.clone()
    }

    #[cfg(feature = "postgres")]
    pub fn pool(&self) -> Option<&Pool> {
        self.postgres_pool.as_ref()
    }

    pub fn db_type(&self) -> DbType {
        self.db_type
    }
}
