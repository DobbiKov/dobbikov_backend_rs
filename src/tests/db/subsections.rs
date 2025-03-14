use crate::db::{
    self,
    sections::CreateSectionForm,
    subsections::{
        create_subsection, delete_subsection, delete_subsections,
        get_max_subsection_position_in_section, get_subsection, get_subsections, swap_subsections,
        update_subsections, CreateSubsectionForm, DeleteSubsectionsError, GetSubsectionsForm,
        SubsectionFromDb, SwapSubsectionsError, UpdateSubsectionForm, UpdateSubsectionsError,
    },
};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn create_subsection_test() {
    let pool: sqlx::Pool<sqlx::MySql> = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section to attach subsections
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;

    // Ensure no subsections exist for section 1 yet
    let subs = get_subsections(
        &pool,
        GetSubsectionsForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(subs.is_ok());
    assert_eq!(subs.unwrap().len(), 0);

    // Check that the max position is None for section 1
    let max_pos = get_max_subsection_position_in_section(&pool, 1).await;
    assert!(max_pos.is_none());

    // Create the first subsection in section 1
    let res = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    assert!(res.is_ok());

    // Now max position should be 0
    let max_pos = get_max_subsection_position_in_section(&pool, 1).await;
    assert_eq!(max_pos, Some(0));

    // Create a second subsection in section 1
    let res = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection B".to_string(),
            section_id: 1,
        },
    )
    .await;
    assert!(res.is_ok());

    // Now max position should be 1
    let max_pos = get_max_subsection_position_in_section(&pool, 1).await;
    assert_eq!(max_pos, Some(1));

    // Get all subsections in section 1 and verify their details
    let subs = get_subsections(
        &pool,
        GetSubsectionsForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(subs.is_ok());
    let mut subs_vec = subs.unwrap();
    // sort by position to ensure order
    subs_vec.sort_by_key(|s| s.position);
    assert_eq!(subs_vec.len(), 2);
    assert_eq!(
        subs_vec[0],
        SubsectionFromDb {
            id: 1,
            title: "Subsection A".to_string(),
            position: 0,
            section_id: 1,
        }
    );
    assert_eq!(
        subs_vec[1],
        SubsectionFromDb {
            id: 2,
            title: "Subsection B".to_string(),
            position: 1,
            section_id: 1,
        }
    );

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn update_subsection_test() {
    let pool: sqlx::Pool<sqlx::MySql> = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and two subsections
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection B".to_string(),
            section_id: 1,
        },
    )
    .await;

    // Try to update a non-existing subsection (id: 90)
    let res = update_subsections(
        &pool,
        UpdateSubsectionForm {
            title: Some("Updated A".to_string()),
            section_id: None,
            position: None,
        },
        GetSubsectionsForm {
            id: Some(90),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(res.unwrap_err(), UpdateSubsectionsError::NotFoundError);

    // Attempt an update with nothing to update (all fields None)
    let res = update_subsections(
        &pool,
        UpdateSubsectionForm {
            title: None,
            section_id: None,
            position: None,
        },
        GetSubsectionsForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert_eq!(
        res.unwrap_err(),
        UpdateSubsectionsError::NothingToUpdateError
    );

    // Update the title of subsection with id 1
    let res = update_subsections(
        &pool,
        UpdateSubsectionForm {
            title: Some("Updated A".to_string()),
            section_id: None,
            position: None,
        },
        GetSubsectionsForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(res.is_ok());

    // Verify update using get_subsection
    let sub = get_subsection(
        &pool,
        GetSubsectionsForm {
            id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(sub.is_ok());
    let sub = sub.unwrap();
    assert_eq!(sub.title, "Updated A".to_string());

    // Update all subsections in section 1 to have a global title update
    let res = update_subsections(
        &pool,
        UpdateSubsectionForm {
            title: Some("Global Update".to_string()),
            section_id: None,
            position: None,
        },
        GetSubsectionsForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(res.is_ok());

    let subs = get_subsections(
        &pool,
        GetSubsectionsForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(subs.is_ok());
    for s in subs.unwrap() {
        assert_eq!(s.title, "Global Update".to_string());
    }

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn swap_subsections_test() {
    let pool: sqlx::Pool<sqlx::MySql> = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and two subsections within it
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection B".to_string(),
            section_id: 1,
        },
    )
    .await;

    // Verify initial positions (expect 0 and 1)
    let subs = get_subsections(
        &pool,
        GetSubsectionsForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(subs.is_ok());
    let subs_vec = subs.unwrap();
    let sub_a = subs_vec.iter().find(|s| s.id == 1).unwrap();
    let sub_b = subs_vec.iter().find(|s| s.id == 2).unwrap();
    assert_eq!(sub_a.position, 0);
    assert_eq!(sub_b.position, 1);

    // Test error cases for swap: non-existent id(s)
    let res = swap_subsections(&pool, [1, 90]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSubsectionsError::NotFoundError((None, Some(90)))
    );

    let res = swap_subsections(&pool, [90, 1]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSubsectionsError::NotFoundError((Some(90), None))
    );

    let res = swap_subsections(&pool, [90, 91]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSubsectionsError::NotFoundError((Some(90), Some(91)))
    );

    // Create another section and a subsection in that section to test cross-section swap
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 2".to_string(),
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection C".to_string(),
            section_id: 2,
        },
    )
    .await;

    // Attempt to swap subsection id 1 (section 1) with subsection id 3 (section 2)
    let res = swap_subsections(&pool, [1, 3]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSubsectionsError::CantSwapFromDifferentSections
    );

    // Now perform a valid swap within section 1
    let res = swap_subsections(&pool, [1, 2]).await;
    assert!(res.is_ok());

    // Verify that positions have been swapped:
    let subs = get_subsections(
        &pool,
        GetSubsectionsForm {
            section_id: Some(1),
            ..Default::default()
        },
    )
    .await;
    assert!(subs.is_ok());
    let subs_vec = subs.unwrap();
    let sub_a = subs_vec.iter().find(|s| s.id == 1).unwrap();
    let sub_b = subs_vec.iter().find(|s| s.id == 2).unwrap();
    // Originally, sub_a.position was 0 and sub_b.position was 1.
    // After swapping, sub_a should have position 1 and sub_b position 0.
    assert_eq!(sub_a.position, 1);
    assert_eq!(sub_b.position, 0);

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn delete_subsections_test() {
    let pool: sqlx::Pool<sqlx::MySql> = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and two subsections
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection B".to_string(),
            section_id: 1,
        },
    )
    .await;

    // Confirm that two subsections exist
    let subs = get_subsections(&pool, Default::default()).await;
    assert!(subs.is_ok());
    assert_eq!(subs.unwrap().len(), 2);

    // Delete all subsections using an unfiltered form
    let res = delete_subsections(&pool, Default::default()).await;
    assert!(res.is_ok());

    // Verify deletion
    let subs = get_subsections(&pool, Default::default()).await;
    assert!(subs.is_ok());
    assert_eq!(subs.unwrap().len(), 0);

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn delete_one_subsection_test() {
    let pool: sqlx::Pool<sqlx::MySql> = match db::establish_connection_for_testing().await {
        Ok(conn) => conn,
        Err(_) => panic!("An error occurred"),
    };

    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    // Create a section and two subsections
    let _ = db::sections::create_section(
        &pool,
        CreateSectionForm {
            title: "Section 1".to_string(),
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection A".to_string(),
            section_id: 1,
        },
    )
    .await;
    let _ = create_subsection(
        &pool,
        CreateSubsectionForm {
            title: "Subsection B".to_string(),
            section_id: 1,
        },
    )
    .await;

    // Confirm that two subsections exist
    let subs = get_subsections(&pool, Default::default()).await;
    assert!(subs.is_ok());
    assert_eq!(subs.unwrap().len(), 2);

    // Delete one subsection using the delete_subsection (limit 1) function
    let res = delete_subsection(&pool, Default::default()).await;
    assert!(res.is_ok());

    // Verify that only one subsection remains
    let subs = get_subsections(&pool, Default::default()).await;
    assert!(subs.is_ok());
    assert_eq!(subs.unwrap().len(), 1);

    db::create_tables::drop_all_tables(&pool).await;
}
