//! A fake location item that every v2 item list and comic carries, so that
//! scripts still on API v2 (which cannot receive server messages) show their
//! users an upgrade notice in the item navigation.

use super::{ItemColor, ItemList, ItemNavigationData, ItemType, NavigationData};
use crate::models::ItemId;

pub const OBSOLETE_SCRIPT_WARNING_SHORT_NAME: &str = "UPGRADE SCRIPT";
pub const OBSOLETE_SCRIPT_WARNING_NAME: &str = "You are running an obsolete version of the script, please upgrade ASAP as it will stop working soon";

/// The id is outside the range the database hands out to real items.
pub fn obsolete_script_warning_item_id() -> ItemId {
    ItemId::from(u16::MAX)
}

/// Puts the warning item first in an item list.
pub fn add_obsolete_script_warning_to_item_list(items: &mut Vec<ItemList>) {
    items.insert(0, obsolete_script_warning_item_list_entry());
}

/// Puts the warning item first in a comic's item navigation data.
pub fn add_obsolete_script_warning_to_navigation_items(items: &mut Vec<ItemNavigationData>) {
    items.insert(0, obsolete_script_warning_navigation_data());
}

fn obsolete_script_warning_item_list_entry() -> ItemList {
    ItemList {
        id: obsolete_script_warning_item_id(),
        short_name: OBSOLETE_SCRIPT_WARNING_SHORT_NAME.to_owned(),
        name: OBSOLETE_SCRIPT_WARNING_NAME.to_owned(),
        r#type: ItemType::Location,
        color: ItemColor(0xff, 0x00, 0x00),
        count: 0,
    }
}

fn obsolete_script_warning_navigation_data() -> ItemNavigationData {
    ItemNavigationData {
        id: obsolete_script_warning_item_id(),
        navigation_data: NavigationData {
            first: None,
            previous: None,
            next: None,
            last: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn real_item(id: u16) -> ItemList {
        ItemList {
            id: ItemId::from(id),
            short_name: format!("Item {id}"),
            name: format!("Item number {id}"),
            r#type: ItemType::Cast,
            color: ItemColor(1, 2, 3),
            count: 10,
        }
    }

    #[test]
    fn item_list_gets_warning_item_first_and_keeps_real_items() {
        let mut items = vec![real_item(1), real_item(2)];

        add_obsolete_script_warning_to_item_list(&mut items);

        let ids: Vec<u16> = items.iter().map(|i| i.id.into_inner()).collect();
        assert_eq!(ids, vec![u16::MAX, 1, 2]);
    }

    #[test]
    fn warning_item_list_entry_serializes_as_upgrade_location() {
        let mut items = Vec::new();
        add_obsolete_script_warning_to_item_list(&mut items);

        assert_eq!(
            serde_json::to_value(&items).unwrap(),
            json!([{
                "id": 65535,
                "shortName": "UPGRADE SCRIPT",
                "name": "You are running an obsolete version of the script, please upgrade ASAP as it will stop working soon",
                "type": "location",
                "color": "ff0000",
                "count": 0,
            }])
        );
    }

    #[test]
    fn navigation_items_get_warning_item_first_with_matching_id() {
        let mut items = vec![ItemNavigationData {
            id: ItemId::from(7),
            navigation_data: NavigationData {
                first: None,
                previous: None,
                next: None,
                last: None,
            },
        }];

        add_obsolete_script_warning_to_navigation_items(&mut items);

        assert_eq!(
            serde_json::to_value(&items).unwrap(),
            json!([
                { "id": 65535, "first": null, "previous": null, "next": null, "last": null },
                { "id": 7, "first": null, "previous": null, "next": null, "last": null },
            ])
        );
        let mut item_list = Vec::new();
        add_obsolete_script_warning_to_item_list(&mut item_list);
        assert_eq!(items[0].id, item_list[0].id);
    }
}
