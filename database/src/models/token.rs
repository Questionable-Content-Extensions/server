use shared::token_permissions;

#[derive(Debug)]
pub struct Token {
    pub id: String,
    pub identifier: String,
    pub can_add_image_to_item: u8,
    pub can_add_item_to_comic: u8,
    pub can_change_comic_data: u8,
    pub can_change_item_data: u8,
    pub can_remove_image_from_item: u8,
    pub can_remove_item_from_comic: u8,
}

impl Token {
    #[tracing::instrument(skip(executor, token), fields(token = token.as_ref()))]
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
                SELECT * FROM `Token`
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
        if token.can_add_item_to_comic != 0 {
            permissions.push(token_permissions::CAN_ADD_ITEM_TO_COMIC.to_string());
        }
        if token.can_remove_item_from_comic != 0 {
            permissions.push(token_permissions::CAN_REMOVE_ITEM_FROM_COMIC.to_string());
        }
        if token.can_change_comic_data != 0 {
            permissions.push(token_permissions::CAN_CHANGE_COMIC_DATA.to_string());
        }
        if token.can_add_image_to_item != 0 {
            permissions.push(token_permissions::CAN_ADD_IMAGE_TO_ITEM.to_string());
        }
        if token.can_remove_image_from_item != 0 {
            permissions.push(token_permissions::CAN_REMOVE_IMAGE_FROM_ITEM.to_string());
        }
        if token.can_change_item_data != 0 {
            permissions.push(token_permissions::CAN_CHANGE_ITEM_DATA.to_string());
        }
        Ok(permissions)
    }
}
