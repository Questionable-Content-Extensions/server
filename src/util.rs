use crate::database::models::Token;
use crate::models::token_permissions;
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use anyhow::anyhow;
use chrono::Utc;
use ilyvion_util::string_extensions::StrExtensions;
use log::info;
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
        dotenv::dotenv().ok();
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

pub async fn log_action<'e, 'c: 'e, E>(
    executor: E,
    token: Uuid,
    action: impl AsRef<str>,
) -> sqlx::Result<()>
where
    E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
{
    let action = action.as_ref();
    sqlx::query!(
        r#"
            INSERT INTO `log_entry`
                (UserToken, DateTime, Action)
            VALUES
                (?, ?, ?)
        "#,
        token.to_string(),
        Utc::now().naive_utc(),
        action,
    )
    .execute(executor)
    .await?;

    info!("{}", action);

    Ok(())
}

pub async fn get_permissions_for_token<'e, 'c: 'e, E>(
    executor: E,
    token: uuid::Uuid,
) -> sqlx::Result<Vec<String>>
where
    E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
{
    let result = sqlx::query_as!(
        Token,
        r#"
            SELECT * FROM `token`
            WHERE `id` = ?
        "#,
        token.to_string()
    )
    .fetch_optional(executor)
    .await?;

    let token = if let Some(token) = result {
        token
    } else {
        // Invalid token provided, there are no permissions
        return Ok(vec![]);
    };

    let mut permissions = Vec::with_capacity(7);
    permissions.push(token_permissions::HAS_VALID_TOKEN.to_string());
    if token.CanAddItemToComic != 0 {
        permissions.push(token_permissions::CAN_ADD_ITEM_TO_COMIC.to_string());
    }
    if token.CanRemoveItemFromComic != 0 {
        permissions.push(token_permissions::CAN_REMOVE_ITEM_FROM_COMIC.to_string());
    }
    if token.CanChangeComicData != 0 {
        permissions.push(token_permissions::CAN_CHANGE_COMIC_DATA.to_string());
    }
    if token.CanAddImageToItem != 0 {
        permissions.push(token_permissions::CAN_ADD_IMAGE_TO_ITEM.to_string());
    }
    if token.CanRemoveImageFromItem != 0 {
        permissions.push(token_permissions::CAN_REMOVE_IMAGE_FROM_ITEM.to_string());
    }
    if token.CanChangeItemData != 0 {
        permissions.push(token_permissions::CAN_CHANGE_ITEM_DATA.to_string());
    }
    Ok(permissions)
}

#[inline]
pub fn ensure_is_authorized(auth: &AuthDetails, permission: &str) -> anyhow::Result<()> {
    if !auth.has_permission(permission) {
        return Err(anyhow!("Invalid token or insufficient permissions"));
    }

    Ok(())
}
