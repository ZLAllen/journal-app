use chrono::{Datelike, TimeZone};
use journal::commands::{entries, search, tags};
use journal::db::DbConnection;
use std::fs;
use uuid::Uuid;

fn setup_db() -> DbConnection {
    DbConnection::new_memory().expect("Failed to create in-memory test DB")
}

#[test]
fn integration_entry_crud_flow() {
    let db = setup_db();

    let created = entries::create_entry(
        &db,
        "First integration title".to_string(),
        "First integration entry".to_string(),
        Some(3),
    )
    .expect("create_entry should succeed");
    assert_eq!(created.title, "First integration title");
    assert_eq!(created.body, "First integration entry");
    assert_eq!(created.mood, Some(3));

    let updated = entries::update_entry(
        &db,
        created.id.clone(),
        "Updated integration title".to_string(),
        "Updated integration entry".to_string(),
        Some(5),
        None,
    )
    .expect("update_entry should succeed");
    assert_eq!(updated.title, "Updated integration title");
    assert_eq!(updated.body, "Updated integration entry");
    assert_eq!(updated.mood, Some(5));

    let all = entries::get_entries(&db).expect("get_entries should succeed");
    assert_eq!(all.len(), 1);
    assert_eq!(all[0].id, created.id);

    entries::delete_entry(&db, created.id.clone()).expect("delete_entry should succeed");

    let all_after_delete = entries::get_entries(&db).expect("get_entries should succeed");
    assert_eq!(all_after_delete.len(), 0);
}

#[test]
fn integration_tags_assignment_flow() {
    let db = setup_db();

    let entry = entries::create_entry(
        &db,
        "Taggable entry".to_string(),
        "Taggable body".to_string(),
        None,
    )
    .expect("create_entry should succeed");
    let tag1 = tags::create_tag(&db, "work".to_string()).expect("create_tag should succeed");
    let tag2 = tags::create_tag(&db, "reflection".to_string()).expect("create_tag should succeed");

    tags::assign_tag_to_entry(&db, entry.id.clone(), tag1.id.clone())
        .expect("assign_tag_to_entry should succeed");
    tags::assign_tag_to_entry(&db, entry.id.clone(), tag2.id.clone())
        .expect("assign_tag_to_entry should succeed");

    let entry_tags =
        tags::get_tags_for_entry(&db, entry.id.clone()).expect("get_tags_for_entry should succeed");
    assert_eq!(entry_tags.len(), 2);

    tags::remove_tag_from_entry(&db, entry.id.clone(), tag1.id.clone())
        .expect("remove_tag_from_entry should succeed");

    let entry_tags_after_remove =
        tags::get_tags_for_entry(&db, entry.id).expect("get_tags_for_entry should succeed");
    assert_eq!(entry_tags_after_remove.len(), 1);
    assert_eq!(entry_tags_after_remove[0].name, "reflection");
}

#[test]
fn integration_deleted_entries_not_returned() {
    let db = setup_db();

    let keep = entries::create_entry(&db, "Keep me".to_string(), "Keep me body".to_string(), None)
        .expect("create_entry should succeed");
    let remove = entries::create_entry(
        &db,
        "Delete me".to_string(),
        "Delete me body".to_string(),
        None,
    )
    .expect("create_entry should succeed");

    entries::delete_entry(&db, remove.id).expect("delete_entry should succeed");

    let results = entries::get_entries(&db).expect("get_entries should succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, keep.id);
}

#[test]
fn integration_deleted_entries_excluded_from_search_results() {
    let db = setup_db();

    let keep = entries::create_entry(
        &db,
        "Keep".to_string(),
        "<p>visible keyword entry</p>".to_string(),
        None,
    )
    .expect("create_entry should succeed");
    let remove = entries::create_entry(
        &db,
        "Remove".to_string(),
        "<p>visible keyword removed</p>".to_string(),
        None,
    )
    .expect("create_entry should succeed");

    entries::delete_entry(&db, remove.id).expect("delete_entry should succeed");

    let search_results = search::search_entries(&db, "keyword".to_string(), Some(20), Some(0))
        .expect("search_entries should succeed");

    assert_eq!(search_results.results.len(), 1);
    assert_eq!(search_results.results[0].entry.id, keep.id);
}

#[test]
fn integration_get_entry_returns_none_after_delete() {
    let db = setup_db();

    let entry = entries::create_entry(
        &db,
        "Lookup".to_string(),
        "<p>Lookup body</p>".to_string(),
        None,
    )
    .expect("create_entry should succeed");

    let loaded = entries::get_entry(&db, entry.id.clone()).expect("get_entry should succeed");
    assert!(loaded.is_some());

    entries::delete_entry(&db, entry.id.clone()).expect("delete_entry should succeed");

    let after_delete = entries::get_entry(&db, entry.id).expect("get_entry should succeed");
    assert!(after_delete.is_none());
}

#[test]
fn integration_formatting_persists_after_db_reopen() {
    let db_path = std::env::temp_dir().join(format!("journal-test-{}.db", Uuid::new_v4()));
    let db_path_str = db_path.to_string_lossy().to_string();

    {
        let db = DbConnection::new(&db_path_str).expect("Failed to create file-backed DB");
        entries::create_entry(
            &db,
            "Formatted".to_string(),
            "<p>Hello <strong>world</strong> <em>today</em></p>".to_string(),
            Some(4),
        )
        .expect("create_entry should succeed");
    }

    {
        let reopened = DbConnection::new(&db_path_str).expect("Failed to reopen file-backed DB");
        let all = entries::get_entries(&reopened).expect("get_entries should succeed");
        assert_eq!(all.len(), 1);
        assert_eq!(
            all[0].body_html,
            "<p>Hello <strong>world</strong> <em>today</em></p>"
        );
    }

    fs::remove_file(db_path).expect("Failed to remove temporary DB file");
}

#[test]
fn integration_summary_stats_returns_expected_aggregates() {
    let db = setup_db();

    let now = chrono::Local::now();
    let today_ms = chrono::Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 0, 0)
        .single()
        .expect("valid today")
        .timestamp_millis();
    let yesterday_ms = (chrono::Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 0, 0)
        .single()
        .expect("valid yesterday anchor")
        - chrono::Duration::days(1))
    .timestamp_millis();
    let two_days_ago_ms = (chrono::Local
        .with_ymd_and_hms(now.year(), now.month(), now.day(), 9, 0, 0)
        .single()
        .expect("valid two days anchor")
        - chrono::Duration::days(2))
    .timestamp_millis();

    let entry_a = entries::create_entry(
        &db,
        "A".to_string(),
        "<p>alpha beta gamma</p>".to_string(),
        Some(4),
    )
    .expect("create A");
    let entry_b = entries::create_entry(
        &db,
        "B".to_string(),
        "<p>delta epsilon</p>".to_string(),
        Some(3),
    )
    .expect("create B");
    let entry_c = entries::create_entry(&db, "C".to_string(), "<p>zeta</p>".to_string(), Some(2))
        .expect("create C");
    let entry_deleted = entries::create_entry(
        &db,
        "Deleted".to_string(),
        "<p>should not count</p>".to_string(),
        Some(1),
    )
    .expect("create deleted");

    entries::update_entry(
        &db,
        entry_a.id.clone(),
        entry_a.title.clone(),
        entry_a.body.clone(),
        entry_a.mood,
        Some(today_ms),
    )
    .expect("set date A");
    entries::update_entry(
        &db,
        entry_b.id.clone(),
        entry_b.title.clone(),
        entry_b.body.clone(),
        entry_b.mood,
        Some(yesterday_ms),
    )
    .expect("set date B");
    entries::update_entry(
        &db,
        entry_c.id.clone(),
        entry_c.title.clone(),
        entry_c.body.clone(),
        entry_c.mood,
        Some(two_days_ago_ms),
    )
    .expect("set date C");
    entries::update_entry(
        &db,
        entry_deleted.id.clone(),
        entry_deleted.title.clone(),
        entry_deleted.body.clone(),
        entry_deleted.mood,
        Some(today_ms),
    )
    .expect("set date deleted");

    entries::delete_entry(&db, entry_deleted.id.clone()).expect("delete entry");

    let tag_focus = tags::create_tag(&db, "focus".to_string()).expect("create focus");
    let tag_work = tags::create_tag(&db, "work".to_string()).expect("create work");

    tags::assign_tag_to_entry(&db, entry_a.id.clone(), tag_focus.id.clone()).expect("tag A focus");
    tags::assign_tag_to_entry(&db, entry_b.id.clone(), tag_focus.id.clone()).expect("tag B focus");
    tags::assign_tag_to_entry(&db, entry_c.id, tag_work.id.clone()).expect("tag C work");

    let stats = journal::commands::stats::get_summary_stats(&db).expect("stats");

    assert_eq!(stats.writing_streak_days, 3);
    assert_eq!(stats.total_entries, 3);
    assert_eq!(stats.total_word_count, 6);
    assert_eq!(stats.entries_this_month, 3);
    assert_eq!(stats.top_tags.len(), 2);
    assert_eq!(stats.top_tags[0].name, "focus");
    assert_eq!(stats.top_tags[0].usage_count, 2);
    assert_eq!(stats.top_tags[1].name, "work");
    assert_eq!(stats.top_tags[1].usage_count, 1);
}

#[test]
fn integration_tag_rename_delete_updates_tags_without_deleting_entries() {
    let db = setup_db();

    let entry = entries::create_entry(
        &db,
        "Tag lifecycle".to_string(),
        "<p>entry should survive tag changes</p>".to_string(),
        Some(4),
    )
    .expect("create entry");

    let original_tag = tags::create_tag(&db, "Work".to_string()).expect("create tag");
    tags::assign_tag_to_entry(&db, entry.id.clone(), original_tag.id.clone()).expect("assign tag");

    let renamed = tags::rename_tag(&db, original_tag.id.clone(), "Deep Work".to_string())
        .expect("rename tag");
    assert_eq!(renamed.name, "Deep Work");

    let tags_after_rename = tags::get_tags_for_entry(&db, entry.id.clone()).expect("get tags");
    assert_eq!(tags_after_rename.len(), 1);
    assert_eq!(tags_after_rename[0].id, original_tag.id);
    assert_eq!(tags_after_rename[0].name, "Deep Work");

    let loaded_after_rename = entries::get_entry(&db, entry.id.clone()).expect("get entry");
    assert!(loaded_after_rename.is_some());
    assert_eq!(
        loaded_after_rename.expect("entry exists").title,
        "Tag lifecycle".to_string()
    );

    tags::delete_tag(&db, original_tag.id.clone()).expect("delete tag");

    let tags_after_delete = tags::get_tags_for_entry(&db, entry.id.clone()).expect("get tags");
    assert!(tags_after_delete.is_empty());

    let loaded_after_delete = entries::get_entry(&db, entry.id).expect("get entry");
    assert!(loaded_after_delete.is_some());
    assert_eq!(
        loaded_after_delete.expect("entry exists").title,
        "Tag lifecycle".to_string()
    );
}

#[test]
fn integration_list_entries_supports_cursor_and_filters() {
    let db = setup_db();

    let e1 = entries::create_entry(&db, "Pinned".to_string(), "<p>one</p>".to_string(), Some(5))
        .expect("create e1");
    let e2 = entries::create_entry(&db, "Mood3".to_string(), "<p>two</p>".to_string(), Some(3))
        .expect("create e2");
    let e3 = entries::create_entry(
        &db,
        "Mood2".to_string(),
        "<p>three</p>".to_string(),
        Some(2),
    )
    .expect("create e3");

    let _ = entries::set_pinned(&db, e1.id.clone(), true).expect("set pinned");
    entries::delete_entry(&db, e3.id.clone()).expect("soft delete e3");

    let first_page = entries::list_entries(&db, None, Some(1), None).expect("list page 1");
    assert_eq!(first_page.entries.len(), 1);
    assert_eq!(first_page.entries[0].id, e1.id);
    assert!(first_page.next_cursor.is_some());

    let second_page =
        entries::list_entries(&db, first_page.next_cursor, Some(1), None).expect("list page 2");
    assert_eq!(second_page.entries.len(), 1);
    assert_eq!(second_page.entries[0].id, e2.id);
    assert!(second_page.next_cursor.is_none());

    let mood_filtered = entries::list_entries(
        &db,
        None,
        Some(10),
        Some(entries::ListEntriesFilters {
            mood: Some(5),
            ..Default::default()
        }),
    )
    .expect("list mood filtered");
    assert_eq!(mood_filtered.entries.len(), 1);
    assert_eq!(mood_filtered.entries[0].id, e1.id);
}
