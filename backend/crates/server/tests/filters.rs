use chrono::NaiveDate;
use db::filter::{FilterTarget, Subject, TagScope};
use server::filters::parse_filters;
use uuid::Uuid;

fn expressions(raw: &[&str]) -> Vec<String> {
    raw.iter()
        .map(|expression| expression.to_string())
        .collect()
}

#[test]
fn parses_alternatives_exclusion_and_separate_clauses() {
    let artist_a = Uuid::now_v7();
    let artist_b = Uuid::now_v7();
    let tag_a = Uuid::now_v7();
    let raw = expressions(&[
        &format!("singer:{artist_a},{artist_b}"),
        &format!("-genre:{tag_a}"),
        &format!("singer:{artist_b}"),
    ]);

    let parsed = parse_filters(&raw, Subject::Performance).unwrap();

    assert_eq!(parsed.clauses.len(), 3);
    assert_eq!(parsed.clauses[0].target, FilterTarget::Singer);
    assert_eq!(parsed.clauses[0].ids, vec![artist_a, artist_b]);
    assert!(!parsed.clauses[0].exclude);
    assert!(parsed.clauses[1].exclude);
    assert_eq!(parsed.clauses[2].ids, vec![artist_b]);
}

#[test]
fn misc_reads_performance_and_song_tags() {
    let raw = expressions(&[&format!("misc:{}", Uuid::now_v7())]);

    let parsed = parse_filters(&raw, Subject::Performance).unwrap();

    assert_eq!(
        parsed.clauses[0].target,
        FilterTarget::Tag {
            kind: "misc",
            scope: TagScope::Either,
        }
    );
}

#[test]
fn date_ranges_narrow_bounds_and_allow_open_ends() {
    let raw = expressions(&[
        "date:2024-01-01..",
        "date:..2024-12-31",
        "date:2024-03-01..2024-06-30",
    ]);

    let parsed = parse_filters(&raw, Subject::Performance).unwrap();

    assert_eq!(parsed.date_from, NaiveDate::from_ymd_opt(2024, 3, 1));
    assert_eq!(parsed.date_to, NaiveDate::from_ymd_opt(2024, 6, 30));
    assert!(parsed.clauses.is_empty());
}

#[test]
fn rejects_invalid_expressions() {
    let id = Uuid::now_v7();
    let invalid = [
        "genre".to_string(),
        format!("mood:{id}"),
        "genre:,".to_string(),
        "genre:not-an-id".to_string(),
        format!("-date:2024-01-01..{id}"),
        "-date:2024-01-01..2024-02-01".to_string(),
        "date:2024-01-01".to_string(),
        "date:2024-13-01..".to_string(),
    ];

    for expression in invalid {
        assert!(
            parse_filters(std::slice::from_ref(&expression), Subject::Performance).is_err(),
            "{expression} should be rejected"
        );
    }
}

#[test]
fn songs_reject_performance_only_fields() {
    let id = Uuid::now_v7();

    for field in ["song", "instrument", "modifier"] {
        let raw = expressions(&[&format!("{field}:{id}")]);
        assert!(parse_filters(&raw, Subject::Song).is_err(), "{field}");
    }
    assert!(parse_filters(&expressions(&["date:2024-01-01.."]), Subject::Song).is_err());

    let raw = expressions(&[&format!("genre:{id}"), &format!("misc:{id}")]);
    assert!(parse_filters(&raw, Subject::Song).is_ok());
}
