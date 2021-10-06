use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Copy, Clone, Debug, Deserialize, Eq, Hash, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Token(Uuid);

derive_transparent_display!(Token);

impl From<Uuid> for Token {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}
