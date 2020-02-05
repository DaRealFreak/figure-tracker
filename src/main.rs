use rusqlite::Error;

mod database;

fn main() -> Result<(), Error> {
    let test = database::test::Database::open("cats.db")?;
    test.create_migrations()?;

    Ok(())
}