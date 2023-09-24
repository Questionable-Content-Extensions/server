use parse_display::Display;
use serde::{Deserialize, Serialize};
use ts_rs::TS;
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Deserialize, Display, Eq, Hash, PartialEq, Serialize, TS)]
#[serde(transparent)]
#[ts(export)]
pub struct Token(#[ts(type = "string")] Uuid);

impl From<Uuid> for Token {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
