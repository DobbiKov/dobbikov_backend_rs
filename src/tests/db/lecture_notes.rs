use crate::db::{
    self,
    lecture_notes::{
        create_note, delete_note, delete_notes, get_max_note_position_in_subsection, get_note,
        get_notes, swap_notes, update_notes, CreateNoteForm, SwapNotesError, UpdateNoteForm,
        UpdateNotesError,
    },
    sections::CreateSectionForm,
    subsections::CreateSubsectionForm,
};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn create_note_test() {
    let pool = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    // Setup fresh tables.
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and a subsection for note attachment.
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;

    // Ensure no notes exist yet.
    let notes = get_notes(&pool, Default::default()).await;
    assert!(notes.is_ok());
    assert_eq!(notes.unwrap().len(), 0);

    // For subsection 1, max note position should be None.
    let max_pos = get_max_note_position_in_subsection(&pool, 1).await;
    assert!(max_pos.is_none());

    // Create the first note in subsection 1.
    let res = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 1".to_string(),
            url: "http://note1.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;
    assert!(res.is_ok());

    // Now max position for subsection 1 should be 0.
    let max_pos = get_max_note_position_in_subsection(&pool, 1).await;
    assert_eq!(max_pos, Some(0));

    // Create a second note in the same subsection.
    let res = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 2".to_string(),
            url: "http://note2.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;
    assert!(res.is_ok());

    // Now max position should be 1.
    let max_pos = get_max_note_position_in_subsection(&pool, 1).await;
    assert_eq!(max_pos, Some(1));

    // Create a note without specifying a subsection (global note).
    let res = create_note(
        &pool,
        CreateNoteForm {
            name: "Global Note".to_string(),
            url: "http://globalnote.com".to_string(),
            section_id: None,
            subsection_id: None,
        },
    )
    .await;
    assert!(res.is_ok());

    // The global max position (across all notes) should now be 2.
    let query_str = "SELECT MAX(position) FROM notes";
    let query = sqlx::query_scalar(query_str);
    let global_max: Option<u32> = query.fetch_one(&pool).await.unwrap();
    assert_eq!(global_max, Some(2));

    // Fetch and sort all notes by position to verify details.
    let mut notes_vec = get_notes(&pool, Default::default()).await.unwrap();
    notes_vec.sort_by_key(|n| n.position);
    assert_eq!(notes_vec.len(), 3);
    assert_eq!(notes_vec[0].name, "Note 1");
    assert_eq!(notes_vec[0].position, 0);
    assert_eq!(notes_vec[1].name, "Note 2");
    assert_eq!(notes_vec[1].position, 1);
    assert_eq!(notes_vec[2].name, "Global Note");
    assert_eq!(notes_vec[2].position, 2);

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn update_note_test() {
    let pool = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    // Setup fresh tables.
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and a subsection, then two notes.
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 1".to_string(),
            url: "http://note1.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 2".to_string(),
            url: "http://note2.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;

    // Try to update a non-existing note (id: 90).
    let res = update_notes(
        &pool,
        UpdateNoteForm {
            name: Some("Updated Note".to_string()),
            url: None,
            section_id: None,
            subsection_id: None,
            position: None,
        },
        crate::db::lecture_notes::GetNotesForm {
            id: Some(90),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(res.unwrap_err(), UpdateNotesError::NotFoundError);

    // Attempt an update with nothing to update (all fields None).
    let res = update_notes(
        &pool,
        UpdateNoteForm {
            name: None,
            url: None,
            section_id: None,
            subsection_id: None,
            position: None,
        },
        crate::db::lecture_notes::GetNotesForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(res.unwrap_err(), UpdateNotesError::NothingToUpdateError);

    // Update the name of note with id 1.
    let res = update_notes(
        &pool,
        UpdateNoteForm {
            name: Some("Updated Note 1".to_string()),
            url: None,
            section_id: None,
            subsection_id: None,
            position: None,
        },
        crate::db::lecture_notes::GetNotesForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(res.is_ok());

    // Verify update using get_note.
    let note = get_note(
        &pool,
        crate::db::lecture_notes::GetNotesForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(note.is_ok());
    let note = note.unwrap();
    assert_eq!(note.name, "Updated Note 1".to_string());

    // Update all notes (global update) to have a new URL.
    let res = update_notes(
        &pool,
        UpdateNoteForm {
            name: None,
            url: Some("http://updatedurl.com".to_string()),
            section_id: None,
            subsection_id: None,
            position: None,
        },
        crate::db::lecture_notes::GetNotesForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(res.is_ok());

    let notes = get_notes(
        &pool,
        crate::db::lecture_notes::GetNotesForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    for n in notes {
        assert_eq!(n.url, "http://updatedurl.com".to_string());
    }

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn swap_notes_test() {
    let pool = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    // Setup fresh tables.
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and a subsection then two notes in that subsection.
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 1".to_string(),
            url: "http://note1.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 2".to_string(),
            url: "http://note2.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;

    // Verify initial positions.
    let notes = get_notes(
        &pool,
        crate::db::lecture_notes::GetNotesForm {
            subsection_id: Some(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let note1 = notes.iter().find(|n| n.id == 1).unwrap();
    let note2 = notes.iter().find(|n| n.id == 2).unwrap();
    assert_eq!(note1.position, 0);
    assert_eq!(note2.position, 1);

    // Test error cases for swap: non-existent note ID.
    let res = swap_notes(&pool, [1, 90]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapNotesError::NotFoundError((None, Some(90)))
    );

    let res = swap_notes(&pool, [90, 1]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapNotesError::NotFoundError((Some(90), None))
    );

    // Create another section and a different subsection with a note to test cross-subsection swap.
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 2".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection B".to_string(),
            section_id: 2,
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 3".to_string(),
            url: "http://note3.com".to_string(),
            section_id: Some(2),
            subsection_id: Some(2),
        },
    )
    .await;

    // Attempt to swap note 1 (subsection 1) with note 3 (subsection 2).
    let res = swap_notes(&pool, [1, 3]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapNotesError::CantSwapFromDifferentSubsections
    );

    // Now perform a valid swap within subsection 1.
    let res = swap_notes(&pool, [1, 2]).await;
    assert!(res.is_ok());

    // Verify that positions have been swapped.
    let notes = get_notes(
        &pool,
        crate::db::lecture_notes::GetNotesForm {
            subsection_id: Some(1),
            ..Default::default()
        },
    )
    .await
    .unwrap();
    let note1 = notes.iter().find(|n| n.id == 1).unwrap();
    let note2 = notes.iter().find(|n| n.id == 2).unwrap();
    assert_eq!(note1.position, 1);
    assert_eq!(note2.position, 0);

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn delete_notes_test() {
    let pool = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    // Setup fresh tables.
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section, subsection, and two notes.
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 1".to_string(),
            url: "http://note1.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 2".to_string(),
            url: "http://note2.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;

    // Confirm that two notes exist.
    let notes = get_notes(&pool, Default::default()).await;
    assert!(notes.is_ok());
    assert_eq!(notes.unwrap().len(), 2);

    // Delete all notes using an unfiltered form.
    let res = delete_notes(&pool, Default::default()).await;
    assert!(res.is_ok());

    // Verify deletion.
    let notes = get_notes(&pool, Default::default()).await;
    assert!(notes.is_ok());
    assert_eq!(notes.unwrap().len(), 0);

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn delete_one_note_test() {
    let pool = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    // Setup fresh tables.
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section, subsection, and two notes.
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = db::subsections::create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 1".to_string(),
            url: "http://note1.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;
    let _ = create_note(
        &pool,
        CreateNoteForm {
            name: "Note 2".to_string(),
            url: "http://note2.com".to_string(),
            section_id: Some(1),
            subsection_id: Some(1),
        },
    )
    .await;

    // Confirm that two notes exist.
    let notes = get_notes(&pool, Default::default()).await;
    assert!(notes.is_ok());
    assert_eq!(notes.unwrap().len(), 2);

    // Delete one note (using limit 1).
    let res = delete_note(&pool, Default::default()).await;
    assert!(res.is_ok());

    // Verify that only one note remains.
    let notes = get_notes(&pool, Default::default()).await;
    assert!(notes.is_ok());
    assert_eq!(notes.unwrap().len(), 1);

    db::create_tables::drop_all_tables(&pool).await;
}
