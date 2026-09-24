use crate::db::{
    self,
    lecture_notes::{create_note, delete_note, get_notes, CreateNoteForm, GetNotesForm},
    sections::CreateSectionForm,
    subsections::CreateSubsectionForm,
    tags::{
        add_tag_to_note, create_tag, delete_tag, find_missing_tag_ids, get_tag, get_tags,
        get_tags_for_notes, remove_tag_from_note, set_note_tags, update_tag, AssignTagsError,
        CreateTagError, CreateTagForm, GetTagError, TagFromDb, UpdateTagError, UpdateTagForm,
    },
};
use crate::services::tags::{normalize_tag_color, normalize_tag_name, TagValidationError};

async fn setup_pool() -> sqlx::Pool<sqlx::MySql> {
    let pool = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;
    pool
}

/// Create a section, a subsection and two notes (ids 1 and 2).
async fn create_two_notes(pool: &sqlx::Pool<sqlx::MySql>) {
    let _ = db::sections::create_section(
        pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    for i in 1..=2 {
        let id = create_note(
            pool,
            CreateNoteForm {
                name: format!("Note {}", i),
                description: format!("Description {}", i),
                url: format!("http://note{}.com", i),
                section_id: Some(1),
                subsection_id: Some(1),
            },
        )
        .await;
        assert_eq!(id, Ok(i));
    }
}

fn tag_form(name: &str, color: &str) -> CreateTagForm {
    CreateTagForm {
        name: name.to_string(),
        color: color.to_string(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn create_and_get_tags_test() {
    let pool = setup_pool().await;

    assert_eq!(get_tags(&pool).await.unwrap().len(), 0);

    assert_eq!(create_tag(&pool, tag_form("math", "#ff0000")).await, Ok(1));
    assert_eq!(
        create_tag(&pool, tag_form("algebra", "#00ff00")).await,
        Ok(2)
    );

    // Names are unique.
    assert_eq!(
        create_tag(&pool, tag_form("math", "#0000ff")).await,
        Err(CreateTagError::AlreadyExistsError)
    );

    // Tags are ordered by name.
    let tags = get_tags(&pool).await.unwrap();
    assert_eq!(
        tags,
        vec![
            TagFromDb {
                id: 2,
                name: "algebra".to_string(),
                color: "#00ff00".to_string(),
            },
            TagFromDb {
                id: 1,
                name: "math".to_string(),
                color: "#ff0000".to_string(),
            },
        ]
    );

    assert_eq!(get_tag(&pool, 1).await.unwrap().name, "math");
    assert_eq!(get_tag(&pool, 42).await, Err(GetTagError::NotFoundError));
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn update_tag_test() {
    let pool = setup_pool().await;
    let _ = create_tag(&pool, tag_form("math", "#ff0000")).await;
    let _ = create_tag(&pool, tag_form("physics", "#00ff00")).await;

    assert_eq!(
        update_tag(
            &pool,
            1,
            UpdateTagForm {
                name: None,
                color: None,
            },
        )
        .await,
        Err(UpdateTagError::NothingToUpdateError)
    );
    assert_eq!(
        update_tag(
            &pool,
            42,
            UpdateTagForm {
                name: Some("x".to_string()),
                color: None,
            },
        )
        .await,
        Err(UpdateTagError::NotFoundError)
    );
    assert_eq!(
        update_tag(
            &pool,
            1,
            UpdateTagForm {
                name: Some("physics".to_string()),
                color: None,
            },
        )
        .await,
        Err(UpdateTagError::AlreadyExistsError)
    );

    // Only the color changes.
    assert!(update_tag(
        &pool,
        1,
        UpdateTagForm {
            name: None,
            color: Some("#123456".to_string()),
        },
    )
    .await
    .is_ok());
    let tag = get_tag(&pool, 1).await.unwrap();
    assert_eq!(tag.name, "math");
    assert_eq!(tag.color, "#123456");

    assert!(update_tag(
        &pool,
        1,
        UpdateTagForm {
            name: Some("analysis".to_string()),
            color: Some("#abcdef".to_string()),
        },
    )
    .await
    .is_ok());
    let tag = get_tag(&pool, 1).await.unwrap();
    assert_eq!(tag.name, "analysis");
    assert_eq!(tag.color, "#abcdef");
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn assign_tags_to_notes_test() {
    let pool = setup_pool().await;
    create_two_notes(&pool).await;
    let _ = create_tag(&pool, tag_form("math", "#ff0000")).await;
    let _ = create_tag(&pool, tag_form("algebra", "#00ff00")).await;
    let _ = create_tag(&pool, tag_form("exam", "#0000ff")).await;

    // Missing note / missing tags are reported.
    assert_eq!(
        set_note_tags(&pool, 42, &[1]).await,
        Err(AssignTagsError::NoteNotFoundError)
    );
    assert_eq!(
        set_note_tags(&pool, 1, &[1, 7, 5, 7]).await,
        Err(AssignTagsError::TagsNotFoundError(vec![5, 7]))
    );
    assert_eq!(
        find_missing_tag_ids(&pool, &[1, 2, 9]).await.unwrap(),
        vec![9]
    );

    // Set, with a duplicate id which is ignored.
    assert!(set_note_tags(&pool, 1, &[1, 2, 2]).await.is_ok());
    assert!(set_note_tags(&pool, 2, &[3]).await.is_ok());

    let tags = get_tags_for_notes(&pool, None).await.unwrap();
    let names = |note_id: u32| -> Vec<String> {
        tags.get(&note_id)
            .map(|list| list.iter().map(|tag| tag.name.clone()).collect())
            .unwrap_or_default()
    };
    assert_eq!(names(1), vec!["algebra", "math"]);
    assert_eq!(names(2), vec!["exam"]);

    // Replacing the set drops tags not in the new list.
    assert!(set_note_tags(&pool, 1, &[3]).await.is_ok());
    let tags = get_tags_for_notes(&pool, Some(&[1])).await.unwrap();
    assert_eq!(tags.get(&1).unwrap().len(), 1);
    assert_eq!(tags.get(&1).unwrap()[0].id, 3);
    assert!(tags.get(&2).is_none());

    // Add / remove single tags; both are idempotent.
    assert!(add_tag_to_note(&pool, 1, 1).await.is_ok());
    assert!(add_tag_to_note(&pool, 1, 1).await.is_ok());
    assert_eq!(
        add_tag_to_note(&pool, 1, 42).await,
        Err(AssignTagsError::TagsNotFoundError(vec![42]))
    );
    let tags = get_tags_for_notes(&pool, Some(&[1])).await.unwrap();
    assert_eq!(tags.get(&1).unwrap().len(), 2);
    assert!(remove_tag_from_note(&pool, 1, 1).await.is_ok());
    assert!(remove_tag_from_note(&pool, 1, 1).await.is_ok());
    let tags = get_tags_for_notes(&pool, Some(&[1])).await.unwrap();
    assert_eq!(tags.get(&1).unwrap().len(), 1);

    // Empty list clears all tags.
    assert!(set_note_tags(&pool, 1, &[]).await.is_ok());
    let tags = get_tags_for_notes(&pool, Some(&[1])).await.unwrap();
    assert!(tags.get(&1).is_none());
    assert!(get_tags_for_notes(&pool, Some(&[]))
        .await
        .unwrap()
        .is_empty());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn filter_notes_by_tag_test() {
    let pool = setup_pool().await;
    create_two_notes(&pool).await;
    let _ = create_tag(&pool, tag_form("math", "#ff0000")).await;
    let _ = create_tag(&pool, tag_form("exam", "#0000ff")).await;
    let _ = set_note_tags(&pool, 1, &[1, 2]).await;
    let _ = set_note_tags(&pool, 2, &[2]).await;

    let by_tag = |tag_id: u32| GetNotesForm {
        tag_id: Some(tag_id),
        ..Default::default()
    };
    let notes = get_notes(&pool, by_tag(1)).await.unwrap();
    assert_eq!(notes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![1]);
    let notes = get_notes(&pool, by_tag(2)).await.unwrap();
    assert_eq!(notes.len(), 2);

    // Combined with other filters.
    let notes = get_notes(
        &pool,
        GetNotesForm {
            tag_id: Some(2),
            name: Some("Note 2".to_string()),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    assert_eq!(notes.iter().map(|n| n.id).collect::<Vec<_>>(), vec![2]);
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn cascade_delete_test() {
    let pool = setup_pool().await;
    create_two_notes(&pool).await;
    let _ = create_tag(&pool, tag_form("math", "#ff0000")).await;
    let _ = create_tag(&pool, tag_form("exam", "#0000ff")).await;
    let _ = set_note_tags(&pool, 1, &[1, 2]).await;
    let _ = set_note_tags(&pool, 2, &[1]).await;

    // Deleting a tag removes it from notes.
    assert!(delete_tag(&pool, 1).await.is_ok());
    let tags = get_tags_for_notes(&pool, None).await.unwrap();
    assert_eq!(tags.get(&1).unwrap().len(), 1);
    assert!(tags.get(&2).is_none());

    // Deleting a note removes its assignments but keeps the tag.
    assert!(delete_note(
        &pool,
        GetNotesForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await
    .is_ok());
    assert!(get_tags_for_notes(&pool, None).await.unwrap().is_empty());
    assert_eq!(get_tags(&pool).await.unwrap().len(), 1);
}

#[test]
fn normalize_tag_color_test() {
    assert_eq!(normalize_tag_color("#1E90FF"), Ok("#1e90ff".to_string()));
    assert_eq!(normalize_tag_color(" #abc "), Ok("#aabbcc".to_string()));
    assert_eq!(
        normalize_tag_color("1e90ff"),
        Err(TagValidationError::InvalidColor)
    );
    assert_eq!(
        normalize_tag_color("#12345"),
        Err(TagValidationError::InvalidColor)
    );
    assert_eq!(
        normalize_tag_color("#gggggg"),
        Err(TagValidationError::InvalidColor)
    );
    assert_eq!(
        normalize_tag_color("red"),
        Err(TagValidationError::InvalidColor)
    );
}

#[test]
fn normalize_tag_name_test() {
    assert_eq!(normalize_tag_name("  math "), Ok("math".to_string()));
    assert_eq!(
        normalize_tag_name("   "),
        Err(TagValidationError::EmptyName)
    );
    assert_eq!(
        normalize_tag_name(&"a".repeat(65)),
        Err(TagValidationError::NameTooLong)
    );
    assert!(normalize_tag_name(&"é".repeat(64)).is_ok());
}
