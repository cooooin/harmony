pub mod object;
pub mod trade;

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, PartialOrd, Serialize, Deserialize)]
pub struct Quantity(Decimal);

impl Quantity {
    pub fn to_string(&self) -> String {
        self.0.to_string()
    }
}

mod database {
    use std::str::FromStr;

    use rusqlite::types::{FromSql, FromSqlResult, ToSql, ToSqlOutput, ValueRef};

    use rust_decimal::prelude::FromPrimitive;

    use super::Quantity;

    impl FromSql for Quantity {
        fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
            if let Ok(text) = value.as_str() {
                if let Ok(decimal) = rust_decimal::Decimal::from_str(text) {
                    return Ok(Quantity(decimal));
                }
            }

            if let Ok(float) = value.as_f64() {
                if let Some(decimal) = rust_decimal::Decimal::from_f64(float) {
                    return Ok(Quantity(decimal));
                }
            }

            Err(rusqlite::types::FromSqlError::InvalidType)
        }
    }

    impl ToSql for Quantity {
        fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
            Ok(ToSqlOutput::from(self.to_string()))
        }
    }
}
