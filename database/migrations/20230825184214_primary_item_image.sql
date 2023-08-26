ALTER TABLE `Item`
  ADD COLUMN `primary_image` int(11) UNSIGNED NULL;

ALTER TABLE `Item` ADD FOREIGN KEY (`primary_image`) REFERENCES `ItemImage` (`id`);
