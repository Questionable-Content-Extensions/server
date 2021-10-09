use crate::database::models::Token as DatabaseToken;
use crate::models::{token_permissions, Token};
use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use anyhow::anyhow;
use chrono::Utc;
use futures::Future;
use ilyvion_util::string_extensions::StrExtensions;
use log::info;
use once_cell::sync::Lazy;
use semval::context::Context as ValidationContext;
use semval::{Invalidity, Validate};
use std::fmt::Display;
use std::pin::Pin;
use std::task::{Context, Poll};

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
    token: Token,
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
    token: Token,
) -> sqlx::Result<Vec<String>>
where
    E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
{
    let result = sqlx::query_as!(
        DatabaseToken,
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

#[derive(Debug, Clone)]
pub enum Either<A, B> {
    /// First branch of the type
    Left(A),
    /// Second branch of the type
    Right(B),
}

impl<A, B> Either<A, B> {
    #[allow(unsafe_code)]
    fn project(self: Pin<&mut Self>) -> Either<Pin<&mut A>, Pin<&mut B>> {
        unsafe {
            match self.get_unchecked_mut() {
                Either::Left(a) => Either::Left(Pin::new_unchecked(a)),
                Either::Right(b) => Either::Right(Pin::new_unchecked(b)),
            }
        }
    }
}

impl<A, B> Future for Either<A, B>
where
    A: Future,
    B: Future,
{
    type Output = Either<A::Output, B::Output>;

    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match self.project() {
            Either::Left(x) => x.poll(cx).map(Either::Left),
            Either::Right(x) => x.poll(cx).map(Either::Right),
        }
    }
}

struct ValidationContextErrorFormatter<V: Invalidity + Display> {
    invalidations: Vec<V>,
}

impl<V: Invalidity + Display> From<ValidationContext<V>> for ValidationContextErrorFormatter<V> {
    #[inline]
    fn from(validation_context: ValidationContext<V>) -> Self {
        Self {
            invalidations: validation_context.into_iter().collect(),
        }
    }
}

impl<V: Invalidity + Display> Display for ValidationContextErrorFormatter<V> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Validation failed: ")?;
        for v in &self.invalidations {
            writeln!(f, "* {}", v)?;
        }

        Ok(())
    }
}

#[inline]
pub fn ensure_is_valid<V: Validate>(v: &V) -> Result<(), anyhow::Error>
where
    V::Invalidity: Display,
{
    if let Err(c) = v.validate() {
        return Err(anyhow!("{}", ValidationContextErrorFormatter::from(c)));
    }
    Ok(())
}
