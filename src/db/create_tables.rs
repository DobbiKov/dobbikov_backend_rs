async fn create_users_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
        CREATE TABLE IF NOT EXISTS users(\
            id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
            username TEXT NOT NULL,\
            password TEXT NOT NULL,\
            is_admin TINYINT(1) DEFAULT 0\
            );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_sessions_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
        CREATE TABLE IF NOT EXISTS sessions(\
            id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
            user_id INT UNSIGNED NOT NULL,\
            token VARCHAR(128) NOT NULL UNIQUE,\
            expires_at BIGINT NOT NULL,\
            FOREIGN KEY (user_id) REFERENCES users(id) ON DELETE CASCADE\
        );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_sections_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
         CREATE TABLE IF NOT EXISTS sections (\
             id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
             title VARCHAR(255) NOT NULL,\
             position INT UNSIGNED NOT NULL UNIQUE\
         );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_subsections_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
                     CREATE TABLE IF NOT EXISTS subsections ( \
                         id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY, \
                         title VARCHAR(255) NOT NULL, \
                         position INT UNSIGNED NOT NULL, \
                         section_id INT UNSIGNED NOT NULL, \
                         FOREIGN KEY (section_id) REFERENCES sections(id), \
                         UNIQUE (position, section_id) \
                     ); \
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_notes_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
        CREATE TABLE IF NOT EXISTS notes (\
            id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
            name TEXT NOT NULL,\
            description TEXT NOT NULL,\
            url TEXT NOT NULL,\
            position INT UNSIGNED,\
            section_id INT UNSIGNED,\
            subsection_id INT UNSIGNED,\
            FOREIGN KEY (section_id) REFERENCES sections(id),\
            FOREIGN KEY (subsection_id) REFERENCES subsections(id),\
            UNIQUE (position, subsection_id)\
        );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_tags_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
        CREATE TABLE IF NOT EXISTS tags (\
            id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
            name VARCHAR(64) NOT NULL UNIQUE,\
            color VARCHAR(7) NOT NULL\
        );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_note_tags_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
        CREATE TABLE IF NOT EXISTS note_tags (\
            note_id INT UNSIGNED NOT NULL,\
            tag_id INT UNSIGNED NOT NULL,\
            PRIMARY KEY (note_id, tag_id),\
            FOREIGN KEY (note_id) REFERENCES notes(id) ON DELETE CASCADE,\
            FOREIGN KEY (tag_id) REFERENCES tags(id) ON DELETE CASCADE\
        );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

pub async fn notes_description_column_exists(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<bool, sqlx::Error> {
    let query = sqlx::query_scalar::<_, i64>(
        "\
        SELECT COUNT(*) FROM INFORMATION_SCHEMA.COLUMNS \
        WHERE TABLE_SCHEMA = DATABASE() \
          AND TABLE_NAME = 'notes' \
          AND COLUMN_NAME = 'description'\
        ",
    );
    let count = query.fetch_one(pool).await?;
    Ok(count > 0)
}

pub async fn ensure_notes_description_column_exists(
    pool: &sqlx::Pool<sqlx::MySql>,
) -> Result<bool, sqlx::Error> {
    if notes_description_column_exists(pool).await? {
        return Ok(false);
    }

    sqlx::query("ALTER TABLE notes ADD COLUMN description TEXT AFTER name")
        .execute(pool)
        .await?;

    if notes_description_column_exists(pool).await? {
        return Ok(true);
    }

    Err(sqlx::Error::Protocol(
        "notes.description column is still missing after ALTER TABLE".into(),
    ))
}

pub async fn create_required_tables(pool: &sqlx::Pool<sqlx::MySql>) {
    create_users_table(pool).await;
    create_sessions_table(pool).await;
    create_sections_table(pool).await;
    create_subsections_table(pool).await;
    create_notes_table(pool).await;
    create_tags_table(pool).await;
    create_note_tags_table(pool).await;
}
pub async fn drop_all_tables(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_strs = [
        "DROP TABLE note_tags;",
        "DROP TABLE tags;",
        "DROP TABLE notes;",
        "DROP TABLE subsections;",
        "DROP TABLE sections;",
        "DROP TABLE sessions;",
        "DROP TABLE users;",
    ];
    let table_names = [
        "note_tags",
        "tags",
        "notes",
        "subsections",
        "sections",
        "sessions",
        "users",
    ];
    for table_name in table_names {
        let query_str = format!("DROP TABLE IF EXISTS {} ;", table_name);
        let _ = sqlx::query(query_str.as_str()).execute(pool).await;
    }
}
