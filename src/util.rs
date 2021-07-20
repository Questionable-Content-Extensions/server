use crate::database::DbPoolConnection;
use ilyvion_util::string_extensions::StrExtensions;
use once_cell::sync::Lazy;
use uuid::Uuid;

pub use comic_updater::*;
pub use news_updater::*;

mod comic_updater;
mod news_updater;

macro_rules! lazy_environment {
    ($static_name:ident, $name:ident) => {
        lazy_environment!(pub, $static_name, $name);
    };
    ($vis:vis, $static_name:ident, $name:ident) => {
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
            $vis fn $name() -> &'static str {
                &*$static_name
            }
        }
    };
}

pub struct Environment;

lazy_environment!(PORT, port);
lazy_environment!(DATABASE_URL, database_url);
lazy_environment!(QC_TIMEZONE, qc_timezone);
lazy_environment!(, BACKGROUND_SERVICES, background_services);

impl Environment {
    pub fn init() {
        dotenv::from_filename(".env").ok();
    }

    pub fn background_services_enabled() -> bool {
        !matches!(
            Self::background_services()
                .to_ascii_lowercase_cow()
                .as_ref(),
            "off" | "false" | "no" | "0"
        )
    }
}

pub async fn is_token_valid(conn: &mut DbPoolConnection, token: Uuid) -> Result<bool, sqlx::Error> {
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
    .fetch_optional(conn)
    .await?;

    Ok(result.is_some())
}
