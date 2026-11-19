use actix_web_grants::authorities::{AuthDetails, AuthoritiesCheck};
use anyhow::{anyhow, Result};
use chrono::{DateTime, Datelike, TimeZone, Timelike, Utc};
use futures::{Future, Stream, StreamExt};
use ilyvion_util::chrono::days_from_month_in_year;
use semval::{Invalidity, Validate};
use std::cell::RefCell;
use std::fmt::Display;
use std::pin::Pin;
use std::task::{Context, Poll};

pub use comic_updater::*;
pub use comic_updater_trigger::ComicUpdaterTrigger;
pub use news_updater::*;
pub use token_cache::TokenPermissionsCache;

mod comic_updater;
mod comic_updater_trigger;
mod news_updater;
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

pub(crate) trait AddMonths {
    fn add_months(self, add_months: u32) -> Self;
}

impl AddMonths for DateTime<Utc> {
    fn add_months(self, mut add_months: u32) -> Self {
        let mut year = self.year();
        let mut month = self.month();
        let mut day = self.day();

        // Add whole years first
        while add_months >= 12 {
            year += 1;
            add_months -= 12;
        }

        // Check if remaining months add up to cross another year boundary
        month = if month + add_months > 12 {
            year += 1;
            (month + add_months) % 12
        } else {
            month + add_months
        };

        // Check if the day we have is bigger than the biggest day of the resulting month
        // and if so, truncate
        let days_in_month = u32::try_from(days_from_month_in_year(month, year).unwrap())
            .expect("days in month are always 28-31");
        if day > days_in_month {
            day = days_in_month;
        }

        Utc.with_ymd_and_hms(year, month, day, self.hour(), self.minute(), self.second())
            .unwrap()
    }
}

#[test]
fn test_add_month() {
    let jan012000 = Utc.with_ymd_and_hms(2000, 1, 1, 0, 0, 0).unwrap();
    for m in 1..12 {
        let result = jan012000.add_months(m);
        assert_eq!(result.month(), m + 1);
    }

    let jan312000 = Utc.with_ymd_and_hms(2000, 1, 31, 0, 0, 0).unwrap();
    let result = jan312000.add_months(1);
    assert_eq!(result.month(), 2);
    assert_eq!(result.day(), 29);
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
    use actix_web::http::header::{HeaderMap, HeaderValue, CONTENT_TYPE};
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
