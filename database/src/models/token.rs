use shared::token_permissions;

#[derive(Debug)]
pub struct Token {
    pub id: String,
    pub identifier: String,
    pub CanAddImageToItem: u8,
    pub CanAddItemToComic: u8,
    pub CanChangeComicData: u8,
    pub CanChangeItemData: u8,
    pub CanRemoveImageFromItem: u8,
    pub CanRemoveItemFromComic: u8,
}

impl Token {
    pub async fn get_permissions_for_token<'e, 'c: 'e, E>(
        executor: E,
        token: impl AsRef<str>,
    ) -> sqlx::Result<Vec<String>>
    where
        E: 'e + sqlx::Executor<'c, Database = sqlx::MySql>,
    {
        let token = token.as_ref();

        let result = sqlx::query_as!(
            Self,
            r#"
            SELECT * FROM `token`
            WHERE `id` = ?
        "#,
            token
        )
        .fetch_optional(executor)
        .await?;

        let token = if let Some(token) = result {
            token
        } else {
            // Invalid token provided, there are no permissions
            return Ok(vec![]);
        };

        let mut permissions = Vec::with_capacity(7);
        permissions.push(token_permissions::HAS_VALID_TOKEN.to_string());
        if token.CanAddItemToComic != 0 {
            permissions.push(token_permissions::CAN_ADD_ITEM_TO_COMIC.to_string());
        }
        if token.CanRemoveItemFromComic != 0 {
            permissions.push(token_permissions::CAN_REMOVE_ITEM_FROM_COMIC.to_string());
        }
        if token.CanChangeComicData != 0 {
            permissions.push(token_permissions::CAN_CHANGE_COMIC_DATA.to_string());
        }
        if token.CanAddImageToItem != 0 {
            permissions.push(token_permissions::CAN_ADD_IMAGE_TO_ITEM.to_string());
        }
        if token.CanRemoveImageFromItem != 0 {
            permissions.push(token_permissions::CAN_REMOVE_IMAGE_FROM_ITEM.to_string());
        }
        if token.CanChangeItemData != 0 {
            permissions.push(token_permissions::CAN_CHANGE_ITEM_DATA.to_string());
        }
        Ok(permissions)
    }
}
