use rusqlite::Error;

mod database;

fn main() -> Result<(), Error> {
    let con = database::Database::open("figure_tracker.db")?;

    Ok(())
}