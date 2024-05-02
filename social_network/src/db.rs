use rusqlite::{params, Connection, Result};
use bcrypt::{hash, verify};
use serde_derive::{Deserialize, Serialize};

const USERS_DB: &str = "users.db";
const WORDS_DB: &str = "words.db";

#[derive(Deserialize, Serialize, Debug)]
pub struct Word {
    pub word: String,
    pub language: String,
}

pub mod user_db {
    use super::*;

    pub fn create_table() -> Result<()> {
        let conn = Connection::open(USERS_DB)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS users (
                id INTEGER PRIMARY KEY,
                username TEXT NOT NULL UNIQUE,
                password_hash TEXT NOT NULL
            )",
            [],
        )?;
        println!("Table 'users' created successfully.");
        Ok(())
    }

    pub fn find_user_by_username(username: &str) -> Result<Option<String>> {
        let conn = Connection::open(USERS_DB)?;

        let mut stmt = conn.prepare("SELECT username FROM users WHERE username = ?")?;
        let mut rows = stmt.query(params![username])?;

        if let Some(row) = rows.next()? {
            Ok(Some(row.get(0)?))
        } else {
            Ok(None)
        }
    }

    pub fn register_user(username: &str, password: &str) -> Result<()> {
        let hashed_password = hash(password, bcrypt::DEFAULT_COST).unwrap();

        let conn = Connection::open(USERS_DB)?;

        let mut stmt = conn.prepare("SELECT COUNT(*) FROM users WHERE username = ?")?;
        let user_count: i64 = stmt.query_row(&[username], |row| row.get(0))?;

        if user_count > 0 {
            return Err(rusqlite::Error::QueryReturnedNoRows);
        }

        let mut stmt = conn.prepare("INSERT INTO users (username, password_hash) VALUES (?, ?)")?;

        stmt.execute(&[username, &hashed_password])?;

        Ok(())
    }

    pub fn check_credentials(username: &str, password: &str) -> Result<bool> {
        let conn = Connection::open(USERS_DB)?;

        let mut stmt = conn.prepare("SELECT password_hash FROM users WHERE username = ?")?;
        let mut rows = stmt.query(&[username])?;

        if let Some(row) = rows.next()? {
            let hashed_password: String = row.get(0)?;

            let is_valid_password = verify(password, &hashed_password).unwrap();

            return Ok(is_valid_password);
        }

        Ok(false)
    }
}

pub mod word_db {
    use super::*;

    pub fn create_table() -> Result<()> {
        let conn = Connection::open(WORDS_DB)?;

        conn.execute(
            "CREATE TABLE IF NOT EXISTS words (
                id INTEGER PRIMARY KEY,
                word TEXT NOT NULL UNIQUE,
                language TEXT NOT NULL
            )",
            [],
        )?;
        println!("Table 'words' created successfully.");
        Ok(())
    }

    pub fn add_word(word: &str, language: &str) -> Result<()> {
        if word_exists(word, language) {
            return Ok(());
        }

        let conn = Connection::open(WORDS_DB)?;

        let mut stmt = conn.prepare("INSERT INTO words (word, language) VALUES (?, ?)")?;

        stmt.execute(&[word, language])?;

        Ok(())
    }

    pub fn word_exists(word: &str, language: &str) -> bool {
        let conn = Connection::open(WORDS_DB).expect("Failed to open database");

        let sql = "SELECT EXISTS(SELECT 1 FROM words WHERE word = ? AND language = ? LIMIT 1)";
        let mut stmt = conn.prepare(sql).expect("Failed to prepare statement");

        let exists: Result<bool> = stmt.query_row(params![word, language], |row| row.get(0));
        match exists {
            Ok(result) => result,
            Err(_) => false,
        }
    }

    pub fn fetch_words_from_db() -> Result<Vec<Word>> {
        let conn = Connection::open(WORDS_DB)?;

        let sql = "SELECT word, language FROM words";
        let mut stmt = conn.prepare(sql)?;

        let word_iter = stmt.query_map([], |row| {
            Ok(Word {
                word: row.get(0)?,
                language: row.get(1)?,
            })
        })?;

        let mut words = Vec::new();
        for word_result in word_iter {
            words.push(word_result?);
        }

        Ok(words)
    }
}
