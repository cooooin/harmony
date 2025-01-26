pub mod finance;
pub mod person;

pub mod database {
    use rusqlite::Error;

    use r2d2::PooledConnection;
    use r2d2_sqlite::SqliteConnectionManager;

    pub mod prelude {
        pub use super::connection;
    }

    pub type Result<T> = std::result::Result<T, Error>;

    pub fn connection() -> Result<PooledConnection<SqliteConnectionManager>> {
        use crate::consts::database::DATABASE;

        Ok(DATABASE.get().or(Err(Error::ExecuteReturnedResults))?)
    }
}

trait Model {
    fn initialize() -> &'static str;
}

pub(crate) fn initialize() -> String {
    [
        person::Person::initialize(),
        finance::object::Object::initialize(),
        finance::trade::Trade::initialize(),
        finance::trade::transaction::Transaction::initialize(),
    ]
    .concat()
}
