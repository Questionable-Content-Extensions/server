-- Add migration script here

RENAME TABLE `comic` TO `Comic`;
RENAME TABLE `items` TO `Item`;
RENAME TABLE `ItemImages` TO `ItemImage`;
RENAME TABLE `log_entry` TO `LogEntry`;
RENAME TABLE `news` TO `News`;
RENAME TABLE `occurences` TO `Occurrence`;
RENAME TABLE `token` TO `Token`;

DROP TABLE `errorlog`;