using Microsoft.EntityFrameworkCore.Migrations;

namespace QCExtensions.Persistence.Migrations
{
	public partial class MissingFeatureFlags : Migration
	{
		protected override void Up(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.AddColumn<bool>(
				name: "HasNoCast",
				table: "comic",
				nullable: false,
				defaultValue: false);

			migrationBuilder.AddColumn<bool>(
				name: "HasNoLocation",
				table: "comic",
				nullable: false,
				defaultValue: false);

			migrationBuilder.AddColumn<bool>(
				name: "HasNoStoryline",
				table: "comic",
				nullable: false,
				defaultValue: false);

			migrationBuilder.AddColumn<bool>(
				name: "HasNoTagline",
				table: "comic",
				nullable: false,
				defaultValue: false);

			migrationBuilder.AddColumn<bool>(
				name: "HasNoTitle",
				table: "comic",
				nullable: false,
				defaultValue: false);

			migrationBuilder.Sql("DROP PROCEDURE `ComicEditorData`");
			var sp = @"
CREATE PROCEDURE `ComicEditorData`(IN `comicId` SMALLINT)
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
	    AND NOT c.HasNoTagline
		AND c.id > 3132;

	SELECT c.id INTO @previous
	FROM comic c
	WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
	    AND NOT c.HasNoTagline
		AND c.id < comicId
		AND c.id > 3132
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE (c.tagline IS NULL OR NULLIF(c.tagline, '') IS NULL)
	    AND NOT c.HasNoTagline
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
	WHERE (c.title IS NULL or NULLIF(c.title, '') IS NULL)
	    AND NOT c.HasNoTitle;

	SELECT c.id INTO @previous
	FROM comic c
	WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
	    AND NOT c.HasNoTitle
		AND c.id < comicId
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE (c.title IS NULL OR NULLIF(c.title, '') IS NULL)
	    AND NOT c.HasNoTitle
		AND c.id > comicId
	ORDER BY c.id ASC
	LIMIT 1;

	INSERT INTO TypeData
		(Type, First, Previous, Next, Last)
	VALUES
		('title', @first, @previous, @next, @last);

	SELECT c.id INTO @first
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'cast'
		GROUP BY ci.id
	)
		AND NOT c.HasNoCast
	ORDER BY c.id ASC
	LIMIT 1;

	SELECT c.id INTO @previous
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'cast'
		GROUP BY ci.id
	)
		AND c.id < comicId
		AND NOT c.HasNoCast
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'cast'
		GROUP BY ci.id
	)
		AND c.id > comicId
		AND NOT c.HasNoCast
	ORDER BY c.id ASC
	LIMIT 1;

	SELECT c.id INTO @last
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'cast'
		GROUP BY ci.id
	)
		AND NOT c.HasNoCast
	ORDER BY c.id DESC
	LIMIT 1;

	INSERT INTO TypeData
		(Type, First, Previous, Next, Last)
	VALUES
		('cast', @first, @previous, @next, @last);

	SELECT c.id INTO @first
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'location'
		GROUP BY ci.id
	)
		AND NOT c.HasNoLocation
	ORDER BY c.id ASC
	LIMIT 1;

	SELECT c.id INTO @previous
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'location'
		GROUP BY ci.id
	)
		AND c.id < comicId
		AND NOT c.HasNoLocation
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'location'
		GROUP BY ci.id
	)
		AND c.id > comicId
		AND NOT c.HasNoLocation
	ORDER BY c.id ASC
	LIMIT 1;

	SELECT c.id INTO @last
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'location'
		GROUP BY ci.id
	)
		AND NOT c.HasNoLocation
	ORDER BY c.id DESC
	LIMIT 1;

	INSERT INTO TypeData
		(Type, First, Previous, Next, Last)
	VALUES
		('location', @first, @previous, @next, @last);

	SELECT c.id INTO @first
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'storyline'
		GROUP BY ci.id
	)
		AND NOT c.HasNoStoryline
	ORDER BY c.id ASC
	LIMIT 1;

	SELECT c.id INTO @previous
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'storyline'
		GROUP BY ci.id
	)
		AND c.id < comicId
		AND NOT c.HasNoStoryline
	ORDER BY c.id DESC
	LIMIT 1;

	SELECT c.id INTO @next
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'storyline'
		GROUP BY ci.id
	)
		AND c.id > comicId
		AND NOT c.HasNoStoryline
	ORDER BY c.id ASC
	LIMIT 1;

	SELECT c.id INTO @last
	FROM comic c
	WHERE c.id NOT IN (
		SELECT ci.id
		FROM comic ci
		JOIN occurences o ON o.comic_id = ci.id
		LEFT JOIN items i ON o.items_id = i.id
		WHERE i.type = 'storyline'
		GROUP BY ci.id
	)
		AND NOT c.HasNoStoryline
	ORDER BY c.id DESC
	LIMIT 1;

	INSERT INTO TypeData
		(Type, First, Previous, Next, Last)
	VALUES
		('storyline', @first, @previous, @next, @last);

	SELECT * FROM TypeData;
END";
			migrationBuilder.Sql(sp);
		}

		protected override void Down(MigrationBuilder migrationBuilder)
		{
			migrationBuilder.DropColumn(
				name: "HasNoCast",
				table: "comic");

			migrationBuilder.DropColumn(
				name: "HasNoLocation",
				table: "comic");

			migrationBuilder.DropColumn(
				name: "HasNoStoryline",
				table: "comic");

			migrationBuilder.DropColumn(
				name: "HasNoTagline",
				table: "comic");

			migrationBuilder.DropColumn(
				name: "HasNoTitle",
				table: "comic");

			migrationBuilder.Sql("DROP PROCEDURE `ComicEditorData`");
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
	}
}
