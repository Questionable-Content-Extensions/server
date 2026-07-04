-- Storyline lifecycle: explicit editor-set start/end comic, independent of
-- attachment. Only meaningful for `type` = 'storyline'; NULL for cast/location.
ALTER TABLE `Item`
    ADD COLUMN `start_comic_id` SMALLINT UNSIGNED NULL DEFAULT NULL,
    ADD COLUMN `end_comic_id`   SMALLINT UNSIGNED NULL DEFAULT NULL;

-- Migration default for existing storylines: startComicId = first appearance
-- (reproduces current "always shown when attached" behavior as "always active").
UPDATE `Item` `i`
JOIN (
    SELECT `item_id`, MIN(`comic_id`) AS `first_comic_id`
    FROM `Occurrence`
    GROUP BY `item_id`
) `o` ON `o`.`item_id` = `i`.`id`
SET `i`.`start_comic_id` = `o`.`first_comic_id`
WHERE `i`.`type` = 'storyline';
