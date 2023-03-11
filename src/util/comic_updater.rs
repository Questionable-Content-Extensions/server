use std::convert::TryInto;

use crate::models::{ComicId, ImageType};
use crate::util::NewsUpdater;
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use const_format::concatcp;
use database::models::Comic as DatabaseComic;
use database::DbPool;
use futures::{select, FutureExt};
use ilyvion_util::string_extensions::StrExtensions;
use log::info;
use reqwest::Client;
use scraper::{Html, Selector};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration as StdDuration};

const FRONT_PAGE_URL: &str = "https://questionablecontent.net/";
const ARCHIVE_URL: &str = concatcp!(FRONT_PAGE_URL, "archive.php");
const STARTUP_DELAY_DURATION: StdDuration = StdDuration::from_secs(15);

pub struct ComicUpdater {
    client: Client,
}

impl ComicUpdater {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
        }
    }

    #[allow(clippy::too_many_lines)]
    pub async fn background_comic_updater(
        &self,
        db_pool: &DbPool,
        news_updater: &NewsUpdater,
        shutdown_receiver: &mut broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        // Wait a short period of time to avoid hammering the website on frequent restarts due to some
        // unresolved startup panic.
        sleep(STARTUP_DELAY_DURATION).await;

        loop {
            let now = Utc::now();
            info!(
                "Fetching data for the comic on {}.",
                now.format("%A, %d %B %Y")
            );

            let comic_id = self.fetch_latest_comic_data(db_pool).await?;
            news_updater.check_for(comic_id);

            let delay = time_until_next_update(now);
            let hours = delay.num_hours();
            let minutes = delay.num_minutes() - (hours * 60);
            info!(
                "Waiting for {} hours and {} minutes until doing another update.",
                hours, minutes
            );

            #[allow(clippy::mut_mut)]
            {
                select! {
                    _ = sleep(delay.to_std().expect("valid delay")).fuse() => {},
                    _ = shutdown_receiver.recv().fuse() => {
                        info!("Shutting down background comic updater");
                        break;
                    },
                };
            }
        }

        Ok(())
    }

    #[allow(clippy::too_many_lines)]
    async fn fetch_latest_comic_data(&self, db_pool: &DbPool) -> Result<ComicId> {
        info!("Fetching QC front page");
        let response = self.client.get(FRONT_PAGE_URL).send().await;
        let qc_front_page = match response {
            Err(e) => {
                anyhow::bail!(
                    "Could not fetch front page, got HTTP status {}",
                    e.status()
                        .map_or_else(|| String::from("(Unknown)"), |s| s.to_string())
                );
            }
            Ok(r) => r.text().await?,
        };

        if qc_front_page.trim().is_empty() {
            anyhow::bail!("Could not fetch front page, got empty response");
        }

        let (comic_id, image_type) = {
            let document = Html::parse_document(&qc_front_page);

            let comic_image_selector =
                Selector::parse("img[src*=\"/comics/\"]").expect("valid selector");
            let comic_image = document
                .select(&comic_image_selector)
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!("Could not fetch front page, couldn't find comic image element")
                })?;
            let comic_image_url = comic_image.value().attr("src").ok_or_else(|| {
                anyhow!("Could not fetch front page, couldn't get comic image element source")
            })?;

            let (comic_image, comic_type) = comic_image_url
                .rsplit_once('/')
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not fetch front page, couldn't find '/' in comic image source"
                    )
                })?
                .1
                .split_once('.')
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not fetch front page, couldn't find '.' in comic image source"
                    )
                })?;

            let comic_image_type = match comic_type.to_ascii_lowercase_cow().as_ref() {
                "png" => 1,
                "gif" => 2,
                "jpg" | "jpeg" => 3,
                _ => 0,
            };

            let comic_image = comic_image.parse().context(
                "Could not fetch front page, couldn't parse comic id from comic image source",
            )?;

            (comic_image, comic_image_type)
        };

        info!(
            "Comic on front page is #{} ({:?}), uploaded at approximately {}",
            comic_id,
            ImageType::from(image_type),
            Utc::now(),
        );

        let mut transaction = db_pool.begin().await?;

        let (needs_title, needs_image_type, current_comic_date) =
            DatabaseComic::needs_updating_by_id(&mut *transaction, comic_id).await?;

        info!(
            "Comic #{} needs title: {}, needs image type: {}",
            comic_id, needs_title, needs_image_type
        );

        let new_title = if needs_title {
            info!("Fetching QC archive page");
            let response = self.client.get(ARCHIVE_URL).send().await;
            let qc_archive_page = match response {
                Err(e) => {
                    anyhow::bail!(
                        "Could not fetch archive page, got HTTP status {}",
                        e.status()
                            .map_or_else(|| String::from("(Unknown)"), |s| s.to_string())
                    );
                }
                Ok(r) => r.text().await?,
            };

            let document = Html::parse_document(&qc_archive_page);
            let comic_title_selector = format!("a[href*=\"comic={}\"]", comic_id);
            let comic_title_selector =
                Selector::parse(&comic_title_selector).expect("valid comic title selector");
            let comic_title = document
                .select(&comic_title_selector)
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not fetch archive page, couldn't find comic title element",
                    )
                })?;
            let comic_title = comic_title.inner_html();
            let comic_title = comic_title
                .split_once(':')
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not fetch archive page, couldn't find ':' in comic title",
                    )
                })?
                .1
                .trim();

            Some(String::from(comic_title))
        } else {
            None
        };

        let comic_date = current_comic_date.map_or_else(Utc::now, |d| Utc.from_utc_datetime(&d));

        if let Some(title) = new_title {
            info!(
                "Setting comic #{}'s title to '{}' and image type to '{:?}'",
                comic_id,
                title,
                ImageType::from(image_type)
            );

            DatabaseComic::insert_or_update_title_imagetype_and_publish_date_by_id(
                &mut *transaction,
                comic_id,
                &title,
                image_type,
                comic_date.naive_utc(),
            )
            .await?;
        } else if needs_image_type && image_type != 0 {
            info!(
                "Setting comic #{}'s image type to '{:?}'",
                comic_id,
                ImageType::from(image_type)
            );

            DatabaseComic::update_image_type_and_publish_date_by_id(
                &mut *transaction,
                comic_id,
                image_type,
                comic_date.naive_utc(),
            )
            .await?;
        } else if current_comic_date.is_none() {
            DatabaseComic::update_publish_date_by_id(
                &mut *transaction,
                comic_id,
                comic_date,
                false,
            )
            .await?;
        }

        info!("Saving any changes to the database.");
        transaction.commit().await?;

        comic_id
            .try_into()
            .map_err(|_| anyhow!("comic id extracted from front page is invalid"))
    }
}

fn time_until_next_update(now: DateTime<Utc>) -> Duration {
    let weekday = now.weekday();
    let time = now.time();
    let hour = time.hour();
    match weekday {
        Weekday::Sat | Weekday::Sun => {
            if time < NaiveTime::from_hms(12, 0, 0) {
                NaiveTime::from_hms(12, 0, 0) - time
            } else {
                NaiveTime::from_hms(23, 59, 59) - time + Duration::seconds(1)
            }
        }
        _ => {
            if hour < 23 {
                NaiveTime::from_hms(hour + 1, 0, 0) - time
            } else {
                NaiveTime::from_hms(23, 59, 59) - time + Duration::seconds(1)
            }
        }
    }
}
