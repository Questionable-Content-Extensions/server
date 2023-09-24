use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Deserialize, Serialize, TS)]
#[serde(transparent)]
#[ts(export)]
pub struct True(#[ts(type = "true")] bool);
impl Default for True {
    fn default() -> Self {
        Self(true)
    }
}

#[derive(Debug, Deserialize, Serialize, TS)]
#[serde(transparent)]
#[ts(export)]
pub struct False(#[ts(type = "false")] bool);
// I want the impl to be explict here, to match the impl of `True`
#[allow(clippy::derivable_impls)]
impl Default for False {
    fn default() -> Self {
        Self(false)
    }
}
