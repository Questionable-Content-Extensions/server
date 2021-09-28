-- Add migration script here

ALTER TABLE `log_entry`
CHANGE `UserToken` `UserToken` CHAR(36)
CHARACTER SET utf8
COLLATE utf8_unicode_ci
NOT NULL
COMMENT '(DC2Type:guid)';