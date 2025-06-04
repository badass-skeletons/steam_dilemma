use rusqlite::{Connection, Result};

fn table_exists(conn: &Connection, table_name: &str) -> Result<bool> {
    let mut stmt =
        conn.prepare("SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1")?;
    let count: i64 = stmt.query_row([table_name], |row| row.get(0))?;
    Ok(count > 0)
}

pub fn main() {
    library::test_lib();
}
