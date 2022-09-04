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
#[allow(clippy::derivable_impls)] // I want the impl to be explict here
impl Default for False {
    fn default() -> Self {
        Self(false)
    }
}
