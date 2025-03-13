use crate::db::{
    self,
    sections::{
        GetSectionsForm, SectionFromDb, SwapSectionsError, UpdateSectionForm, UpdateSectionsError,
    },
};

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn create_section_test() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    assert_eq!(sections.unwrap_or_default().len(), 0);

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 1);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "title haha".to_string(),
        },
    )
    .await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 1
        }
    );
    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn get_max_position_test() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let max_pos = db::sections::get_max_position(&pool).await;
    assert!(max_pos.is_none());

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;

    let max_pos = db::sections::get_max_position(&pool).await;
    assert!(max_pos.is_some());
    assert_eq!(max_pos.unwrap(), 0);

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;

    let max_pos = db::sections::get_max_position(&pool).await;
    assert!(max_pos.is_some());
    assert_eq!(max_pos.unwrap(), 1);

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn update_section_test() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    assert_eq!(sections.unwrap_or_default().len(), 0);

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 1);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "title haha".to_string(),
        },
    )
    .await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 1
        }
    );

    // try to update
    // doesn't exist
    let res = db::sections::update_sections(
        &pool,
        UpdateSectionForm {
            title: Some("update_title_for_1".to_string()),
        },
        GetSectionsForm {
            id: Some(90),
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;
    assert_eq!(res.unwrap_err(), UpdateSectionsError::NotFoundError);

    // all none
    let res = db::sections::update_sections(
        &pool,
        UpdateSectionForm { title: None },
        GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;
    assert_eq!(res.unwrap_err(), UpdateSectionsError::NothingToUpdateError);

    //actually updates
    let res = db::sections::update_sections(
        &pool,
        UpdateSectionForm {
            title: Some("update_title_for_1".to_string()),
        },
        GetSectionsForm {
            id: Some(1),
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;
    println!("{:?}", res);
    assert!(res.is_ok());

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "update_title_for_1".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 1
        }
    );

    // update all
    //actually updates
    let res = db::sections::update_sections(
        &pool,
        UpdateSectionForm {
            title: Some("same_title?".to_string()),
        },
        GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;
    println!("{:?}", res);
    assert!(res.is_ok());

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "same_title?".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "same_title?".to_string(),
            position: 1
        }
    );

    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn limit_test() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    assert_eq!(sections.unwrap_or_default().len(), 0);

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "title haha".to_string(),
        },
    )
    .await;

    // limit: none
    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            limit: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 1
        }
    );

    //limit: 2
    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            limit: Some(2),
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 1
        }
    );

    //limit: 1
    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            limit: Some(1),
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 1);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );
    db::create_tables::drop_all_tables(&pool).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
pub async fn swap_sections() {
    let pool: sqlx::Pool<sqlx::MySql>;

    match db::establish_connection_for_testing().await {
        Ok(conn) => pool = conn,
        Err(_) => {
            panic!("an error occured")
        }
    };
    db::create_tables::drop_all_tables(&pool).await;
    db::create_tables::create_required_tables(&pool).await;

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "new_title_0".to_string(),
        },
    )
    .await;

    let _ = db::sections::create_section(
        &pool,
        db::sections::CreateSectionForm {
            title: "title haha".to_string(),
        },
    )
    .await;

    // get sections
    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 0
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 1
        }
    );

    //checking errors
    let res = db::sections::swap_sections(&pool, [1, 14]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSectionsError::NotFoundError((None, Some(14)))
    );

    let res = db::sections::swap_sections(&pool, [14, 1]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSectionsError::NotFoundError((Some(14), None))
    );

    let res = db::sections::swap_sections(&pool, [14, 15]).await;
    assert!(res.is_err());
    assert_eq!(
        res.unwrap_err(),
        SwapSectionsError::NotFoundError((Some(14), Some(15)))
    );

    //verify the sections' positions are really swapped
    let res = db::sections::swap_sections(&pool, [1, 2]).await;
    assert!(res.is_ok());
    let sections = db::sections::get_sections(
        &pool,
        db::sections::GetSectionsForm {
            id: None,
            title: None,
            position: None,
            ..Default::default()
        },
    )
    .await;

    assert!(sections.is_ok());
    let sections_vec = sections.unwrap_or_default();
    assert_eq!(sections_vec.len(), 2);
    assert_eq!(
        sections_vec[0],
        db::sections::SectionFromDb {
            id: 1,
            title: "new_title_0".to_string(),
            position: 1
        }
    );
    assert_eq!(
        sections_vec[1],
        db::sections::SectionFromDb {
            id: 2,
            title: "title haha".to_string(),
            position: 0
        }
    );
    db::create_tables::drop_all_tables(&pool).await;
}
