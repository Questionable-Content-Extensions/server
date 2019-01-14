using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Persistence.Migrations
{
	public partial class ComicItemNavigationData_SPv1 : Migration
	{
		protected override void Up(MigrationBuilder migrationBuilder)
		{
			var sp = @"CREATE PROCEDURE `ComicItemNavigationData`(IN `comicId` SMALLINT, IN `exclude` TEXT)
    READS SQL DATA
BEGIN
	DECLARE itemId SMALLINT;
	DECLARE curs CURSOR FOR SELECT items_id FROM `occurences` WHERE comic_id = comicId;
	DECLARE CONTINUE HANDLER FOR NOT FOUND SET @bDone = 1;

	CREATE TEMPORARY TABLE ItemData
		(Id SMALLINT, First INT, Previous INT, Next INT, Last INT, Count INT);

	OPEN curs;

	SET @bDone = 0;
	FETCH curs INTO itemId;
	REPEAT
		BEGIN
			DECLARE CONTINUE HANDLER FOR NOT FOUND BEGIN END;

			SET @first = NULL;
			SET @previous = NULL;
			SET @next = NULL;
			SET @last = NULL;
			SET @count = NULL;

			IF exclude = 'guest' THEN
				SELECT
					MIN(c.id),
					MAX(c.id),
					COUNT(c.id)
				INTO @first, @last, @count
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.isGuestComic = false;
			ELSEIF exclude = 'non-canon' THEN
				SELECT
					MIN(c.id),
					MAX(c.id),
					COUNT(c.id)
				INTO @first, @last, @count
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.isNonCanon = false;
			ELSE
				SELECT
					MIN(c.id),
					MAX(c.id),
					COUNT(c.id)
				INTO @first, @last, @count
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId;
			END IF;

			IF exclude = 'guest' THEN
				SELECT c.id INTO @previous
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id < comicId
					AND c.isGuestComic = false
				ORDER BY c.id DESC
				LIMIT 1;
			ELSEIF exclude = 'non-canon' THEN
				SELECT c.id INTO @previous
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id < comicId
					AND c.isNonCanon = false
				ORDER BY c.id DESC
				LIMIT 1;
			ELSE
			SELECT c.id INTO @previous
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id < comicId
				ORDER BY c.id DESC
				LIMIT 1;
			END IF;

			IF exclude = 'guest' THEN
				SELECT c.id INTO @next
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id > comicId
					AND c.isGuestComic = false
				ORDER BY c.id ASC
				LIMIT 1;
			ELSEIF exclude = 'non-canon' THEN
				SELECT c.id INTO @next
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id > comicId
					AND c.isNonCanon = false
				ORDER BY c.id ASC
				LIMIT 1;
			ELSE
			SELECT c.id INTO @next
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id > comicId
				ORDER BY c.id ASC
				LIMIT 1;
			END IF;
		END;

		INSERT INTO ItemData
			(Id, First, Previous, Next, Last, Count)
		VALUES
			(itemId, @first, @previous, @next, @last, @count);

		FETCH curs INTO itemId;
	UNTIL @bDone END REPEAT;

	CLOSE curs;
	SELECT * FROM ItemData;
END";
			migrationBuilder.Sql(sp);

			sp = @"CREATE PROCEDURE `ComicAllItemNavigationData`(IN `comicId` INT, IN `exclude` TEXT)
    READS SQL DATA
BEGIN
	DECLARE itemId SMALLINT;
	DECLARE curs CURSOR FOR SELECT id FROM items;
	DECLARE CONTINUE HANDLER FOR NOT FOUND SET @bDone = 1;

	CREATE TEMPORARY TABLE ItemData
		(Id SMALLINT, First INT, Previous INT, Next INT, Last INT, Count INT);

	OPEN curs;

	SET @bDone = 0;
	FETCH curs INTO itemId;
	REPEAT
		BEGIN
			DECLARE CONTINUE HANDLER FOR NOT FOUND BEGIN END;

			SET @first = NULL;
			SET @previous = NULL;
			SET @next = NULL;
			SET @last = NULL;
			SET @count = NULL;

			IF exclude = 'guest' THEN
				SELECT
					MIN(c.id),
					MAX(c.id),
					COUNT(c.id)
				INTO @first, @last, @count
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.isGuestComic = false;
			ELSEIF exclude = 'non-canon' THEN
				SELECT
					MIN(c.id),
					MAX(c.id),
					COUNT(c.id)
				INTO @first, @last, @count
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.isNonCanon = false;
			ELSE
				SELECT
					MIN(c.id),
					MAX(c.id),
					COUNT(c.id)
				INTO @first, @last, @count
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId;
			END IF;

			IF exclude = 'guest' THEN
				SELECT c.id INTO @previous
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id < comicId
					AND c.isGuestComic = false
				ORDER BY c.id DESC
				LIMIT 1;
			ELSEIF exclude = 'non-canon' THEN
				SELECT c.id INTO @previous
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id < comicId
					AND c.isNonCanon = false
				ORDER BY c.id DESC
				LIMIT 1;
			ELSE
			SELECT c.id INTO @previous
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id < comicId
				ORDER BY c.id DESC
				LIMIT 1;
			END IF;

			IF exclude = 'guest' THEN
				SELECT c.id INTO @next
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id > comicId
					AND c.isGuestComic = false
				ORDER BY c.id ASC
				LIMIT 1;
			ELSEIF exclude = 'non-canon' THEN
				SELECT c.id INTO @next
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id > comicId
					AND c.isNonCanon = false
				ORDER BY c.id ASC
				LIMIT 1;
			ELSE
				SELECT c.id INTO @next
				FROM items i
				JOIN occurences o ON o.items_id = i.id
				JOIN comic c ON c.id = o.comic_id
				WHERE i.id = itemId
					AND c.id > comicId
				ORDER BY c.id ASC
				LIMIT 1;
			END IF;
		END;

		INSERT INTO ItemData
			(Id, First, Previous, Next, Last, Count)
		VALUES
			(itemId, @first, @previous, @next, @last, @count);

		FETCH curs INTO itemId;
	UNTIL @bDone END REPEAT;

	CLOSE curs;
	SELECT * FROM ItemData;
END";
			migrationBuilder.Sql(sp);
		}

		protected override void Down(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.Sql("DROP PROCEDURE IF EXISTS `ComicItemNavigationData`");
			migrationBuilder.Sql("DROP PROCEDURE IF EXISTS `ComicAllItemNavigationData`");
		}
	}
}
