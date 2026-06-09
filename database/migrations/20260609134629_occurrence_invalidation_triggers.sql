-- Add migration script here

-- Add a dirty flag so triggers can signal the background refresher.
ALTER TABLE `stats_cache_meta`
    ADD COLUMN `needs_refresh` TINYINT UNSIGNED NOT NULL DEFAULT 0;

-- Cache is currently fresh (just populated by the previous migration).
UPDATE `stats_cache_meta` SET `needs_refresh` = 0 WHERE `cache_key` = 'cast_rank_stints';

-- Invalidate the stints cache whenever an occurrence is added or removed.
-- Single-statement triggers: no BEGIN/END, no DELIMITER needed.
CREATE TRIGGER `trg_occurrence_ins_invalidate_stints`
AFTER INSERT ON `Occurrence`
FOR EACH ROW
UPDATE `stats_cache_meta` SET `needs_refresh` = 1 WHERE `cache_key` = 'cast_rank_stints';

CREATE TRIGGER `trg_occurrence_del_invalidate_stints`
AFTER DELETE ON `Occurrence`
FOR EACH ROW
UPDATE `stats_cache_meta` SET `needs_refresh` = 1 WHERE `cache_key` = 'cast_rank_stints';
