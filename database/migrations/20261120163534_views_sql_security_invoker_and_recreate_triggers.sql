-- Views run with the caller's privileges, so a database copied to another
-- environment keeps working under that environment's user.
CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_comics_missing_cast` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE NOT EXISTS (
    SELECT 1 FROM `Occurrence` `o`
    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
    WHERE `i`.`type` = 'cast' AND `o`.`comic_id` = `c`.`id`
)
AND NOT `c`.`has_no_cast`
AND NOT `c`.`hidden`;

CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_comics_missing_location` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE NOT EXISTS (
    SELECT 1 FROM `Occurrence` `o`
    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
    WHERE `i`.`type` = 'location' AND `o`.`comic_id` = `c`.`id`
)
AND NOT `c`.`has_no_location`
AND NOT `c`.`hidden`;

CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_comics_missing_storyline` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE NOT EXISTS (
    SELECT 1 FROM `Occurrence` `o`
    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
    WHERE `i`.`type` = 'storyline' AND `o`.`comic_id` = `c`.`id`
)
AND NOT `c`.`has_no_storyline`
AND NOT `c`.`hidden`;

CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_comics_missing_title` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE (`c`.`title` IS NULL OR NULLIF(`c`.`title`, '') IS NULL)
AND NOT `c`.`has_no_title`
AND NOT `c`.`hidden`;

CREATE OR REPLACE SQL SECURITY INVOKER VIEW `v_comics_missing_tagline` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE (`c`.`tagline` IS NULL OR NULLIF(`c`.`tagline`, '') IS NULL)
AND NOT `c`.`has_no_tagline`
AND `c`.`id` > 3132
AND NOT `c`.`hidden`;

-- Triggers always run as their definer, which is the user running this
-- migration; re-creating them hands them to this environment's user.
CREATE OR REPLACE TRIGGER `trg_occurrence_ins_invalidate_stints`
AFTER INSERT ON `Occurrence`
FOR EACH ROW
UPDATE `stats_cache_meta` SET `needs_refresh` = 1 WHERE `cache_key` = 'cast_rank_stints';

CREATE OR REPLACE TRIGGER `trg_occurrence_del_invalidate_stints`
AFTER DELETE ON `Occurrence`
FOR EACH ROW
UPDATE `stats_cache_meta` SET `needs_refresh` = 1 WHERE `cache_key` = 'cast_rank_stints';
