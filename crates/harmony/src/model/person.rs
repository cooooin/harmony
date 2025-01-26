use chrono::{DateTime, Utc};

pub struct Person {
    id: i64,
    pub nickname: String,
    pub password: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Person {
    pub fn id(&self) -> i64 {
        self.id
    }
}

mod database {
    use rusqlite::Connection;
    use rusqlite::OptionalExtension;

    pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

    impl crate::model::Model for super::Person {
        fn initialize() -> &'static str {
            "
                CREATE TABLE IF NOT EXISTS person (
                    id           INTEGER   NOT NULL  PRIMARY KEY AUTOINCREMENT,
                    nickname     TEXT      NOT NULL  UNIQUE,
                    password     TEXT      NOT NULL,
                    created_at   DATETIME  NOT NULL  DEFAULT CURRENT_TIMESTAMP,
                    updated_at   DATETIME  NOT NULL  DEFAULT CURRENT_TIMESTAMP
                );

                CREATE TRIGGER IF NOT EXISTS update_person_updated_at
                AFTER UPDATE ON person
                FOR EACH ROW
                BEGIN
                    UPDATE person
                    SET updated_at = CURRENT_TIMESTAMP
                    WHERE id = OLD.id;
                END;
            "
        }
    }

    impl super::Person {
        pub fn insert_one(conn: &Connection, nickname: &String, password: &String) -> Result<Self> {
            let sql = "INSERT INTO person (nickname, password) VALUES (?1, ?2) RETURNING id, nickname, password, created_at, updated_at";
            let mut statement = conn.prepare(sql)?;

            let item = statement.query_row([nickname, password], |row| {
                Ok(Self {
                    id: row.get(0)?,
                    nickname: row.get(1)?,
                    password: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                })
            })?;

            Ok(item)
        }

        pub fn select_one_by_id(conn: &Connection, id: i64) -> Result<Option<Self>> {
            let sql = "SELECT id, nickname, password, created_at, updated_at FROM person WHERE id = ?1 LIMIT 1";
            let mut statement = conn.prepare(sql)?;

            let item = statement
                .query_row([id], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        nickname: row.get(1)?,
                        password: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })
                .optional()?;

            Ok(item)
        }

        pub fn select_one_by_nickname(
            conn: &Connection,
            nickname: &String,
        ) -> Result<Option<Self>> {
            let sql = "SELECT id, nickname, password, created_at, updated_at FROM person WHERE nickname = ?1 LIMIT 1";
            let mut statement = conn.prepare(sql)?;

            let item = statement
                .query_row([nickname], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        nickname: row.get(1)?,
                        password: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })
                .optional()?;

            Ok(item)
        }

        pub fn select_one_by_nickname_password(
            conn: &Connection,
            nickname: &String,
            password: &String,
        ) -> Result<Option<Self>> {
            let sql = "SELECT id, nickname, password, created_at, updated_at FROM person WHERE nickname = ?1 AND password = ?2 LIMIT 1";
            let mut statement = conn.prepare(sql)?;

            let item = statement
                .query_row([nickname, password], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        nickname: row.get(1)?,
                        password: row.get(2)?,
                        created_at: row.get(3)?,
                        updated_at: row.get(4)?,
                    })
                })
                .optional()?;

            Ok(item)
        }

        pub fn update_one_by_id(conn: &Connection, id: i64, item: &Self) -> Result<()> {
            let sql = "UPDATE person SET nickname = ?2, password = ?3 WHERE id = ?1";
            let mut statement = conn.prepare(sql)?;

            statement.execute([&id.to_string(), &item.nickname, &item.password])?;

            Ok(())
        }
    }

    #[cfg(test)]
    mod tests {
        use r2d2::PooledConnection;
        use r2d2_sqlite::SqliteConnectionManager;

        use crate::model::{person::Person, Model};

        fn setup_database() -> PooledConnection<SqliteConnectionManager> {
            let database = SqliteConnectionManager::memory();
            let pool = r2d2::Pool::new(database).unwrap();

            let conn = pool.get().unwrap();

            conn.execute_batch(Person::initialize()).unwrap();

            conn
        }

        #[test]
        fn test_insert_one() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");

            let person = Person::insert_one(&conn, &nickname, &password).unwrap();

            assert_eq!(person.nickname, nickname);
            assert_eq!(person.password, password);
            assert!(person.id() > 0);
        }

        #[test]
        fn test_select_one_by_id() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");

            let inserted_person = Person::insert_one(&conn, &nickname, &password).unwrap();
            let selected_person = Person::select_one_by_id(&conn, inserted_person.id())
                .unwrap()
                .unwrap();

            assert_eq!(inserted_person.id(), selected_person.id());
            assert_eq!(inserted_person.nickname, selected_person.nickname);
            assert_eq!(inserted_person.password, selected_person.password);
        }

        #[test]
        fn test_select_one_by_nickname() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");

            let inserted_person = Person::insert_one(&conn, &nickname, &password).unwrap();
            let selected_person = Person::select_one_by_nickname(&conn, &nickname)
                .unwrap()
                .unwrap();

            assert_eq!(inserted_person.id(), selected_person.id());
            assert_eq!(inserted_person.nickname, selected_person.nickname);
            assert_eq!(inserted_person.password, selected_person.password);
        }

        #[test]
        fn test_select_one_by_nickname_password() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");

            let inserted_person = Person::insert_one(&conn, &nickname, &password).unwrap();
            let selected_person =
                Person::select_one_by_nickname_password(&conn, &nickname, &password)
                    .unwrap()
                    .unwrap();

            assert_eq!(inserted_person.id(), selected_person.id());
            assert_eq!(inserted_person.nickname, selected_person.nickname);
            assert_eq!(inserted_person.password, selected_person.password);
        }

        #[test]
        fn test_update_one_by_id() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");

            let mut inserted_person = Person::insert_one(&conn, &nickname, &password).unwrap();
            let new_nickname = String::from("new_test_user");
            let new_password = String::from("new_test_password");

            inserted_person.nickname = new_nickname.clone();
            inserted_person.password = new_password.clone();

            Person::update_one_by_id(&conn, inserted_person.id(), &inserted_person).unwrap();

            let updated_person = Person::select_one_by_id(&conn, inserted_person.id())
                .unwrap()
                .unwrap();

            assert_eq!(updated_person.nickname, new_nickname);
            assert_eq!(updated_person.password, new_password);
        }

        #[test]
        fn test_select_one_by_id_not_found() {
            let conn = setup_database();
            let non_existent_id = 999;

            let result = Person::select_one_by_id(&conn, non_existent_id).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn test_select_one_by_nickname_not_found() {
            let conn = setup_database();
            let non_existent_nickname = String::from("non_existent_user");

            let result = Person::select_one_by_nickname(&conn, &non_existent_nickname).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn test_select_one_by_nickname_password_not_found() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");
            let wrong_password = String::from("wrong_password");

            Person::insert_one(&conn, &nickname, &password).unwrap();

            let result =
                Person::select_one_by_nickname_password(&conn, &nickname, &wrong_password).unwrap();

            assert!(result.is_none());
        }

        #[test]
        fn test_select_one_by_nickname_password_mismatch() {
            let conn = setup_database();
            let nickname = String::from("test_user");
            let password = String::from("test_password");
            let wrong_password = String::from("wrong_password");

            Person::insert_one(&conn, &nickname, &password).unwrap();

            let result =
                Person::select_one_by_nickname_password(&conn, &nickname, &wrong_password).unwrap();
            assert!(result.is_none());

            let wrong_nickname = String::from("wrong_user");
            let result =
                Person::select_one_by_nickname_password(&conn, &wrong_nickname, &password).unwrap();
            assert!(result.is_none());
        }
    }
}
