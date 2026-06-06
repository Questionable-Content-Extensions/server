CREATE VIEW `v_comics_missing_cast` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE NOT EXISTS (
    SELECT 1 FROM `Occurrence` `o`
    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
    WHERE `i`.`type` = 'cast' AND `o`.`comic_id` = `c`.`id`
)
AND NOT `c`.`has_no_cast`;

CREATE VIEW `v_comics_missing_location` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE NOT EXISTS (
    SELECT 1 FROM `Occurrence` `o`
    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
    WHERE `i`.`type` = 'location' AND `o`.`comic_id` = `c`.`id`
)
AND NOT `c`.`has_no_location`;

CREATE VIEW `v_comics_missing_storyline` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE NOT EXISTS (
    SELECT 1 FROM `Occurrence` `o`
    JOIN `Item` `i` ON `o`.`item_id` = `i`.`id`
    WHERE `i`.`type` = 'storyline' AND `o`.`comic_id` = `c`.`id`
)
AND NOT `c`.`has_no_storyline`;

CREATE VIEW `v_comics_missing_title` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE (`c`.`title` IS NULL OR NULLIF(`c`.`title`, '') IS NULL)
AND NOT `c`.`has_no_title`;

CREATE VIEW `v_comics_missing_tagline` AS
SELECT `c`.`id`
FROM `Comic` `c`
WHERE (`c`.`tagline` IS NULL OR NULLIF(`c`.`tagline`, '') IS NULL)
AND NOT `c`.`has_no_tagline`
AND `c`.`id` > 3132;
