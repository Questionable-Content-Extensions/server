-- Add migration script here

ALTER TABLE `ItemImages` ENGINE = InnoDB;
ALTER TABLE `ItemImages` CHANGE `ItemId` `ItemId` SMALLINT(6) NOT NULL;
ALTER TABLE `ItemImages` ADD FOREIGN KEY (`ItemId`) REFERENCES `items`(`id`);