#![allow(nonstandard_style)]

use chrono::{NaiveDate, NaiveDateTime};
use log::debug;

pub struct Comic {
    pub id: i16,
    pub ImageType: i32,
    pub isGuestComic: i8,
    pub isNonCanon: i8,
    pub HasNoCast: u8,
    pub HasNoLocation: u8,
    pub HasNoStoryline: u8,
    pub HasNoTitle: u8,
    pub HasNoTagline: u8,
    pub title: String,
    pub tagline: Option<String>,
    pub publishDate: Option<NaiveDateTime>,
    pub isAccuratePublishDate: i8,
}

pub struct Item {
    pub id: i16,
    pub shortName: String,
    pub name: String,
    pub r#type: String,
    pub Color_Blue: u8,
    pub Color_Green: u8,
    pub Color_Red: u8,
}

pub struct News {
    pub comic: i16,
    pub lastUpdated: NaiveDate,
    pub news: String,
    pub updateFactor: f64,
    pub isLocked: i8,
}

impl News {
    pub fn is_outdated(&self) -> bool {
        let days_since_update = (chrono::Utc::today().naive_utc() - self.lastUpdated).num_days();
        let update_factor_days = (31.0 * self.updateFactor) as i64;
        let locked = self.isLocked != 0;

        debug!(
            "News outdated info: id:{} updated:{} factor:{} locked:{} | ({} > {} = {})",
            self.comic,
            self.lastUpdated,
            self.updateFactor,
            locked,
            days_since_update,
            update_factor_days,
            days_since_update > update_factor_days
        );

        !locked && self.updateFactor < 12.0 && days_since_update > update_factor_days
    }
}
