use journal::commands::{entries, tags};
use journal::db::DbConnection;

fn setup_db() -> DbConnection {
    DbConnection::new_memory().expect("Failed to create in-memory test DB")
}

#[test]
fn integration_entry_crud_flow() {
    let db = setup_db();

    let created = entries::create_entry(&db, "First integration entry".to_string(), Some(3))
        .expect("create_entry should succeed");
    assert_eq!(created.body, "First integration entry");
    assert_eq!(created.mood, Some(3));

    let updated = entries::update_entry(
        &db,
        created.id.clone(),
        "Updated integration entry".to_string(),
        Some(5),
    )
    .expect("update_entry should succeed");
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

    let entry = entries::create_entry(&db, "Taggable entry".to_string(), None)
        .expect("create_entry should succeed");
    let tag1 = tags::create_tag(&db, "work".to_string()).expect("create_tag should succeed");
    let tag2 = tags::create_tag(&db, "reflection".to_string()).expect("create_tag should succeed");

    tags::assign_tag_to_entry(&db, entry.id.clone(), tag1.id.clone())
        .expect("assign_tag_to_entry should succeed");
    tags::assign_tag_to_entry(&db, entry.id.clone(), tag2.id.clone())
        .expect("assign_tag_to_entry should succeed");

    let entry_tags = tags::get_tags_for_entry(&db, entry.id.clone())
        .expect("get_tags_for_entry should succeed");
    assert_eq!(entry_tags.len(), 2);

    tags::remove_tag_from_entry(&db, entry.id.clone(), tag1.id.clone())
        .expect("remove_tag_from_entry should succeed");

    let entry_tags_after_remove = tags::get_tags_for_entry(&db, entry.id)
        .expect("get_tags_for_entry should succeed");
    assert_eq!(entry_tags_after_remove.len(), 1);
    assert_eq!(entry_tags_after_remove[0].name, "reflection");
}

#[test]
fn integration_deleted_entries_not_returned() {
    let db = setup_db();

    let keep = entries::create_entry(&db, "Keep me".to_string(), None)
        .expect("create_entry should succeed");
    let remove = entries::create_entry(&db, "Delete me".to_string(), None)
        .expect("create_entry should succeed");

    entries::delete_entry(&db, remove.id).expect("delete_entry should succeed");

    let results = entries::get_entries(&db).expect("get_entries should succeed");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, keep.id);
}
