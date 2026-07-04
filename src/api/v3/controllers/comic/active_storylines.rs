use crate::api::v3::models::{ActiveStorylineSummary, StorylineFactSegment};
use crate::models::ComicId;
use actix_web::{Result, error};
use database::DbPoolConnection;
use database::models::{ActiveStorylineOccurrence, Comic as DatabaseComic, Item as DatabaseItem};
use std::convert::TryInto;

/// Fetches every storyline active at `comic_id` and pre-aggregates its
/// attachment history into an RLE of featured/gap segments, so the client
/// only has to do the (tunable) *visual* compression.
#[tracing::instrument(skip(conn))]
pub async fn fetch_active_storylines(
    conn: &mut DbPoolConnection,
    comic_id: ComicId,
) -> Result<Vec<ActiveStorylineSummary>> {
    let occurrences =
        DatabaseItem::active_storylines_by_comic_id(&mut **conn, comic_id.into_inner())
            .await
            .map_err(error::ErrorInternalServerError)?;

    let latest_comic_id = DatabaseComic::latest_id(&mut **conn)
        .await
        .map_err(error::ErrorInternalServerError)?
        .unwrap_or_else(|| comic_id.into_inner());

    group_into_summaries(&occurrences, latest_comic_id)
        .into_iter()
        .map(|group| {
            Ok(ActiveStorylineSummary {
                id: group.item_id.into(),
                start_comic_id: group
                    .start_comic_id
                    .try_into()
                    .expect("database has valid comicIds"),
                end_comic_id: group
                    .end_comic_id
                    .map(TryInto::try_into)
                    .transpose()
                    .expect("database has valid comicIds"),
                segments: build_segments(
                    group.start_comic_id,
                    group.end_comic_id,
                    latest_comic_id,
                    &group.occurrence_comic_ids,
                ),
            })
        })
        .collect()
}

struct StorylineGroup {
    item_id: u16,
    start_comic_id: u16,
    end_comic_id: Option<u16>,
    occurrence_comic_ids: Vec<u16>,
}

/// Groups the flat (one row per occurrence, or one `NULL`-occurrence row for
/// storylines with no attachment in range) query result by `item_id`.
/// Relies on the query's `ORDER BY item_id, comic_id`.
fn group_into_summaries(
    occurrences: &[ActiveStorylineOccurrence],
    _latest_comic_id: u16,
) -> Vec<StorylineGroup> {
    let mut groups: Vec<StorylineGroup> = Vec::new();

    for occurrence in occurrences {
        match groups.last_mut() {
            Some(group) if group.item_id == occurrence.item_id => {
                if let Some(comic_id) = occurrence.occurrence_comic_id {
                    group.occurrence_comic_ids.push(comic_id);
                }
            }
            _ => {
                groups.push(StorylineGroup {
                    item_id: occurrence.item_id,
                    start_comic_id: occurrence.start_comic_id,
                    end_comic_id: occurrence.end_comic_id,
                    occurrence_comic_ids: occurrence.occurrence_comic_id.into_iter().collect(),
                });
            }
        }
    }

    groups
}

/// Builds a contiguous RLE of featured/gap segments spanning
/// `[start_comic_id, right_bound)`, where `right_bound` is `end_comic_id` if
/// set, or one past `latest_comic_id` otherwise (so an open-ended storyline's
/// segments cover up to and including the latest known comic).
fn build_segments(
    start_comic_id: u16,
    end_comic_id: Option<u16>,
    latest_comic_id: u16,
    occurrence_comic_ids: &[u16],
) -> Vec<StorylineFactSegment> {
    let right_bound = end_comic_id.unwrap_or_else(|| latest_comic_id.saturating_add(1));

    if right_bound <= start_comic_id {
        return Vec::new();
    }

    let mut segments = Vec::new();
    let mut cursor = start_comic_id;
    let mut remaining = occurrence_comic_ids
        .iter()
        .copied()
        .filter(|&c| c >= start_comic_id && c < right_bound)
        .peekable();

    while cursor < right_bound {
        if remaining.peek() == Some(&cursor) {
            let from = cursor;
            while remaining.peek() == Some(&cursor) {
                remaining.next();
                cursor += 1;
            }
            segments.push(segment(from, cursor, true));
        } else {
            let to = remaining.peek().copied().unwrap_or(right_bound);
            segments.push(segment(cursor, to, false));
            cursor = to;
        }
    }

    segments
}

fn segment(from_comic_id: u16, to_comic_id: u16, featured: bool) -> StorylineFactSegment {
    StorylineFactSegment {
        from_comic_id: from_comic_id
            .try_into()
            .expect("database has valid comicIds"),
        to_comic_id: to_comic_id.try_into().expect("database has valid comicIds"),
        featured,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn occ(
        item_id: u16,
        start: u16,
        end: Option<u16>,
        comic: Option<u16>,
    ) -> ActiveStorylineOccurrence {
        ActiveStorylineOccurrence {
            item_id,
            start_comic_id: start,
            end_comic_id: end,
            occurrence_comic_id: comic,
        }
    }

    #[test]
    fn fully_featured_span_is_one_segment() {
        let segments = build_segments(10, Some(13), 100, &[10, 11, 12]);
        assert_eq!(
            segments,
            vec![StorylineFactSegment {
                from_comic_id: 10_u16.try_into().unwrap(),
                to_comic_id: 13_u16.try_into().unwrap(),
                featured: true,
            }]
        );
    }

    #[test]
    fn single_gap_between_featured_runs() {
        let segments = build_segments(1, Some(10), 100, &[1, 2, 8, 9]);
        assert_eq!(
            segments,
            vec![
                StorylineFactSegment {
                    from_comic_id: 1_u16.try_into().unwrap(),
                    to_comic_id: 3_u16.try_into().unwrap(),
                    featured: true,
                },
                StorylineFactSegment {
                    from_comic_id: 3_u16.try_into().unwrap(),
                    to_comic_id: 8_u16.try_into().unwrap(),
                    featured: false,
                },
                StorylineFactSegment {
                    from_comic_id: 8_u16.try_into().unwrap(),
                    to_comic_id: 10_u16.try_into().unwrap(),
                    featured: true,
                },
            ]
        );
    }

    #[test]
    fn no_occurrences_is_a_single_gap() {
        let segments = build_segments(5, Some(10), 100, &[]);
        assert_eq!(
            segments,
            vec![StorylineFactSegment {
                from_comic_id: 5_u16.try_into().unwrap(),
                to_comic_id: 10_u16.try_into().unwrap(),
                featured: false,
            }]
        );
    }

    #[test]
    fn open_ended_uses_latest_comic_id_plus_one_as_right_bound() {
        let segments = build_segments(5, None, 7, &[5, 6, 7]);
        assert_eq!(
            segments,
            vec![StorylineFactSegment {
                from_comic_id: 5_u16.try_into().unwrap(),
                to_comic_id: 8_u16.try_into().unwrap(),
                featured: true,
            }]
        );
    }

    #[test]
    fn degenerate_span_yields_no_segments() {
        let segments = build_segments(10, Some(10), 100, &[]);
        assert_eq!(segments, Vec::new());
    }

    #[test]
    fn groups_multiple_items_and_drops_null_occurrence_placeholder() {
        let occurrences = vec![
            occ(1, 5, None, Some(5)),
            occ(1, 5, None, Some(6)),
            occ(2, 8, Some(20), None),
        ];

        let groups = group_into_summaries(&occurrences, 100);

        assert_eq!(groups.len(), 2);
        assert_eq!(groups[0].item_id, 1);
        assert_eq!(groups[0].occurrence_comic_ids, vec![5, 6]);
        assert_eq!(groups[1].item_id, 2);
        assert_eq!(groups[1].occurrence_comic_ids, Vec::<u16>::new());
    }
}
