use rusqlite::Error;

use crate::database::migrations::Migration;

mod database;

fn main() -> Result<(), Error> {
    let con = database::Database::open("figure_tracker.db")?;
    con.create_migrations()?;

    Ok(())
}