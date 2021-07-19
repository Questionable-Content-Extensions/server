use crate::database::DbPool;
use once_cell::sync::Lazy;
use uuid::Uuid;

pub use comic_updater::*;
pub use news_updater::*;

mod comic_updater;
mod news_updater;

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
lazy_environment!(QC_TIMEZONE, qc_timezone);

impl Environment {
    pub fn init() {
        dotenv::from_filename(".env").ok();
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
