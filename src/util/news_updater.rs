use crate::models::ComicId;
use anyhow::Result;
use chrono::Utc;
use database::DbPool;
use database::models::{Comic, News};
use futures::{FutureExt, select};
use regex::Regex;
use reqwest::Client;
use scraper::{Html, Selector};
use std::collections::HashSet;
use std::sync::Arc;
use std::sync::Mutex;
use tokio::sync::broadcast;
use tokio::time::{Duration, sleep};
use tracing::{Instrument, debug, info, info_span, warn};

const QC_COMIC_URL_BASE: &str = "https://questionablecontent.net/view.php?comic=";
const TASK_DELAY_TIME: Duration = Duration::from_secs(5);

static REMOVE_NEWLINES: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"\r|\n").expect("valid regex"));
static REPLACE_HTML_NEWLINES: std::sync::LazyLock<Regex> =
    std::sync::LazyLock::new(|| Regex::new(r"<br\s*/?>").expect("valid regex"));

#[derive(Debug)]
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
        self.update_set.lock().unwrap().insert(comic_id);
    }

    pub async fn background_news_updater(
        &self,
        db_pool: &DbPool,
        shutdown_receiver: &mut broadcast::Receiver<()>,
    ) -> anyhow::Result<()> {
        loop {
            let update_entries = self.get_pending_update_entries();
            debug!("There are {} news updates pending.", update_entries.len());

            if !update_entries.is_empty() {
                info!("Running background news update...");
                self.run_news_update(db_pool, update_entries.iter()).await?;
            }

            self.remove_pending_update_entries(&update_entries);

            {
                select! {
                    () = sleep(TASK_DELAY_TIME).fuse() => {},
                    _ = shutdown_receiver.recv().fuse() => {
                        info!("Shutting down background news updater");
                        break;
                    },
                };
            }
        }

        Ok(())
    }

    #[tracing::instrument(skip(db_pool))]
    async fn run_news_update(
        &self,
        db_pool: &DbPool,
        update_entries: impl Iterator<Item = &ComicId> + std::fmt::Debug,
    ) -> Result<()> {
        let mut transaction = db_pool
            .begin()
            .instrument(info_span!("Pool::begin"))
            .await?;
        for comic_id in update_entries.copied() {
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
                        Utc::now().date_naive(),
                    )
                    .await?;
                } else {
                    info!(
                        "News text for comic #{} has changed. Resetting update factor and updating text.",
                        comic_id
                    );
                    News::update_by_comic_id(
                        &mut *transaction,
                        comic_id.into_inner(),
                        &news_text,
                        1.0,
                        Utc::now().date_naive(),
                    )
                    .await?;
                }
            } else {
                info!(
                    "News text for comic #{} has changed. Resetting update factor and updating text.",
                    comic_id
                );
                News::create_for_comic_id(
                    &mut *transaction,
                    comic_id.into_inner(),
                    &news_text,
                    1.0,
                    Utc::now().date_naive(),
                )
                .await?;
            }

            // Take a short break after a news update to not hammer the server.
            sleep(Duration::from_millis(500)).await;
        }

        info!("Saving any changes to the news to the database.");
        transaction.commit().await?;

        Ok(())
    }

    pub fn get_pending_update_entries(&self) -> HashSet<ComicId> {
        self.update_set.lock().unwrap().clone()
    }

    pub fn remove_pending_update_entries(&self, updated_entries: &HashSet<ComicId>) {
        self.update_set
            .lock()
            .unwrap()
            .retain(|e| !updated_entries.contains(e));
    }

    #[tracing::instrument]
    pub async fn fetch_news_for(&self, comic_id: ComicId) -> Result<String> {
        let url = format!("{QC_COMIC_URL_BASE}{comic_id}");
        let response = self
            .client
            .get(url)
            .send()
            .instrument(info_span!("fetch_comic_page", ?comic_id))
            .await;
        let qc_page = match response {
            Err(e) => {
                anyhow::bail!(
                    "Could not fetch news for #{}, got HTTP status {}",
                    comic_id,
                    e.status()
                        .map_or_else(|| String::from("(Unknown)"), |s| s.to_string())
                );
            }
            Ok(r) => {
                r.text()
                    .instrument(info_span!("fetch_comic_page_text", ?comic_id))
                    .await?
            }
        };

        if qc_page.trim().is_empty() {
            anyhow::bail!("Could not fetch news for #{comic_id}, got empty response");
        }

        let parse_document_span = info_span!("parse_comic_page_document", ?comic_id);
        let news_inner_html = parse_document_span.in_scope(|| -> Result<String> {
            let document = Html::parse_document(&qc_page);
            let news_selector = Selector::parse("#news,#newspost").expect("valid selector");
            let news = document.select(&news_selector).next().ok_or_else(|| {
                anyhow::anyhow!(
                    "Could not fetch news for #{comic_id}, couldn't find #news or #newspost element"
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
                }
                news_inner_html = trimmed_news_inner_html;
            }

            let news_inner_html = REMOVE_NEWLINES.replace_all(news_inner_html, "");
            let news_inner_html = REPLACE_HTML_NEWLINES.replace_all(&news_inner_html, "\n");
            let news_inner_html = news_inner_html.trim();

            Ok(String::from(news_inner_html))
        })?;
        Ok(news_inner_html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn comic(id: u16) -> ComicId {
        ComicId::from_trusted(id)
    }

    #[test]
    fn check_for_inserts_into_pending_set() {
        let updater = NewsUpdater::new();
        updater.check_for(comic(1));
        updater.check_for(comic(2));
        let entries = updater.get_pending_update_entries();
        assert!(entries.contains(&comic(1)));
        assert!(entries.contains(&comic(2)));
        assert_eq!(entries.len(), 2);
    }

    #[test]
    fn check_for_is_idempotent() {
        let updater = NewsUpdater::new();
        updater.check_for(comic(5));
        updater.check_for(comic(5));
        assert_eq!(updater.get_pending_update_entries().len(), 1);
    }

    #[test]
    fn remove_pending_update_entries_removes_only_processed() {
        let updater = NewsUpdater::new();
        updater.check_for(comic(1));
        updater.check_for(comic(2));
        updater.check_for(comic(3));
        let processed = [comic(1), comic(3)].into_iter().collect::<HashSet<_>>();
        updater.remove_pending_update_entries(&processed);
        let remaining = updater.get_pending_update_entries();
        assert!(!remaining.contains(&comic(1)));
        assert!(remaining.contains(&comic(2)));
        assert!(!remaining.contains(&comic(3)));
    }
}
