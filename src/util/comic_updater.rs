use std::convert::TryInto;

use crate::database::DbPool;
use crate::models::{ComicId, ImageType};
use crate::util::{Environment, NewsUpdater};
use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Datelike, Duration, NaiveTime, TimeZone, Timelike, Utc, Weekday};
use chrono_tz::Tz;
use const_format::concatcp;
use ego_tree::NodeRef;
use futures::{select, FutureExt};
use ilyvion_util::string_extensions::StrExtensions;
use log::{info, warn};
use once_cell::sync::Lazy;
use reqwest::Client;
use scraper::{ElementRef, Html, Node, Selector};
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration as StdDuration};

const FRONT_PAGE_URL: &str = "https://questionablecontent.net/";
const ARCHIVE_URL: &str = concatcp!(FRONT_PAGE_URL, "archive.php");
const STARTUP_DELAY_DURATION: StdDuration = StdDuration::from_secs(15);

static TIME_ZONE: Lazy<Tz> =
    Lazy::new(|| Environment::qc_timezone().parse().expect("valid timezone"));

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
    ) -> sqlx::Result<()> {
        // Wait a short period of time to avoid hammering the website on frequent restarts due to some
        // unresolved startup panic.
        sleep(STARTUP_DELAY_DURATION).await;

        loop {
            let now = Utc::now();
            info!(
                "Fetching data for the comic on {}.",
                now.format("%A, %d %B %Y")
            );

            match self.fetch_latest_comic_data(db_pool).await {
                Err(e) => warn!("{}", e),
                Ok(comic_id) => news_updater.check_for(comic_id).await,
            }

            let delay = time_until_next_update(now);
            let hours = delay.num_hours();
            let minutes = delay.num_minutes() - (hours * 60);
            info!("Waiting for {}:{} until next update.", hours, minutes);

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
        #[allow(nonstandard_style)]
        struct NeedsQuery {
            title: String,
            ImageType: i32,
        }

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

        let (comic_id, image_type, comic_date) = {
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

            let (comic_image, comic_type): (i16, i32) = {
                let (comic_image, comic_type) = comic_image_url
                    .rsplit_once("/")
                    .ok_or_else(|| {
                        anyhow::anyhow!(
                            "Could not fetch front page, couldn't find '/' in comic image source"
                        )
                    })?
                    .1
                    .split_once(".")
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

            let news_selector = Selector::parse("#newspost").expect("valid selector");
            let news_element = document.select(&news_selector).next().ok_or_else(|| {
                anyhow::anyhow!("Could not fetch front page, couldn't find news element")
            })?;

            let mut news_node = NodeRef::clone(&news_element);
            let news_previous_sibling = ElementRef::wrap(
                loop {
                    let ps = news_node.prev_sibling();
                    if let Some(prev_news_node) = ps {
                        if let Node::Element(_) = prev_news_node.value() {
                            break Some(prev_news_node);
                        }
                        news_node = prev_news_node;
                    } else {
                        break None;
                    }
                }
                .ok_or_else(|| {
                    anyhow::anyhow!(
                        "Could not fetch front page, couldn't get news element previous sibling"
                    )
                })?,
            )
            .expect("previous news sibling");

            let date_selector = Selector::parse("b").expect("valid selector");
            let date = news_previous_sibling
                .select(&date_selector)
                .next()
                .ok_or_else(|| {
                    anyhow::anyhow!("Could not fetch front page, couldn't find date element")
                })?;

            // "July 6, 2021 9:11pm"
            let comic_date = TIME_ZONE
                .datetime_from_str(&date.inner_html(), "%B %e, %Y %l:%M%P")
                .context("Could not fetch front page, couldn't parse date")?
                .with_timezone(&Utc);

            (comic_image, comic_type, comic_date)
        };

        info!(
            "Comic on front page is #{} ({:?}), uploaded at {}",
            comic_id,
            ImageType::from(image_type),
            comic_date
        );

        let mut transaction = db_pool.begin().await?;

        let (needs_title, needs_image_type) = if let Some(needs) = sqlx::query_as!(
            NeedsQuery,
            r#"
                SELECT title, ImageType FROM `comic`
                WHERE `id` = ?
            "#,
            comic_id
        )
        .fetch_optional(&mut *transaction)
        .await?
        {
            (needs.title.is_empty(), needs.ImageType == 0)
        } else {
            (true, true)
        };

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
        if let Some(title) = new_title {
            info!(
                "Setting comic #{}'s title to '{}' and image type to '{:?}'",
                comic_id,
                title,
                ImageType::from(image_type)
            );

            sqlx::query!(
                r#"
                    INSERT INTO `comic`
                        (id, title, imagetype)
                    VALUES
                        (?, ?, ?)
                    ON DUPLICATE KEY UPDATE
                        title = ?,
                        imagetype = ?
                "#,
                comic_id,
                title,
                image_type,
                title,
                image_type,
            )
            .execute(&mut *transaction)
            .await?;
        } else if needs_image_type && image_type != 0 {
            info!(
                "Setting comic #{}'s image type to '{:?}'",
                comic_id,
                ImageType::from(image_type)
            );

            sqlx::query!(
                r#"
                    UPDATE `comic`
                    SET
                        ImageType = ?
                    WHERE id = ?
                "#,
                image_type,
                comic_id,
            )
            .execute(&mut *transaction)
            .await?;
        }

        info!("Saving any changes to the database.");
        transaction.commit().await?;

        Ok(comic_id
            .try_into()
            .map_err(|_| anyhow!("comic id extracted from front page is invalid"))?)
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
            if hour < 3 {
                NaiveTime::from_hms(hour + 1, 0, 0) - time
            } else if hour < 6 {
                NaiveTime::from_hms(6, 0, 0) - time
            } else if hour < 12 {
                NaiveTime::from_hms(12, 0, 0) - time
            } else if hour < 18 {
                NaiveTime::from_hms(18, 0, 0) - time
            } else if hour < 21 {
                NaiveTime::from_hms(21, 0, 0) - time
            } else if hour < 23 {
                NaiveTime::from_hms(hour + 1, 0, 0) - time
            } else {
                NaiveTime::from_hms(23, 59, 59) - time + Duration::seconds(1)
            }
        }
    }
}
