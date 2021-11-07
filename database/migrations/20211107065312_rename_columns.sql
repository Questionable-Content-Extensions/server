-- Drop all foreign keys and indexes
ALTER TABLE `ItemImage` DROP FOREIGN KEY `ItemImage_ibfk_1`;
ALTER TABLE `ItemImage` DROP INDEX `IX_ItemImages_ItemId`;
ALTER TABLE `LogEntry` DROP FOREIGN KEY `FK_B5F762D3BA3304E`;
ALTER TABLE `News` DROP FOREIGN KEY `FK_1DD399505B7EA5AA`;
ALTER TABLE `Occurrence` DROP FOREIGN KEY `FK_5E37CF346BB0AE84`;
ALTER TABLE `Occurrence` DROP FOREIGN KEY `FK_5E37CF34D663094A`;

ALTER TABLE `Comic`
  CHANGE `id` `id` SMALLINT(6) UNSIGNED NOT NULL,
  CHANGE `isGuestComic` `is_guest_comic` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `isNonCanon` `is_non_canon` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `publishDate` `publish_date` DATETIME NULL DEFAULT NULL,
  CHANGE `isAccuratePublishDate` `is_accurate_publish_date` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `HasNoCast` `has_no_cast` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `HasNoLocation` `has_no_location` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `HasNoStoryline` `has_no_storyline` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `HasNoTagline` `has_no_tagline` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `HasNoTitle` `has_no_title` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `ImageType` `image_type` INT(11) NOT NULL DEFAULT '0' COMMENT '0 = Unknown, 1 = PNG, 2 = GIF, 3 = JPG';

ALTER TABLE `ItemImage`
  CHANGE `Id` `id` INT(11) UNSIGNED NOT NULL AUTO_INCREMENT,
  CHANGE `ItemId` `item_id` SMALLINT(6) UNSIGNED NOT NULL,
  CHANGE `Image` `image` LONGBLOB NOT NULL,
  CHANGE `CRC32CHash` `crc32c_hash` INT(10) UNSIGNED NOT NULL;

ALTER TABLE `Item`
  CHANGE `id` `id` SMALLINT(6) UNSIGNED NOT NULL AUTO_INCREMENT,
  CHANGE `shortName` `short_name` VARCHAR(50) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  CHANGE `Color_Blue` `color_blue` TINYINT(3) UNSIGNED NOT NULL DEFAULT '0',
  CHANGE `Color_Green` `color_green` TINYINT(3) UNSIGNED NOT NULL DEFAULT '0',
  CHANGE `Color_Red` `color_red` TINYINT(3) UNSIGNED NOT NULL DEFAULT '0';

ALTER TABLE `LogEntry`
  CHANGE `id` `id` INT(11) UNSIGNED NOT NULL AUTO_INCREMENT,
  CHANGE `UserToken` `user_token` CHAR(36) CHARACTER SET utf8 COLLATE utf8_unicode_ci NOT NULL,
  CHANGE `DateTime` `date_time` DATETIME NOT NULL,
  CHANGE `Action` `action` LONGTEXT CHARACTER SET utf8 COLLATE utf8_unicode_ci NOT NULL;

ALTER TABLE `News`
  CHANGE `comic` `comic_id` SMALLINT(6) UNSIGNED NOT NULL,
  CHANGE `lastUpdated` `last_updated` DATE NOT NULL,
  CHANGE `updateFactor` `update_factor` DOUBLE NOT NULL DEFAULT '1',
  CHANGE `isLocked` `is_locked` BIT(1) NOT NULL DEFAULT b'0';

ALTER TABLE `Occurrence`
  CHANGE `comic_id` `comic_id` SMALLINT(6) UNSIGNED NOT NULL,
  CHANGE `items_id` `item_id` SMALLINT(6) UNSIGNED NOT NULL;

ALTER TABLE `Token`
  CHANGE `id` `id` CHAR(36) CHARACTER SET utf8 COLLATE utf8_unicode_ci NOT NULL,
  CHANGE `CanAddImageToItem` `can_add_image_to_item` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `CanAddItemToComic` `can_add_item_to_comic` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `CanChangeComicData` `can_change_comic_data` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `CanChangeItemData` `can_change_item_data` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `CanRemoveImageFromItem` `can_remove_image_from_item` BIT(1) NOT NULL DEFAULT b'0',
  CHANGE `CanRemoveItemFromComic` `can_remove_item_from_comic` BIT(1) NOT NULL DEFAULT b'0';

-- Restore any dropped foreign keys and indexes

ALTER TABLE `ItemImage` ADD FOREIGN KEY (`item_id`) REFERENCES `Item` (`id`);
ALTER TABLE `LogEntry` ADD FOREIGN KEY (`user_token`) REFERENCES `Token` (`id`);
ALTER TABLE `News` ADD FOREIGN KEY (`comic_id`) REFERENCES `Comic` (`id`);
ALTER TABLE `Occurrence`
  ADD FOREIGN KEY (`comic_id`) REFERENCES `Comic` (`id`),
  ADD FOREIGN KEY (`item_id`) REFERENCES `Item` (`id`);
ALTER TABLE `ItemImage` ADD KEY `ItemImage_item_id` (`item_id`);
