-- Comics and occurrences that the public may see; the stats queries read these
-- instead of `Comic` and `Occurrence` so advance (hidden) comics stay private.
CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_visible_comic` AS
SELECT
    `c`.`id`,
    `c`.`is_guest_comic`,
    `c`.`is_non_canon`,
    `c`.`title`,
    `c`.`tagline`,
    `c`.`publish_date`,
    `c`.`is_accurate_publish_date`,
    `c`.`has_no_cast`,
    `c`.`has_no_location`,
    `c`.`has_no_storyline`,
    `c`.`has_no_tagline`,
    `c`.`has_no_title`,
    `c`.`image_type`
FROM `Comic` `c`
WHERE NOT `c`.`hidden`;

CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_visible_occurrence` AS
SELECT `o`.`comic_id`, `o`.`item_id`
FROM `Occurrence` `o`
JOIN `Comic` `c` ON `c`.`id` = `o`.`comic_id`
WHERE NOT `c`.`hidden`;

-- The rank-stints cache only counts visible comics, so hiding or publishing a
-- comic makes it stale.
CREATE OR REPLACE TRIGGER `trg_comic_upd_invalidate_stints`
AFTER UPDATE ON `Comic`
FOR EACH ROW
UPDATE `stats_cache_meta` SET `needs_refresh` = 1
WHERE `cache_key` = 'cast_rank_stints' AND NOT (OLD.`hidden` <=> NEW.`hidden`);
