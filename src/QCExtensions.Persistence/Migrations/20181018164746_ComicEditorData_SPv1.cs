using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Persistence.Migrations
{
	public partial class ComicEditorData_SPv1 : Migration
	{
		protected override void Up(MigrationBuilder migrationBuilder)
		{
			CreateComicEditorData(migrationBuilder);
		}

		public static void CreateComicEditorData(MigrationBuilder migrationBuilder)
		{
			var sp = @"CREATE PROCEDURE `ComicEditorData`(IN `comicId` SMALLINT)
	READS SQL DATA
BEGIN
	CREATE TEMPORARY TABLE TypeData
		(Type VARCHAR(255), First INT, Previous INT, Next INT, Last INT, Count INT);

	SELECT
		MIN(c.id),
		MAX(c.id)
	INTO @first, @last
	FROM comic c
	WHERE (c.tagline IS NULL or NULLIF(c.tagline, '') IS NULL)
		AND c.id > 3132;

	SELECT c.id INTO @previous
	FROM comic c
	WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
		AND c.id < comicId
		AND c.id > 3132
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
		AND c.id > comicId
		AND c.id > 3132
	ORDER BY c.id ASC
	LIMIT 1;

	INSERT INTO TypeData
		(Type, First, Previous, Next, Last)
	VALUES
		('tagline', @first, @previous, @next, @last);

	SELECT
		MIN(c.id),
		MAX(c.id)
	INTO
		@first,
		@last
	FROM comic c
	WHERE (c.title IS NULL or NULLIF(c.title, '') IS NULL);

	SELECT c.id INTO @previous
	FROM comic c
	WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
		AND c.id < comicId
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
		AND c.id > comicId
	ORDER BY c.id ASC
	LIMIT 1;

	INSERT INTO TypeData
		(Type, First, Previous, Next, Last)
	VALUES
		('title', @first, @previous, @next, @last);

	SET @types = 'cast,location,storyline,';
	WHILE (LOCATE(',', @types) > 0)
	DO
		SET @value = SUBSTRING(@types,1, LOCATE(',',@types)-1);
		SET @types= SUBSTRING(@types, LOCATE(',',@types) + 1);

		SELECT c.id INTO @first
		FROM comic c
		WHERE c.id NOT IN (
			SELECT ci.id
			FROM comic ci
			JOIN occurences o ON o.comic_id = ci.id
			LEFT JOIN items i ON o.items_id = i.id
			WHERE i.type = @value
			GROUP BY ci.id
		)
		ORDER BY c.id ASC
		LIMIT 1;

		SELECT c.id INTO @previous
		FROM comic c
		WHERE c.id NOT IN (
			SELECT ci.id
			FROM comic ci
			JOIN occurences o ON o.comic_id = ci.id
			LEFT JOIN items i ON o.items_id = i.id
			WHERE i.type = @value
			GROUP BY ci.id
		)
			AND c.id < comicId
		ORDER BY c.id DESC
		LIMIT 1;

		SELECT c.id INTO @next
		FROM comic c
		WHERE c.id NOT IN (
			SELECT ci.id
			FROM comic ci
			JOIN occurences o ON o.comic_id = ci.id
			LEFT JOIN items i ON o.items_id = i.id
			WHERE i.type = @value
			GROUP BY ci.id
		)
			AND c.id > comicId
		ORDER BY c.id ASC
		LIMIT 1;

		SELECT c.id INTO @last
		FROM comic c
		WHERE c.id NOT IN (
			SELECT ci.id
			FROM comic ci
			JOIN occurences o ON o.comic_id = ci.id
			LEFT JOIN items i ON o.items_id = i.id
			WHERE i.type = @value
			GROUP BY ci.id
		)
		ORDER BY c.id DESC
		LIMIT 1;

		INSERT INTO TypeData
			(Type, First, Previous, Next, Last)
		VALUES
			(@value, @first, @previous, @next, @last);
	END WHILE;

	SELECT * FROM TypeData;
END";
			migrationBuilder.Sql(sp);
		}

		protected override void Down(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.Sql("DROP PROCEDURE IF EXISTS `ComicEditorData`");
		}
	}
}
