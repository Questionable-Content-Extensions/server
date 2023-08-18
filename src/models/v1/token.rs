use parse_display::Display;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Deserialize, Display, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Token(Uuid);

impl From<Uuid> for Token {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
