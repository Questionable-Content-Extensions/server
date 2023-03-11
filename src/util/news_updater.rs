use crate::models::ComicId;
use anyhow::Result;
use chrono::Utc;
use database::models::{Comic, News};
use database::DbPool;
use futures::lock::Mutex;
use futures::{select, FutureExt};
use log::{debug, info, warn};
use once_cell::sync::Lazy;
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use tokio::sync::broadcast;
use tokio::time::{sleep, Duration};

const QC_COMIC_URL_BASE: &str = "https://questionablecontent.net/view.php?comic=";
const TASK_DELAY_TIME: Duration = Duration::from_secs(5);

static REMOVE_NEWLINES: Lazy<Regex> = Lazy::new(|| Regex::new(r"\r|\n").expect("valid regex"));
static REPLACE_HTML_NEWLINES: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"<br\s*/?>").expect("valid regex"));

pub struct NewsUpdater {
    client: Client,
    update_set: Arc<Mutex<HashSet<ComicId>>>,
}

impl NewsUpdater {
    pub fn new() -> Self {
        Self {
            client: Client::new(),
            update_set: Arc::new(Mutex::new(HashSet::new())),
        }
    }

    pub fn check_for(&self, comic_id: ComicId) {
        info!("Scheduling a news update check for comic {}", comic_id);
        let update_set = Arc::clone(&self.update_set);
        tokio::task::spawn(async move {
            let mut update_set = update_set.lock().await;
            update_set.insert(comic_id);
        });
    }

    #[allow(clippy::too_many_lines)]
    pub async fn background_news_updater(
        &self,
        db_pool: &DbPool,
        shutdown_receiver: &mut broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            let update_entries = self.get_pending_update_entries().await;
            debug!("There are {} news updates pending.", update_entries.len());

            if !update_entries.is_empty() {
                info!("Running background news update...");

                let mut transaction = db_pool.begin().await?;
                for comic_id in update_entries.iter().copied() {
                    let comic_exists =
                        Comic::exists_by_id(&mut *transaction, comic_id.into_inner()).await?;

                    if !comic_exists {
                        info!(
                            "Cannot update news for comic {}; comic data does not yet exist.",
                            comic_id
                        );
                        continue;
                    }

                    let news = News::by_comic_id(&mut *transaction, comic_id.into_inner()).await?;

                    if let Some(news) = &news {
                        if !news.is_outdated() {
                            info!("News for comic #{} is not outdated.", comic_id);
                            continue;
                        }
                    }

                    info!("Fetching news in the background for comic #{}...", comic_id);
                    let news_text = match self.fetch_news_for(comic_id).await {
                        Ok(news_text) => news_text,
                        Err(e) => {
                            warn!("{}", e);
                            continue;
                        }
                    };

                    if let Some(news) = news {
                        // Old news. Compare news text with the old.
                        if news.news == news_text {
                            info!(
                                "News text for comic #{} is the same. Increasing update factor.",
                                comic_id
                            );
                            let new_update_factor = news.update_factor + 0.5;
                            News::update_last_updated_by_comic_id(
                                &mut *transaction,
                                comic_id.into_inner(),
                                new_update_factor,
                                Utc::today().naive_utc(),
                            )
                            .await?;
                        } else {
                            info!("News text for comic #{} has changed. Resetting update factor and updating text.",comic_id);
                            News::update_by_comic_id(
                                &mut *transaction,
                                comic_id.into_inner(),
                                &news_text,
                                1.0,
                                Utc::today().naive_utc(),
                            )
                            .await?;
                        }
                    } else {
                        info!("News text for comic #{} has changed. Resetting update factor and updating text.",comic_id);
                        News::create_for_comic_id(
                            &mut *transaction,
                            comic_id.into_inner(),
                            &news_text,
                            1.0,
                            Utc::today().naive_utc(),
                        )
                        .await?;
                    }
                }

                info!("Saving any changes to the news to the database.");
                transaction.commit().await?;
            }

            self.remove_pending_update_entries(update_entries).await;

            #[allow(clippy::mut_mut)]
            {
                select! {
                    _ = sleep(TASK_DELAY_TIME).fuse() => {},
                    _ = shutdown_receiver.recv().fuse() => {
                        info!("Shutting down background news updater");
                        break;
                    },
                };
            }
        }

        Ok(())
    }

    pub async fn get_pending_update_entries(&self) -> Vec<ComicId> {
        let update_set = self.update_set.lock().await;
        update_set.iter().copied().collect()
    }

    pub async fn remove_pending_update_entries(&self, update_entries: Vec<ComicId>) {
        let mut update_set = self.update_set.lock().await;
        update_set.retain(|e| !update_entries.contains(e));
    }

    pub async fn fetch_news_for(&self, comic_id: ComicId) -> Result<String> {
        let url = format!("{}{}", QC_COMIC_URL_BASE, comic_id);
        let response = self.client.get(url).send().await;
        let qc_page = match response {
            Err(e) => {
                anyhow::bail!(
                    "Could not fetch news for #{}, got HTTP status {}",
                    comic_id,
                    e.status()
                        .map_or_else(|| String::from("(Unknown)"), |s| s.to_string())
                );
            }
            Ok(r) => r.text().await?,
        };

        if qc_page.trim().is_empty() {
            anyhow::bail!("Could not fetch news for #{}, got empty response", comic_id);
        }

        let document = Html::parse_document(&qc_page);
        let news_selector = Selector::parse("#news,#newspost").expect("valid selector");
        let news = document.select(&news_selector).next().ok_or_else(|| {
            anyhow::anyhow!(
                "Could not fetch news for #{}, couldn't find #news or #newspost element",
                comic_id
            )
        })?;

        let news_inner_html = news.inner_html();
        let mut news_inner_html = &*news_inner_html;
        loop {
            let trimmed_news_inner_html = news_inner_html
                .trim()
                .trim_start_matches("<b></b>")
                .trim_start_matches("<br>");
            if trimmed_news_inner_html == news_inner_html {
                break;
            } else {
                news_inner_html = trimmed_news_inner_html;
            }
        }

        let news_inner_html = REMOVE_NEWLINES.replace_all(news_inner_html, "");
        let news_inner_html = REPLACE_HTML_NEWLINES.replace_all(&news_inner_html, "\n");
        let news_inner_html = news_inner_html.trim();

        Ok(String::from(news_inner_html))
    }
}
