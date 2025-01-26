use chrono::{DateTime, Utc};

use crate::model::finance::Quantity;

pub struct Transaction {
    id: i64,
    pub trade_id: i64,
    pub quantity: Quantity,
    pub is_base_to_quote: bool,
    pub alias: Option<String>,
    pub remark: Option<String>,
    pub occurrence_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Transaction {
    pub fn id(&self) -> i64 {
        self.id
    }
}

mod database {
    use chrono::DateTime;
    use chrono::Utc;
    use rusqlite::params;
    use rusqlite::Connection;
    use rusqlite::OptionalExtension;

    use crate::model::database::Result;
    use crate::model::finance::Quantity;

    impl crate::model::Model for super::Transaction {
        fn initialize() -> &'static str {
            "
                CREATE TABLE IF NOT EXISTS finance_trade_transaction (
                    id                INTEGER  NOT NULL UNIQUE PRIMARY KEY AUTOINCREMENT,
                    trade_id          INTEGER  NOT NULL,
                    quantity          TEXT     NOT NULL,
                    is_base_to_quote  BOOL     NOT NULL,
                    alias             TEXT,
                    remark            TEXT,
                    occurrence_at     DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    created_at        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    updated_at        DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
                    FOREIGN KEY(trade_id) REFERENCES finance_trade(id) ON DELETE CASCADE
                );

                CREATE TRIGGER IF NOT EXISTS update_finance_trade_transaction_updated_at
                AFTER UPDATE ON finance_trade_transaction
                FOR EACH ROW
                BEGIN
                    UPDATE finance_trade_transaction SET updated_at = CURRENT_TIMESTAMP WHERE id = OLD.id;
                END;

                CREATE INDEX IF NOT EXISTS idx_finance_trade_transaction_trade_id ON finance_trade_transaction(trade_id);
            "
        }
    }

    impl super::Transaction {
        pub fn insert(
            conn: &Connection,
            trade_id: i64,
            quantity: Quantity,
            is_base_to_quote: bool,
            alias: Option<String>,
            remark: Option<String>,
            occurrence_at: Option<DateTime<Utc>>,
        ) -> Result<i64> {
            let sql = r#"
                INSERT INTO finance_trade_transaction (trade_id, quantity, is_base_to_quote, alias, remark, occurrence_at)
                VALUES (?1, ?2, ?3, ?4, ?5, ?6)
                RETURNING id;
            "#;

            let occurrence_at = occurrence_at.unwrap_or(Utc::now());

            let id = conn.query_row(
                sql,
                params![
                    trade_id,
                    quantity.to_string(),
                    is_base_to_quote,
                    alias,
                    remark,
                    occurrence_at
                ],
                |row| row.get(0),
            )?;

            Ok(id)
        }

        pub fn count_by_trade_id(conn: &Connection, trade_id: i64) -> Result<usize> {
            let sql = r#"
                SELECT COUNT(*)
                FROM finance_trade_transaction
                WHERE trade_id = ?1;
            "#;

            let count = conn.query_row(sql, params![trade_id], |row| row.get(0))?;

            Ok(count)
        }

        pub fn select_by_id_trade_id(
            conn: &Connection,
            id: i64,
            trade_id: i64,
        ) -> Result<Option<Self>> {
            let sql = r#"
                SELECT id, trade_id, quantity, is_base_to_quote, alias, remark, occurrence_at, created_at, updated_at
                FROM finance_trade_transaction
                WHERE id = ?1 AND trade_id = ?2;
            "#;

            let result = conn
                .query_row(sql, params![id, trade_id], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        trade_id: row.get(1)?,
                        quantity: row.get(2)?,
                        is_base_to_quote: row.get(3)?,
                        alias: row.get(4)?,
                        remark: row.get(5)?,
                        occurrence_at: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                })
                .optional();

            result
        }

        pub fn select_by_trade_id(
            conn: &Connection,
            trade_id: i64,
            limit: usize,
            offset: usize,
        ) -> Result<Vec<Self>> {
            let sql = r#"
                SELECT id, trade_id, quantity, is_base_to_quote, alias, remark, occurrence_at, created_at, updated_at
                FROM finance_trade_transaction
                WHERE trade_id = ?1
                LIMIT ?2 OFFSET ?3;
            "#;

            let mut stmt = conn.prepare(sql)?;
            let transactions = stmt
                .query_map(params![trade_id, limit, offset], |row| {
                    Ok(Self {
                        id: row.get(0)?,
                        trade_id: row.get(1)?,
                        quantity: row.get(2)?,
                        is_base_to_quote: row.get(3)?,
                        alias: row.get(4)?,
                        remark: row.get(5)?,
                        occurrence_at: row.get(6)?,
                        created_at: row.get(7)?,
                        updated_at: row.get(8)?,
                    })
                })?
                .collect::<Result<Vec<Self>>>()?;

            Ok(transactions)
        }

        pub fn update_by_id_trade_id(
            conn: &Connection,
            id: i64,
            trade_id: i64,
            quantity: Quantity,
            is_base_to_quote: bool,
            occurrence_at: DateTime<Utc>,
            alias: Option<String>,
            remark: Option<String>,
        ) -> Result<()> {
            let sql = r#"
                UPDATE finance_trade_transaction
                SET quantity = ?1, is_base_to_quote = ?2, alias = ?3, remark = ?4, occurrence_at = ?5
                WHERE id = ?6 AND trade_id = ?7;
            "#;

            conn.execute(
                sql,
                params![
                    quantity.to_string(),
                    is_base_to_quote,
                    alias,
                    remark,
                    occurrence_at,
                    id,
                    trade_id
                ],
            )?;

            Ok(())
        }

        pub fn delete_by_id_trade_id(conn: &Connection, id: i64, trade_id: i64) -> Result<()> {
            let sql = r#"
                DELETE FROM finance_trade_transaction
                WHERE id = ?1 AND trade_id = ?2;
            "#;

            conn.execute(sql, params![id, trade_id])?;

            Ok(())
        }
    }
}
