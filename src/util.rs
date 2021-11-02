use actix_web_grants::permissions::{AuthDetails, PermissionsCheck};
use anyhow::{anyhow, Result};
use futures::Future;
use ilyvion_util::string_extensions::StrExtensions;
use once_cell::sync::Lazy;
use semval::{Invalidity, Validate};
use std::cell::RefCell;
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

struct InvalidityFormatter<I: IntoIterator<Item = V>, V: Invalidity + Display> {
    invalidations: RefCell<Option<I>>,
}

impl<I: IntoIterator<Item = V>, V: Invalidity + Display> From<I> for InvalidityFormatter<I, V> {
    #[inline]
    fn from(iterator: I) -> Self {
        Self {
            invalidations: RefCell::new(Some(iterator)),
        }
    }
}

impl<I: IntoIterator<Item = V>, V: Invalidity + Display> Display for InvalidityFormatter<I, V> {
    #[inline]
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let invalidations = self
            .invalidations
            .borrow_mut()
            .take()
            .expect("this method should only ever get called once");

        writeln!(f, "Validation failed: ")?;
        for v in invalidations {
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
        return Err(anyhow!("{}", InvalidityFormatter::from(c)));
    }
    Ok(())
}
