pub mod transaction;

use chrono::{DateTime, Utc};

pub struct Trade {
    id: i64,
    pub owner: i64,
    pub base_object_id: i64,
    pub quote_object_id: i64,
    pub alias: Option<String>,
    pub remark: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Trade {
    pub fn id(&self) -> i64 {
        self.id
    }
}

mod database {
    use rusqlite::params;
    use rusqlite::Connection;
    use rusqlite::OptionalExtension;

    use crate::model::database::Result;

    impl crate::model::Model for super::Trade {
        fn initialize() -> &'static str {
            "
                CREATE TABLE IF NOT EXISTS finance_trade (
                    id               INTEGER NOT NULL UNIQUE PRIMARY KEY AUTOINCREMENT,
                    owner            INTEGER NOT NULL,
                    base_object_id   INTEGER NOT NULL,
                    quote_object_id  INTEGER NOT NULL,
                    alias            TEXT,
                    remark           TEXT,
                    created_at       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at       DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(owner) REFERENCES person(id) ON DELETE CASCADE,
                    FOREIGN KEY(base_object_id) REFERENCES finance_object(id) ON DELETE CASCADE,
                    FOREIGN KEY(quote_object_id) REFERENCES finance_object(id) ON DELETE CASCADE
                );

                CREATE TRIGGER IF NOT EXISTS update_finance_trade_updated_at
                AFTER UPDATE ON finance_trade
                FOR EACH ROW
                BEGIN
                    UPDATE finance_trade SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
                END;

                CREATE INDEX IF NOT EXISTS idx_finance_trade_owner ON finance_trade(owner);
            "
        }
    }

    impl super::Trade {
        pub fn insert(
            conn: &Connection,
            owner: i64,
            base_object_id: i64,
            quote_object_id: i64,
            alias: Option<String>,
            remark: Option<String>,
        ) -> Result<i64> {
            let sql = r#"
                INSERT INTO finance_trade (owner, base_object_id, quote_object_id, alias, remark)
                VALUES (?1, ?2, ?3, ?4, ?5)
                RETURNING id;
            "#;

            let id = conn.query_row(
                sql,
                params![owner, base_object_id, quote_object_id, alias, remark],
                |row| row.get(0),
            )?;

            Ok(id)
        }

        pub fn count_by_owner(conn: &Connection, owner: i64) -> Result<usize> {
            let sql = r#"
                SELECT COUNT(*)
                FROM finance_trade
                WHERE owner = ?1;
            "#;

            let count = conn.query_row(sql, params![owner], |row| row.get(0))?;

            Ok(count)
        }

        pub fn select_by_id_owner(conn: &Connection, id: i64, owner: i64) -> Result<Option<Self>> {
            let sql = r#"
                SELECT id, owner, base_object_id, quote_object_id, alias, remark, created_at, updated_at
                FROM finance_trade
                WHERE id = ?1 AND owner = ?2;
            "#;

            let result = conn
                .query_row(sql, params![id, owner], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        owner: row.get(1)?,
                        base_object_id: row.get(2)?,
                        quote_object_id: row.get(3)?,
                        alias: row.get(4)?,
                        remark: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
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
                SELECT id, owner, base_object_id, quote_object_id, alias, remark, created_at, updated_at
                FROM finance_trade
                WHERE owner = ?1
                LIMIT ?2 OFFSET ?3;
            "#;

            let mut stmt = conn.prepare(sql)?;
            let trades = stmt
                .query_map(params![owner, limit, offset], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        owner: row.get(1)?,
                        base_object_id: row.get(2)?,
                        quote_object_id: row.get(3)?,
                        alias: row.get(4)?,
                        remark: row.get(5)?,
                        created_at: row.get(6)?,
                        updated_at: row.get(7)?,
                    })
                })?
                .collect::<Result<Vec<Self>>>()?;

            Ok(trades)
        }

        pub fn update_by_id_owner(
            conn: &Connection,
            id: i64,
            owner: i64,
            base_object_id: i64,
            quote_object_id: i64,
            alias: Option<String>,
            remark: Option<String>,
        ) -> Result<()> {
            let sql = r#"
                UPDATE finance_trade
                SET base_object_id = ?1, quote_object_id = ?2, alias = ?3, remark = ?4
                WHERE id = ?5 AND owner = ?6;
            "#;

            conn.execute(
                sql,
                params![base_object_id, quote_object_id, alias, remark, id, owner],
            )?;

            Ok(())
        }

        pub fn delete_by_id_owner(conn: &Connection, id: i64, owner: i64) -> Result<()> {
            let sql = r#"
                DELETE FROM finance_trade
                WHERE id = ?1 AND owner = ?2;
            "#;

            conn.execute(sql, params![id, owner])?;

            Ok(())
        }
    }
}
