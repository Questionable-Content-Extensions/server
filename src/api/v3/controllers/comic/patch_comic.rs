use crate::models::{ComicId, Token};
use crate::util::{andify_comma_string, ensure_is_authorized};
use actix_web::{error, web, HttpResponse, Result};
use actix_web_grants::permissions::AuthDetails;
use chrono::{DateTime, TimeZone, Utc};
use database::models::{Comic as DatabaseComic, LogEntry};
use database::{DbPool, DbTransaction};
use serde::Deserialize;
use shared::token_permissions;
use tracing::{info_span, Instrument};
use ts_rs::TS;

#[tracing::instrument(skip(pool, auth), fields(permissions = ?auth.permissions))]
#[allow(clippy::too_many_lines)]
pub async fn patch_comic(
    pool: web::Data<DbPool>,
    request: web::Json<PatchComicBody>,
    comic_id: web::Path<ComicId>,
    auth: AuthDetails,
) -> Result<HttpResponse> {
    ensure_is_authorized(&auth, token_permissions::CAN_CHANGE_COMIC_DATA)
        .map_err(error::ErrorForbidden)?;

    let mut transaction = pool
        .begin()
        .instrument(info_span!("Pool::begin"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let comic_id = comic_id.into_inner();
    DatabaseComic::ensure_exists_by_id(&mut *transaction, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;

    let PatchComicBody {
        token,
        publish_date,
        title,
        tagline,
        is_guest_comic,
        is_non_canon,
        has_no_cast,
        has_no_location,
        has_no_storyline,
        has_no_title,
        has_no_tagline,
    } = request.into_inner();

    let mut updated = Vec::with_capacity(10);

    if let Some(PublishDatePatch {
        publish_date,
        is_accurate_publish_date,
    }) = publish_date
    {
        update_publish_date(
            &mut transaction,
            comic_id,
            publish_date,
            is_accurate_publish_date,
            token,
        )
        .await?;

        updated.push("publish date")
    }

    if let Some(title) = title {
        update_title(&mut transaction, comic_id, title, token).await?;

        updated.push("title")
    }

    if let Some(tagline) = tagline {
        update_tagline(&mut transaction, comic_id, tagline, token).await?;

        updated.push("tagline")
    }

    if let Some(is_guest_comic) = is_guest_comic {
        update_flag(
            FlagType::IsGuestComic,
            is_guest_comic,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("isGuestComic flag")
    }

    if let Some(is_non_canon) = is_non_canon {
        update_flag(
            FlagType::IsNonCanon,
            is_non_canon,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("isNonCanon flag")
    }

    if let Some(has_no_cast) = has_no_cast {
        update_flag(
            FlagType::HasNoCast,
            has_no_cast,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("hasNoCast flag")
    }

    if let Some(has_no_location) = has_no_location {
        update_flag(
            FlagType::HasNoLocation,
            has_no_location,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("hasNoLocation flag")
    }

    if let Some(has_no_storyline) = has_no_storyline {
        update_flag(
            FlagType::HasNoStoryline,
            has_no_storyline,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("hasNoStoryline flag")
    }

    if let Some(has_no_title) = has_no_title {
        update_flag(
            FlagType::HasNoTitle,
            has_no_title,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("hasNoTitle flag")
    }

    if let Some(has_no_tagline) = has_no_tagline {
        update_flag(
            FlagType::HasNoTagline,
            has_no_tagline,
            comic_id,
            token,
            &mut transaction,
        )
        .await?;

        updated.push("hasNoTagline flag")
    }

    transaction
        .commit()
        .instrument(info_span!("Transaction::commit"))
        .await
        .map_err(error::ErrorInternalServerError)?;

    let mut changed = updated.join(", ");
    andify_comma_string(&mut changed);

    Ok(HttpResponse::Ok().body(format!("Updated {} for comic {comic_id}", changed)))
}

#[tracing::instrument(skip(transaction))]
async fn update_publish_date(
    transaction: &mut DbTransaction<'_>,
    comic_id: ComicId,
    publish_date: DateTime<Utc>,
    is_accurate_publish_date: bool,
    token: Token,
) -> Result<(), actix_web::Error> {
    let old_publish_date =
        DatabaseComic::publish_date_by_id(&mut **transaction, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;
    DatabaseComic::update_publish_date_by_id(
        &mut **transaction,
        comic_id.into_inner(),
        publish_date,
        is_accurate_publish_date,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    if let Some(old_publish_date) = old_publish_date {
        LogEntry::log_action(
            &mut **transaction,
            token.to_string(),
            format!(
                "Changed publish date on comic #{} from \"{}\" to \"{}\"",
                comic_id,
                Utc.from_utc_datetime(&old_publish_date)
                    .to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                publish_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true)
            ),
            Some(comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        LogEntry::log_action(
            &mut **transaction,
            token.to_string(),
            format!(
                "Set publish date on comic #{} to \"{}\"",
                comic_id,
                publish_date.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
            ),
            Some(comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    }

    Ok(())
}

#[tracing::instrument(skip(transaction))]
async fn update_title(
    transaction: &mut DbTransaction<'_>,
    comic_id: ComicId,
    title: String,
    token: Token,
) -> Result<(), actix_web::Error> {
    let old_title = DatabaseComic::title_by_id(&mut **transaction, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?
        .expect("Due to ensure_exists_by_id call, this should never be None");
    DatabaseComic::update_title_by_id(&mut **transaction, comic_id.into_inner(), &title)
        .await
        .map_err(error::ErrorInternalServerError)?;

    if old_title.is_empty() {
        LogEntry::log_action(
            &mut **transaction,
            token.to_string(),
            format!("Set title on comic #{} to \"{}\"", comic_id, title),
            Some(comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    } else {
        LogEntry::log_action(
            &mut **transaction,
            token.to_string(),
            format!(
                "Changed title on comic #{} from \"{}\" to \"{}\"",
                comic_id, old_title, title
            ),
            Some(comic_id.into_inner()),
            None,
        )
        .await
        .map_err(error::ErrorInternalServerError)?;
    }

    Ok(())
}

#[tracing::instrument(skip(transaction))]
async fn update_tagline(
    transaction: &mut DbTransaction<'_>,
    comic_id: ComicId,
    tagline: String,
    token: Token,
) -> Result<(), actix_web::Error> {
    let old_tagline = DatabaseComic::tagline_by_id(&mut **transaction, comic_id.into_inner())
        .await
        .map_err(error::ErrorInternalServerError)?;
    DatabaseComic::update_tagline_by_id(&mut **transaction, comic_id.into_inner(), &tagline)
        .await
        .map_err(error::ErrorInternalServerError)?;

    match old_tagline {
        Some(old_tagline) if !old_tagline.is_empty() => {
            LogEntry::log_action(
                &mut **transaction,
                token.to_string(),
                format!(
                    "Changed tagline on comic #{} from \"{}\" to \"{}\"",
                    comic_id, old_tagline, tagline
                ),
                Some(comic_id.into_inner()),
                None,
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
        _ => {
            LogEntry::log_action(
                &mut **transaction,
                token.to_string(),
                format!("Set tagline on comic #{} to \"{}\"", comic_id, tagline),
                Some(comic_id.into_inner()),
                None,
            )
            .await
            .map_err(error::ErrorInternalServerError)?;
        }
    }

    Ok(())
}

#[tracing::instrument(skip(transaction))]
pub(super) async fn update_flag(
    flag_type: FlagType,
    flag_value: bool,
    comic_id: ComicId,
    token: Token,
    transaction: &mut DbTransaction<'_>,
) -> Result<()> {
    let (true_value_log_text, false_value_log_text, sql_result) = match flag_type {
        FlagType::IsGuestComic => (
            "to be a guest comic",
            "to be a Jeph comic",
            DatabaseComic::update_is_guest_comic_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
        FlagType::IsNonCanon => (
            "to be non-canon",
            "to be canon",
            DatabaseComic::update_is_non_canon_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
        FlagType::HasNoCast => (
            "to have no cast",
            "to have cast",
            DatabaseComic::update_has_no_cast_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
        FlagType::HasNoLocation => (
            "to have no locations",
            "to have locations",
            DatabaseComic::update_has_no_location_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
        FlagType::HasNoStoryline => (
            "to have no storylines",
            "to have storylines",
            DatabaseComic::update_has_no_storyline_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
        FlagType::HasNoTitle => (
            "to have no title",
            "to have a title",
            DatabaseComic::update_has_no_title_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
        FlagType::HasNoTagline => (
            "to have no tagline",
            "to have a tagline",
            DatabaseComic::update_has_no_tagline_by_id(
                &mut **transaction,
                comic_id.into_inner(),
                flag_value,
            )
            .await,
        ),
    };

    sql_result.map_err(error::ErrorInternalServerError)?;

    LogEntry::log_action(
        &mut **transaction,
        token.to_string(),
        format!(
            "Set comic #{} {}",
            comic_id,
            if flag_value {
                true_value_log_text
            } else {
                false_value_log_text
            }
        ),
        Some(comic_id.into_inner()),
        None,
    )
    .await
    .map_err(error::ErrorInternalServerError)?;

    Ok(())
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PatchComicBody {
    pub token: Token,

    #[ts(optional)]
    pub publish_date: Option<PublishDatePatch>,
    #[ts(optional)]
    pub title: Option<String>,
    #[ts(optional)]
    pub tagline: Option<String>,
    #[ts(optional)]
    pub is_guest_comic: Option<bool>,
    #[ts(optional)]
    pub is_non_canon: Option<bool>,
    #[ts(optional)]
    pub has_no_cast: Option<bool>,
    #[ts(optional)]
    pub has_no_location: Option<bool>,
    #[ts(optional)]
    pub has_no_storyline: Option<bool>,
    #[ts(optional)]
    pub has_no_title: Option<bool>,
    #[ts(optional)]
    pub has_no_tagline: Option<bool>,
}

#[derive(Debug, Deserialize, TS)]
#[serde(rename_all = "camelCase")]
#[ts(export)]
pub struct PublishDatePatch {
    #[ts(type = "string")]
    pub publish_date: DateTime<Utc>,
    #[ts(optional)]
    pub is_accurate_publish_date: bool,
}

#[derive(Debug)]
pub enum FlagType {
    IsGuestComic,
    IsNonCanon,
    HasNoCast,
    HasNoLocation,
    HasNoStoryline,
    HasNoTitle,
    HasNoTagline,
}
