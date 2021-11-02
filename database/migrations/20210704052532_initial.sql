-- Add migration script here

CREATE TABLE `comic` (
  `id` smallint(6) NOT NULL,
  `isGuestComic` tinyint(1) NOT NULL DEFAULT '0',
  `isNonCanon` tinyint(1) NOT NULL DEFAULT '0',
  `title` varchar(255) COLLATE utf8_unicode_ci NOT NULL,
  `tagline` varchar(255) COLLATE utf8_unicode_ci DEFAULT NULL,
  `publishDate` datetime DEFAULT NULL,
  `isAccuratePublishDate` tinyint(1) NOT NULL DEFAULT '0',
  `HasNoCast` bit(1) NOT NULL DEFAULT b'0',
  `HasNoLocation` bit(1) NOT NULL DEFAULT b'0',
  `HasNoStoryline` bit(1) NOT NULL DEFAULT b'0',
  `HasNoTagline` bit(1) NOT NULL DEFAULT b'0',
  `HasNoTitle` bit(1) NOT NULL DEFAULT b'0',
  `ImageType` int(11) NOT NULL DEFAULT '0' COMMENT '2 = GIF, 3 = JPG'
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `news` (
  `comic` smallint(6) NOT NULL,
  `lastUpdated` date NOT NULL,
  `news` text COLLATE utf8_unicode_ci NOT NULL,
  `updateFactor` double NOT NULL DEFAULT '1',
  `isLocked` tinyint(1) NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `items` (
  `id` smallint(6) NOT NULL,
  `shortName` varchar(50) NOT NULL,
  `name` varchar(255) NOT NULL,
  `type` varchar(255) NOT NULL,
  `Color_Blue` tinyint(3) UNSIGNED NOT NULL DEFAULT '0',
  `Color_Green` tinyint(3) UNSIGNED NOT NULL DEFAULT '0',
  `Color_Red` tinyint(3) UNSIGNED NOT NULL DEFAULT '0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `occurences` (
  `comic_id` smallint(6) NOT NULL,
  `items_id` smallint(6) NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `ItemImages` (
  `Id` int(11) NOT NULL,
  `ItemId` int(11) NOT NULL,
  `Image` longblob NOT NULL,
  `CRC32CHash` int(10) UNSIGNED NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `token` (
  `id` char(36) COLLATE utf8_unicode_ci NOT NULL COMMENT '(DC2Type:guid)',
  `identifier` varchar(50) COLLATE utf8_unicode_ci NOT NULL,
  `CanAddImageToItem` bit(1) NOT NULL DEFAULT b'0',
  `CanAddItemToComic` bit(1) NOT NULL DEFAULT b'0',
  `CanChangeComicData` bit(1) NOT NULL DEFAULT b'0',
  `CanChangeItemData` bit(1) NOT NULL DEFAULT b'0',
  `CanRemoveImageFromItem` bit(1) NOT NULL DEFAULT b'0',
  `CanRemoveItemFromComic` bit(1) NOT NULL DEFAULT b'0'
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `log_entry` (
  `id` int(11) NOT NULL,
  `UserToken` char(36) COLLATE utf8_unicode_ci DEFAULT NULL COMMENT '(DC2Type:guid)',
  `DateTime` datetime NOT NULL,
  `Action` longtext COLLATE utf8_unicode_ci NOT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

CREATE TABLE `errorlog` (
  `ID` int(11) NOT NULL,
  `timestamp` datetime NOT NULL,
  `type` varchar(255) NOT NULL,
  `message` varchar(255) NOT NULL,
  `code` varchar(255) NOT NULL,
  `file` varchar(255) NOT NULL,
  `line` smallint(6) NOT NULL,
  `trace` text NOT NULL,
  `ipaddress` varchar(15) DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8 COLLATE=utf8_unicode_ci;

--

ALTER TABLE `comic`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `news`
  ADD PRIMARY KEY (`comic`),
  ADD UNIQUE KEY `UNIQ_1DD399505B7EA5AA` (`comic`);
ALTER TABLE `news`
  ADD CONSTRAINT `FK_1DD399505B7EA5AA` FOREIGN KEY (`comic`) REFERENCES `comic` (`id`);

ALTER TABLE `items`
  ADD PRIMARY KEY (`id`);
ALTER TABLE `items`
  MODIFY `id` smallint(6) NOT NULL AUTO_INCREMENT;

ALTER TABLE `occurences`
  ADD PRIMARY KEY (`comic_id`,`items_id`),
  ADD KEY `IDX_116ACCE4D663094A` (`comic_id`),
  ADD KEY `IDX_116ACCE46BB0AE84` (`items_id`);
ALTER TABLE `occurences`
  ADD CONSTRAINT `FK_5E37CF34D663094A` FOREIGN KEY (`comic_id`) REFERENCES `comic` (`id`),
  ADD CONSTRAINT `FK_5E37CF346BB0AE84` FOREIGN KEY (`items_id`) REFERENCES `items` (`id`);

ALTER TABLE `ItemImages`
  ADD PRIMARY KEY (`Id`),
  ADD KEY `IX_ItemImages_ItemId` (`ItemId`);
ALTER TABLE `ItemImages`
  MODIFY `Id` int(11) NOT NULL AUTO_INCREMENT;

ALTER TABLE `token`
  ADD PRIMARY KEY (`id`);

ALTER TABLE `log_entry`
  ADD PRIMARY KEY (`id`),
  ADD KEY `IDX_B5F762D3BA3304E` (`UserToken`);
ALTER TABLE `log_entry`
  MODIFY `id` int(11) NOT NULL AUTO_INCREMENT;
ALTER TABLE `log_entry`
  ADD CONSTRAINT `FK_B5F762D3BA3304E` FOREIGN KEY (`UserToken`) REFERENCES `token` (`id`);

ALTER TABLE `errorlog`
  ADD PRIMARY KEY (`ID`);
ALTER TABLE `errorlog`
  MODIFY `ID` int(11) NOT NULL AUTO_INCREMENT;