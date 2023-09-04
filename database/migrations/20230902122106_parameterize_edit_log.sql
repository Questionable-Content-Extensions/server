ALTER TABLE `LogEntry`
  ADD COLUMN `comic_involved` smallint(6) UNSIGNED NULL,
  ADD INDEX (`comic_involved`);

UPDATE `LogEntry` l1
  SET `comic_involved` = (
    SELECT REGEXP_REPLACE(REGEXP_SUBSTR(l2.`action`, 'comic #[[:digit:]]+'), 'comic #', '') AS `comic`
      FROM `LogEntry` l2
      WHERE l2.`id` = l1.`id`
      HAVING `comic` <> ''
  );

ALTER TABLE `LogEntry`
  ADD COLUMN `item_involved` smallint(6) UNSIGNED NULL,
  ADD INDEX (`item_involved`);

UPDATE `LogEntry` l1
  SET `item_involved` = (
    SELECT REGEXP_REPLACE(REGEXP_SUBSTR(l2.`action`, '(cast|location|storyline|item) #[[:digit:]]+'), '(cast|location|storyline|item) #', '') AS `item`
      FROM `LogEntry` l2
      WHERE l2.`id` = l1.`id`
      HAVING `item` <> ''
  );
