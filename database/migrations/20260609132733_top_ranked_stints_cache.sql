-- Add migration script here

-- Composite index to speed up the running-count window function
-- (PARTITION BY item_id ORDER BY comic_id).  The existing IDX_116ACCE46BB0AE84
-- only covers item_id; adding comic_id avoids an extra sort pass.
ALTER TABLE `Occurrence`
    ADD INDEX `Occurrence_item_comic` (`item_id`, `comic_id`);

-- Pre-computed leadership stints cache.
-- Recomputed on demand when a new comic appears (see stats controller).
CREATE TABLE `cast_rank_leadership_stints` (
    `from_comic`              SMALLINT UNSIGNED NOT NULL,
    `to_comic_exclusive`      SMALLINT UNSIGNED     NULL,
    `item_id`                 SMALLINT UNSIGNED NOT NULL,
    `appearances_at_takeover` INT      UNSIGNED NOT NULL,
    PRIMARY KEY (`from_comic`),
    KEY `crls_item_id` (`item_id`)
);

-- Single-row-per-key metadata used to detect staleness.
CREATE TABLE `stats_cache_meta` (
    `cache_key`    VARCHAR(100)     NOT NULL,
    `last_comic_id` SMALLINT UNSIGNED NOT NULL,
    PRIMARY KEY (`cache_key`)
);

-- Populate the cache immediately so the first request is instant.
INSERT INTO `cast_rank_leadership_stints`
    (`from_comic`, `to_comic_exclusive`, `item_id`, `appearances_at_takeover`)
WITH `cast_occurrences` AS (
    SELECT `o`.`comic_id`, `o`.`item_id`
    FROM `Occurrence` `o`
    JOIN `Item` `i` ON `i`.`id` = `o`.`item_id`
    WHERE `i`.`type` = 'cast'
),
`running_counts` AS (
    SELECT
        `comic_id`,
        `item_id`,
        COUNT(*) OVER (
            PARTITION BY `item_id`
            ORDER BY `comic_id`
        ) AS `cnt`
    FROM `cast_occurrences`
),
`new_global_highs` AS (
    SELECT `comic_id`, `item_id`, `cnt`
    FROM (
        SELECT
            `comic_id`,
            `item_id`,
            `cnt`,
            MAX(`cnt`) OVER (
                ORDER BY `comic_id`, `cnt` DESC, `item_id`
                ROWS BETWEEN UNBOUNDED PRECEDING AND 1 PRECEDING
            ) AS `prev_global_max`
        FROM `running_counts`
    ) `t`
    WHERE `cnt` > COALESCE(`prev_global_max`, 0)
),
`leader_changes` AS (
    SELECT
        `comic_id`,
        `item_id`,
        `cnt`,
        LAG(`item_id`) OVER (ORDER BY `comic_id`) AS `prev_item_id`
    FROM `new_global_highs`
),
`stints` AS (
    SELECT
        `comic_id` AS `from_comic`,
        LEAD(`comic_id`) OVER (ORDER BY `comic_id`) AS `to_comic_exclusive`,
        `item_id`,
        CAST(`cnt` AS UNSIGNED) AS `appearances_at_takeover`
    FROM `leader_changes`
    WHERE `prev_item_id` IS NULL OR `item_id` != `prev_item_id`
)
SELECT
    CAST(`s`.`from_comic`              AS UNSIGNED) AS `from_comic`,
    `s`.`to_comic_exclusive`,
    CAST(`s`.`item_id`                 AS UNSIGNED) AS `item_id`,
    `s`.`appearances_at_takeover`
FROM `stints` `s`;

INSERT INTO `stats_cache_meta` (`cache_key`, `last_comic_id`)
SELECT 'cast_rank_stints', MAX(`o`.`comic_id`)
FROM `Occurrence` `o`
JOIN `Item` `i` ON `i`.`id` = `o`.`item_id`
WHERE `i`.`type` = 'cast';
