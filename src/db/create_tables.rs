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
        CREATE TABLE IF NOT EXISTS subsections (\
            id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
            title VARCHAR(255) NOT NULL,\
            position INT UNSIGNED UNIQUE,\
            section_id INT UNSIGNED NOT NULL,\
            FOREIGN KEY (section_id) REFERENCES sections(id)\
        );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

async fn create_notes_table(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_str = "\
        CREATE TABLE IF NOT EXISTS notes (\
            id INT UNSIGNED AUTO_INCREMENT PRIMARY KEY,\
            name TEXT NOT NULL,\
            url TEXT NOT NULL,\
            position INT UNSIGNED UNIQUE,\
            section_id INT UNSIGNED,\
            subsection_id INT UNSIGNED,\
            FOREIGN KEY (section_id) REFERENCES sections(id),\
            FOREIGN KEY (subsection_id) REFERENCES subsections(id)\
        );\
        ";
    let _ = sqlx::query(query_str).execute(pool).await;
}

pub async fn create_required_tables(pool: &sqlx::Pool<sqlx::MySql>) {
    create_users_table(pool).await;
    create_sections_table(pool).await;
    create_subsections_table(pool).await;
    create_notes_table(pool).await;
}
pub async fn drop_all_tables(pool: &sqlx::Pool<sqlx::MySql>) {
    let query_strs = [
        "DROP TABLE notes;",
        "DROP TABLE subsections;",
        "DROP TABLE sections;",
        "DROP TABLE users;",
    ];
    let table_names = ["notes", "subsections", "sections", "users"];
    for table_name in table_names {
        let query_str = format!("DROP TABLE IF EXISTS {} ;", table_name);
        let _ = sqlx::query(query_str.as_str()).execute(pool).await;
    }
}
