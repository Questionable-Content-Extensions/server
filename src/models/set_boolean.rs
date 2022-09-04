use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct True(bool);
impl Default for True {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct False(bool);
impl Default for False {
    fn default() -> Self {
        Self(false)
    }
}
