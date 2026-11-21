use actix_web_grants::authorities::{AuthDetails, AuthoritiesCheck};
use anyhow::{Result, anyhow};
use database::models::ITEM_SHORT_NAME_MAX_CHARS;
use futures::{Future, Stream, StreamExt};
use semval::context::Context as ValidationContext;
use semval::{Invalidity, Validate};
use std::borrow::Cow;
use std::cell::RefCell;
use std::fmt::Display;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use comic_updater::*;
pub use comic_updater_trigger::ComicUpdaterTrigger;
pub use news_updater::*;
pub use server_error_body::hide_server_error_details;
pub use token_cache::TokenPermissionsCache;

mod comic_updater;
mod comic_updater_trigger;
mod news_updater;
mod server_error_body;
mod token_cache;

pub mod environment {
    use ilyvion_util::environment::define_environment;

    define_environment! {
        pub port(): u16;
        pub database_url();
        pub database_max_connections(): u32;
        pub database_min_connections(): u32;
        pub qc_timezone();
        pub background_services(): bool;
        pub honeycomb_key();
    }
}

#[inline]
pub fn ensure_is_authorized(auth: &AuthDetails, permission: &str) -> anyhow::Result<()> {
    if !auth.has_authority(permission) {
        return Err(anyhow!("Invalid token or insufficient permissions"));
    }

    Ok(())
}

/// Largest image the image-upload endpoints accept, in bytes.
pub const MAX_IMAGE_BYTES: usize = 5 * 1024 * 1024;

/// Collects a byte stream into a `Vec`, failing with `413 Payload Too Large` as
/// soon as the collected size would exceed `max_bytes`.
pub async fn read_capped<S, B, E>(mut stream: S, max_bytes: usize) -> actix_web::Result<Vec<u8>>
where
    S: Stream<Item = Result<B, E>> + Unpin,
    B: AsRef<[u8]>,
    E: Into<actix_web::Error>,
{
    let mut data = Vec::new();
    while let Some(chunk) = stream.next().await {
        let chunk = chunk.map_err(Into::into)?;
        let chunk = chunk.as_ref();
        if data.len() + chunk.len() > max_bytes {
            return Err(actix_web::error::ErrorPayloadTooLarge(anyhow!(
                "Image exceeds the {max_bytes} byte limit"
            )));
        }
        data.extend_from_slice(chunk);
    }
    Ok(data)
}

#[derive(Debug, Clone)]
pub enum Either<A, B> {
    /// First branch of the type
    Left(A),
    /// Second branch of the type
    Right(B),
}

impl<A, B> Either<A, B> {
    #[expect(unsafe_code)]
    const fn project(self: Pin<&mut Self>) -> Either<Pin<&mut A>, Pin<&mut B>> {
        unsafe {
            match self.get_unchecked_mut() {
                Self::Left(a) => Either::Left(Pin::new_unchecked(a)),
                Self::Right(b) => Either::Right(Pin::new_unchecked(b)),
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
            writeln!(f, "* {v}")?;
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

/// A text field longer than its database column allows.
#[derive(Copy, Clone, Debug, parse_display::Display, Eq, PartialEq)]
#[display("{field} must be at most {max_chars} characters")]
pub struct TextTooLong {
    pub field: &'static str,
    pub max_chars: usize,
}

impl TextTooLong {
    /// `Some` if `value` has more than `max_chars` characters.
    pub fn check(field: &'static str, value: Option<&str>, max_chars: usize) -> Option<Self> {
        value
            .is_some_and(|value| has_more_chars_than(value, max_chars))
            .then_some(Self { field, max_chars })
    }

    /// Collects the results of [`TextTooLong::check`] into a validation context.
    pub fn collect(checks: impl IntoIterator<Item = Option<Self>>) -> ValidationContext<Self> {
        checks
            .into_iter()
            .flatten()
            .fold(ValidationContext::new(), ValidationContext::invalidate)
    }
}

/// The short name a new item gets from its name: the name itself, or, if the
/// name is longer than `Item.short_name` allows, its start ending in `…`.
pub fn short_name_from_name(name: &str) -> Cow<'_, str> {
    if !has_more_chars_than(name, ITEM_SHORT_NAME_MAX_CHARS) {
        return Cow::Borrowed(name);
    }
    let kept: String = name.chars().take(ITEM_SHORT_NAME_MAX_CHARS - 1).collect();
    Cow::Owned(kept + "…")
}

/// Counts characters (Unicode scalar values) as `MariaDB` does for `utf8mb4`
/// `varchar` columns.
fn has_more_chars_than(value: &str, max_chars: usize) -> bool {
    value.chars().nth(max_chars).is_some()
}

/// Replaces the last comma in a string with `and` (if there's a single comma)
/// or with `, and` if there's multiple.
pub fn andify_comma_string(changed: &mut String) {
    let last_index = changed.char_indices().rev().find(|&(_, c)| c == ',');
    if let Some((last_comma_index, _)) = last_index {
        // We can unwrap here because we already know there's at least
        // one comma in this text.
        let (first_comma_index, _) = changed.char_indices().find(|&(_, c)| c == ',').unwrap();
        if first_comma_index == last_comma_index {
            // Only a single comma. Replace with 'and'
            changed.replace_range(last_comma_index..=last_comma_index, " and");
        } else {
            // Multiple commas. Replace with ', and'
            changed.replace_range(last_comma_index..=last_comma_index, ", and");
        }
    }
}

#[cfg(test)]
pub(crate) mod test_support {
    use actix_multipart::Multipart;
    use actix_web::error::PayloadError;
    use actix_web::http::header::{CONTENT_TYPE, HeaderMap, HeaderValue};
    use actix_web::web::Bytes;

    const BOUNDARY: &str = "qcext-test-boundary";

    /// Builds a `Multipart` stream from `(field name, value)` pairs, delivering
    /// the body in 64 KiB chunks.
    pub(crate) fn multipart_from_fields(fields: &[(&str, &[u8])]) -> Multipart {
        let mut body = Vec::new();
        for (name, value) in fields {
            body.extend_from_slice(
                format!("--{BOUNDARY}\r\nContent-Disposition: form-data; name=\"{name}\"\r\n\r\n")
                    .as_bytes(),
            );
            body.extend_from_slice(value);
            body.extend_from_slice(b"\r\n");
        }
        body.extend_from_slice(format!("--{BOUNDARY}--\r\n").as_bytes());

        let mut headers = HeaderMap::new();
        headers.insert(
            CONTENT_TYPE,
            HeaderValue::from_str(&format!("multipart/form-data; boundary={BOUNDARY}"))
                .expect("valid header value"),
        );

        let body = Bytes::from(body);
        let chunks = (0..body.len()).step_by(64 * 1024).map(move |start| {
            Ok::<_, PayloadError>(body.slice(start..body.len().min(start + 64 * 1024)))
        });
        Multipart::new(&headers, futures::stream::iter(chunks))
    }
}

#[cfg(test)]
mod text_length_tests {
    use super::*;

    #[test]
    fn text_at_the_limit_is_accepted() {
        assert_eq!(TextTooLong::check("name", Some(&"a".repeat(5)), 5), None);
    }

    #[test]
    fn text_over_the_limit_is_rejected_with_the_field_and_limit() {
        let invalidity = TextTooLong::check("name", Some(&"a".repeat(6)), 5).unwrap();

        assert_eq!(invalidity.to_string(), "name must be at most 5 characters");
    }

    #[test]
    fn text_length_counts_characters_not_bytes() {
        assert_eq!(TextTooLong::check("name", Some("ææææø"), 5), None);
    }

    #[test]
    fn absent_text_is_accepted() {
        assert_eq!(TextTooLong::check("name", None, 0), None);
    }

    #[test]
    fn short_name_from_name_keeps_a_name_that_fits() {
        let name = "ø".repeat(ITEM_SHORT_NAME_MAX_CHARS);

        assert_eq!(short_name_from_name(&name), name);
    }

    #[test]
    fn short_name_from_name_ends_an_overlong_name_in_an_ellipsis() {
        let name = format!("{}xyz", "ø".repeat(ITEM_SHORT_NAME_MAX_CHARS - 1));

        let short_name = short_name_from_name(&name);

        assert_eq!(
            short_name,
            format!("{}…", "ø".repeat(ITEM_SHORT_NAME_MAX_CHARS - 1))
        );
        assert_eq!(short_name.chars().count(), ITEM_SHORT_NAME_MAX_CHARS);
    }
}

#[cfg(test)]
mod read_capped_tests {
    use super::*;
    use actix_web::http::StatusCode;

    fn chunks(
        sizes: &'static [usize],
    ) -> impl Stream<Item = Result<Vec<u8>, actix_web::Error>> + Unpin {
        futures::stream::iter(sizes.iter().map(|&n| Ok(vec![0xAB; n])))
    }

    #[tokio::test]
    async fn collects_everything_when_exactly_at_limit() {
        let data = read_capped(chunks(&[4, 6]), 10).await.unwrap();

        assert_eq!(data, vec![0xAB; 10]);
    }

    #[tokio::test]
    async fn rejects_with_payload_too_large_when_one_byte_over_limit() {
        let err = read_capped(chunks(&[4, 7]), 10).await.unwrap_err();

        assert_eq!(err.error_response().status(), StatusCode::PAYLOAD_TOO_LARGE);
    }

    #[tokio::test]
    async fn stops_reading_at_the_first_chunk_over_the_limit() {
        let mut stream = chunks(&[6, 6, 6]);

        read_capped(&mut stream, 10).await.unwrap_err();

        assert!(stream.next().await.is_some());
    }

    #[tokio::test]
    async fn propagates_stream_errors() {
        let stream = futures::stream::iter(vec![Err::<Vec<u8>, _>(
            actix_web::error::ErrorBadRequest("broken"),
        )]);

        let err = read_capped(stream, 10).await.unwrap_err();

        assert_eq!(err.error_response().status(), StatusCode::BAD_REQUEST);
    }
}
