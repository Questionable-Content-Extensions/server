// {"comic":1,"title":"Employment Sucks","isNonCanon":false,"isGuestComic":false}
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, sqlx::FromRow)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Comic {
    #[sqlx(rename = "id")]
    pub(crate) comic: i16,
    pub(crate) title: String,
    #[sqlx(rename = "isNonCanon")]
    pub(crate) is_non_canon: bool,
    #[sqlx(rename = "isGuestComic")]
    pub(crate) is_guest_comic: bool,
}
