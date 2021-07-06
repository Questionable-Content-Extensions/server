use std::collections::HashSet;

use futures::lock::Mutex;
use log::info;
use once_cell::sync::Lazy;
use uuid::Uuid;

use crate::database::DbPool;

macro_rules! lazy_environment {
    ($static_name:ident, $name:ident) => {
        #[allow(dead_code)]
        static $static_name: Lazy<String> = Lazy::new(|| {
            std::env::var(stringify!($static_name)).expect(concat!(
                "Tried reading environment variable '",
                stringify!($static_name),
                "'"
            ))
        });

        impl Environment {
            #[allow(dead_code)]
            pub fn $name() -> &'static str {
                &*$static_name
            }
        }
    };
}

pub struct Environment;

lazy_environment!(PORT, port);
lazy_environment!(DATABASE_URL, database_url);

impl Environment {
    pub fn init() {
        dotenv::from_filename(".env").ok();
    }
}

pub struct NewsUpdater {
    update_set: Mutex<HashSet<i16>>,
}

impl NewsUpdater {
    pub fn new() -> Self {
        Self {
            update_set: Mutex::new(HashSet::new()),
        }
    }

    pub async fn check_for(&self, comic: i16) {
        info!("Scheduling a news update check for comic {}", comic);

        let mut update_set = self.update_set.lock().await;
        update_set.insert(comic);
    }
}

pub async fn is_token_valid(conn: &DbPool, token: Uuid) -> Result<bool, sqlx::Error> {
    if token.is_nil() {
        return Ok(false);
    }

    let result = sqlx::query_scalar!(
        r#"
		SELECT id FROM `token`
		WHERE `id` = ?
	"#,
        token.to_string()
    )
    .fetch_optional(&**conn)
    .await?;

    Ok(result.is_some())
}
