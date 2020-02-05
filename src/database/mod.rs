pub mod test {
    extern crate rusqlite;

    use rusqlite::NO_PARAMS;

    use self::rusqlite::{Connection, Error};

    pub(crate) struct Database {
        conn: Connection
    }

    impl Database {
        pub fn open(path: &str) -> Result<Database, Error> {
            let conn = Connection::open(path)?;
            Ok(Database {
                conn,
            })
        }

        pub fn create_migrations(&self) -> Result<(), Error> {
            self.conn.execute(
                "create table if not exists cat_colors (
                         id integer primary key,
                         name text not null unique
                     )",
                NO_PARAMS,
            )?;
            self.conn.execute(
                "create table if not exists cats (
                         id integer primary key,
                         name text not null,
                         color_id integer not null references cat_colors(id)
                     )",
                NO_PARAMS,
            )?;

            Ok(())
        }
    }
}
