use chrono::{DateTime, Utc};

pub struct Object {
    id: i64,
    pub owner: i64,
    pub symbol: String,
    pub alias: Option<String>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Object {
    pub fn id(&self) -> i64 {
        self.id
    }
}

mod database {
    use rusqlite::params;
    use rusqlite::Connection;
    use rusqlite::OptionalExtension;

    use crate::model::database::Result;

    impl crate::model::Model for super::Object {
        fn initialize() -> &'static str {
            "
                CREATE TABLE IF NOT EXISTS finance_object (
                    id         INTEGER  NOT NULL  UNIQUE PRIMARY KEY AUTOINCREMENT,
                    owner      INTEGER  NOT NULL,
                    symbol     TEXT     NOT NULL,
                    alias      TEXT,
                    remark     TEXT,
                    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(owner) REFERENCES person(id) ON DELETE CASCADE
                );

                CREATE TRIGGER IF NOT EXISTS update_finance_object_updated_at
                AFTER UPDATE ON finance_object
                FOR EACH ROW
                BEGIN
                    UPDATE finance_object SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
                END;

                CREATE INDEX IF NOT EXISTS idx_finance_object_owner ON finance_object(owner);
            "
        }
    }

    impl super::Object {
        pub fn insert(
            conn: &Connection,
            owner: i64,
            symbol: String,
            alias: Option<String>,
            remark: Option<String>,
        ) -> Result<i64> {
            let sql = r#"
                INSERT INTO finance_object (owner, symbol, alias, remark)
                VALUES (?1, ?2, ?3, ?4)
                RETURNING id;
            "#;

            let id =
                conn.query_row(sql, params![owner, symbol, alias, remark], |row| row.get(0))?;

            Ok(id)
        }

        pub fn count_by_owner(conn: &Connection, owner: i64) -> Result<usize> {
            let sql = r#"
                SELECT COUNT(*)
                FROM finance_object
                WHERE owner = ?1;
            "#;

            let count = conn.query_row(sql, params![owner], |row| row.get(0))?;

            Ok(count)
        }

        pub fn select_by_id_owner(conn: &Connection, id: i64, owner: i64) -> Result<Option<Self>> {
            let sql = r#"
                SELECT id, owner, symbol, alias, remark, created_at, updated_at
                FROM finance_object
                WHERE id = ?1 AND owner = ?2;
            "#;

            let result = conn
                .query_row(sql, params![id, owner], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        owner: row.get(1)?,
                        symbol: row.get(2)?,
                        alias: row.get(3)?,
                        remark: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })
                .optional();

            result
        }

        pub fn select_by_owner(
            conn: &Connection,
            owner: i64,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<Self>> {
            let sql = r#"
                SELECT id, owner, symbol, alias, remark, created_at, updated_at
                FROM finance_object
                WHERE owner = ?1
                LIMIT ?2 OFFSET ?3;
            "#;

            let mut stmt = conn.prepare(sql)?;
            let objects = stmt
                .query_map(params![owner, limit, offset], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        owner: row.get(1)?,
                        symbol: row.get(2)?,
                        alias: row.get(3)?,
                        remark: row.get(4)?,
                        created_at: row.get(5)?,
                        updated_at: row.get(6)?,
                    })
                })?
                .collect::<Result<Vec<Self>>>()?;

            Ok(objects)
        }

        pub fn update_by_id_owner(
            conn: &Connection,
            id: i64,
            owner: i64,
            symbol: String,
            alias: Option<String>,
            remark: Option<String>,
        ) -> Result<()> {
            let sql = r#"
                UPDATE finance_object
                SET symbol = ?1, alias = ?2, remark = ?3
                WHERE id = ?4 AND owner = ?5;
            "#;

            conn.execute(sql, params![symbol, alias, remark, id, owner])?;

            Ok(())
        }

        pub fn delete_by_id_owner(conn: &Connection, id: i64, owner: i64) -> Result<()> {
            let sql = r#"
                DELETE FROM finance_object
                WHERE id = ?1 AND owner = ?2;
            "#;

            conn.execute(sql, params![id, owner])?;

            Ok(())
        }
    }
}

#[cfg(test)]
mod tests {
    use r2d2::PooledConnection;
    use r2d2_sqlite::SqliteConnectionManager;

    use crate::model::person::Person;
    use crate::model::Model;

    use super::Object;

    // Helper function to set up the database and create a test user
    fn setup() -> (PooledConnection<SqliteConnectionManager>, i64) {
        let database = SqliteConnectionManager::memory();
        let pool = r2d2::Pool::new(database).unwrap();

        let conn = pool.get().unwrap();

        // Initialize the database schema for Person and Object
        conn.execute_batch(Person::initialize()).unwrap();
        conn.execute_batch(Object::initialize()).unwrap();

        // Insert a test user into the database
        let nickname = "test_user".to_string();
        let password = "test_password".to_string();
        let person = Person::insert_one(&conn, &nickname, &password).unwrap();

        (conn, person.id())
    }

    #[test]
    fn test_insert() {
        let (conn, owner_id) = setup();
        let symbol = "AAPL".to_string();

        // Insert a new object into the database
        let id = Object::insert(&conn, owner_id, symbol.clone(), None, None).unwrap();
        assert_eq!(id, 1);

        // Retrieve the inserted object and verify its fields
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.id, id);
        assert_eq!(obj.owner, owner_id);
        assert_eq!(obj.symbol, symbol);
        assert_eq!(obj.alias, None);
        assert_eq!(obj.remark, None);
    }

    #[test]
    fn test_count_by_owner() {
        let (conn, owner_id) = setup();

        // Insert two objects for the same owner
        Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();
        Object::insert(&conn, owner_id, "GOOG".to_string(), None, None).unwrap();

        // Verify that the count of objects for the owner is correct
        assert_eq!(Object::count_by_owner(&conn, owner_id).unwrap(), 2);
    }

    #[test]
    fn test_select_by_id_owner() {
        let (conn, owner_id) = setup();

        // Insert an object and retrieve it by ID and owner
        let id = Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();

        // Correct query: Retrieve the object with the correct ID and owner
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.symbol, "AAPL");

        // Incorrect query: Attempt to retrieve the object with a different owner
        assert!(Object::select_by_id_owner(&conn, id, owner_id + 1)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_select_by_owner() {
        let (conn, owner_id) = setup();

        // Insert two objects for the same owner
        Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();
        Object::insert(&conn, owner_id, "GOOG".to_string(), None, None).unwrap();

        // Retrieve all objects for the owner with a limit of 10 and offset of 0
        let objs = Object::select_by_owner(&conn, owner_id, 10, 0).unwrap();
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].symbol, "AAPL");
        assert_eq!(objs[1].symbol, "GOOG");

        // Retrieve objects with a limit of 1 and offset of 1 (pagination)
        let objs = Object::select_by_owner(&conn, owner_id, 1, 1).unwrap();
        assert_eq!(objs.len(), 1);
        assert_eq!(objs[0].symbol, "GOOG");
    }

    #[test]
    fn test_update_by_id_owner() {
        let (conn, owner_id) = setup();
        let id = Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();

        // Update the object's fields
        Object::update_by_id_owner(
            &conn,
            id,
            owner_id,
            "GOOG".to_string(),
            Some("Google".to_string()),
            Some("Test".to_string()),
        )
        .unwrap();

        // Retrieve the updated object and verify the changes
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.symbol, "GOOG");
        assert_eq!(obj.alias, Some("Google".to_string()));
        assert_eq!(obj.remark, Some("Test".to_string()));
    }

    #[test]
    fn test_delete_by_id_owner() {
        let (conn, owner_id) = setup();
        let id = Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();

        // Delete the object by ID and owner
        Object::delete_by_id_owner(&conn, id, owner_id).unwrap();

        // Verify that the object no longer exists in the database
        assert!(Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .is_none());
    }

    #[test]
    fn test_insert_with_optional_fields() {
        let (conn, owner_id) = setup();
        let symbol = "AAPL".to_string();

        // Insert with alias and remark as None
        let id = Object::insert(&conn, owner_id, symbol.clone(), None, None).unwrap();
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.alias, None);
        assert_eq!(obj.remark, None);

        // Insert with alias and remark as Some
        let id = Object::insert(
            &conn,
            owner_id,
            "GOOG".to_string(),
            Some("Google".to_string()),
            Some("Test".to_string()),
        )
        .unwrap();
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.alias, Some("Google".to_string()));
        assert_eq!(obj.remark, Some("Test".to_string()));
    }

    #[test]
    fn test_update_optional_fields() {
        let (conn, owner_id) = setup();
        let id = Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();

        // Update alias and remark from None to Some
        Object::update_by_id_owner(
            &conn,
            id,
            owner_id,
            "AAPL".to_string(),
            Some("Apple".to_string()),
            Some("Updated".to_string()),
        )
        .unwrap();
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.alias, Some("Apple".to_string()));
        assert_eq!(obj.remark, Some("Updated".to_string()));

        // Update alias and remark from Some to None
        Object::update_by_id_owner(&conn, id, owner_id, "AAPL".to_string(), None, None).unwrap();
        let obj = Object::select_by_id_owner(&conn, id, owner_id)
            .unwrap()
            .unwrap();
        assert_eq!(obj.alias, None);
        assert_eq!(obj.remark, None);
    }

    #[test]
    fn test_foreign_key_constraint() {
        let (conn, owner_id) = setup();

        // Attempt to insert an object with an invalid owner
        let result = Object::insert(&conn, owner_id + 1, "AAPL".to_string(), None, None);
        assert!(result.is_err()); // Should fail due to foreign key constraint
    }

    #[test]
    fn test_select_by_owner_pagination() {
        let (conn, owner_id) = setup();

        // Insert multiple objects
        for i in 0..5 {
            Object::insert(&conn, owner_id, format!("SYM{}", i), None, None).unwrap();
        }

        // Test limit and offset
        let objs = Object::select_by_owner(&conn, owner_id, 2, 0).unwrap();
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].symbol, "SYM0");
        assert_eq!(objs[1].symbol, "SYM1");

        let objs = Object::select_by_owner(&conn, owner_id, 2, 2).unwrap();
        assert_eq!(objs.len(), 2);
        assert_eq!(objs[0].symbol, "SYM2");
        assert_eq!(objs[1].symbol, "SYM3");

        // Test offset beyond the total count
        let objs = Object::select_by_owner(&conn, owner_id, 2, 10).unwrap();
        assert_eq!(objs.len(), 0);
    }

    #[test]
    fn test_delete_non_existent_object() {
        let (conn, owner_id) = setup();

        Object::delete_by_id_owner(&conn, 999, owner_id).unwrap();

        assert_eq!(Object::count_by_owner(&conn, owner_id).unwrap(), 0);
    }

    #[test]
    fn test_count_by_owner_edge_cases() {
        let (conn, owner_id) = setup();

        // Verify count is 0 when no objects exist
        assert_eq!(Object::count_by_owner(&conn, owner_id).unwrap(), 0);

        // Insert an object and verify count is 1
        Object::insert(&conn, owner_id, "AAPL".to_string(), None, None).unwrap();
        assert_eq!(Object::count_by_owner(&conn, owner_id).unwrap(), 1);

        // Delete the object and verify count is 0
        Object::delete_by_id_owner(&conn, 1, owner_id).unwrap();
        assert_eq!(Object::count_by_owner(&conn, owner_id).unwrap(), 0);
    }
}
