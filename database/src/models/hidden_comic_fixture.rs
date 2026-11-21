//! A database fixture of visible comics plus one hidden (advance) comic, for tests asserting
//! that public queries leave hidden comics out.

use sqlx::MySqlConnection;

pub const VISIBLE_COMICS: u16 = 60;
pub const HIDDEN_COMIC: u16 = 1000;

pub const CAST_A: u16 = 1;
pub const CAST_B: u16 = 2;
pub const CAST_C: u16 = 3;
pub const CAST_NEW: u16 = 4;
pub const CAST_NEW_2: u16 = 5;
pub const LOCATION_L: u16 = 10;
pub const LOCATION_NEW: u16 = 11;

/// Defines a `#[sqlx::test]` that runs `$query` with only visible comics, after adding a hidden
/// comic, and after publishing it, and asserts that only publishing changes the output.
macro_rules! hidden_comic_is_excluded {
    ($name:ident, |$conn:ident| $query:expr) => {
        #[sqlx::test]
        async fn $name(pool: sqlx::MySqlPool) -> sqlx::Result<()> {
            let mut $conn = pool.acquire().await?;

            $crate::models::hidden_comic_fixture::insert_visible_comics(&mut $conn).await?;
            let visible_only = format!("{:?}", $query);

            $crate::models::hidden_comic_fixture::insert_hidden_comic(&mut $conn).await?;
            let with_hidden = format!("{:?}", $query);

            $crate::models::Comic::unhide_by_id(
                &mut *$conn,
                $crate::models::hidden_comic_fixture::HIDDEN_COMIC,
            )
            .await?;
            let published = format!("{:?}", $query);

            assert_eq!(
                visible_only, with_hidden,
                "the hidden comic changed the output"
            );
            assert_ne!(
                with_hidden, published,
                "publishing the hidden comic did not change the output, so the fixture does not \
                 exercise this query"
            );
            Ok(())
        }
    };
}
pub(crate) use hidden_comic_is_excluded;

/// Visible comics 1..=60, each with a different gap to the next one (1 to 60 days, spread over
/// several years), cast A, B and location L in every comic, and cast C in every second one.
pub async fn insert_visible_comics(conn: &mut MySqlConnection) -> sqlx::Result<()> {
    for (id, name, kind) in [
        (CAST_A, "A", "cast"),
        (CAST_B, "B", "cast"),
        (CAST_C, "C", "cast"),
        (CAST_NEW, "New", "cast"),
        (CAST_NEW_2, "New 2", "cast"),
        (LOCATION_L, "L", "location"),
        (LOCATION_NEW, "New location", "location"),
    ] {
        sqlx::query("INSERT INTO `Item` (`id`, `short_name`, `name`, `type`) VALUES (?, ?, ?, ?)")
            .bind(id)
            .bind(name)
            .bind(name)
            .bind(kind)
            .execute(&mut *conn)
            .await?;
    }

    for comic in 1..=VISIBLE_COMICS {
        let days = u32::from(comic) * (u32::from(comic) + 1) / 2;
        insert_comic(conn, comic, days, false).await?;

        let mut items = vec![CAST_A, CAST_B, LOCATION_L];
        if comic % 2 == 0 {
            items.push(CAST_C);
        }
        insert_occurrences(conn, comic, &items).await?;
    }
    Ok(())
}

/// A hidden comic 400 days after the last visible one, with a milestone id, the debuts of two
/// cast members and a location, more cast than any visible comic, and cast B without A, which
/// makes B the top-ranked character once it is published.
pub async fn insert_hidden_comic(conn: &mut MySqlConnection) -> sqlx::Result<()> {
    let last_visible = u32::from(VISIBLE_COMICS) * (u32::from(VISIBLE_COMICS) + 1) / 2;
    insert_comic(conn, HIDDEN_COMIC, last_visible + 400, true).await?;
    insert_occurrences(
        conn,
        HIDDEN_COMIC,
        &[
            CAST_B,
            CAST_C,
            CAST_NEW,
            CAST_NEW_2,
            LOCATION_L,
            LOCATION_NEW,
        ],
    )
    .await
}

pub async fn insert_occurrences(
    conn: &mut MySqlConnection,
    comic: u16,
    items: &[u16],
) -> sqlx::Result<()> {
    for &item in items {
        sqlx::query("INSERT INTO `Occurrence` (`comic_id`, `item_id`) VALUES (?, ?)")
            .bind(comic)
            .bind(item)
            .execute(&mut *conn)
            .await?;
    }
    Ok(())
}

async fn insert_comic(
    conn: &mut MySqlConnection,
    id: u16,
    days_after_start: u32,
    hidden: bool,
) -> sqlx::Result<()> {
    sqlx::query(
        r"
            INSERT INTO `Comic` (`id`, `title`, `publish_date`, `hidden`)
            VALUES (?, ?, DATE_ADD('2010-01-04 12:00:00', INTERVAL ? DAY), ?)
        ",
    )
    .bind(id)
    .bind(format!("Comic {id}"))
    .bind(days_after_start)
    .bind(hidden)
    .execute(&mut *conn)
    .await?;
    Ok(())
}
