use rusqlite::{Connection, OptionalExtension, Result, params};

use crate::types::{Phase, UserInfo};

const PATH: &str = "databases/database.db";

pub async fn get_userinfo_by_id(user_id: u64) -> Result<UserInfo> {
    const GET_USER: &str = "
    SELECT * FROM users WHERE discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let res = conn.query_row(GET_USER, params![user_id as i64], |row| {
        Ok(UserInfo {
            discord_id: row.get::<_, i64>(0)? as u64,
            username: row.get(1)?,
            letter: row.get(2)?,
            submission: row.get(3)?,
            giftee_id: row.get::<_, Option<i64>>(4)?.map(|id| id as u64),
            is_banned: row.get(5)?,
            has_joined: row.get(6)?,
        })
    })?;
    Ok(res)
}

pub async fn create_user(username: &str, user_id: u64) -> Result<()> {
    const ADD_USER: &str = "
    INSERT INTO users (discord_id, username, is_banned, has_joined)
    VALUES (?1, ?2, ?3, ?4);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(ADD_USER, params![user_id as i64, username, false, true])
        .map_err(|e| {
            eprintln!("Problem adding user to database: {}", e);
            e
        })?;
    Ok(())
}

pub async fn leave(user_id: u64) -> Result<()> {
    const DELETE_USER: &str = "
    UPDATE users
    SET has_joined = 0
    WHERE discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(DELETE_USER, params![user_id as i64])
        .expect("Error deleting user");
    Ok(())
}

pub async fn set_letter(user_id: u64, letter_content: &str) -> Result<usize> {
    const UPDATE_LETTER: &str = "
        UPDATE users
        SET letter = ?1
        WHERE discord_id = ?2;
    ";

    let mut conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let tx = conn.transaction()?;
    let updated = tx.execute(UPDATE_LETTER, params![letter_content, user_id as i64])?;
    tx.commit()?;

    Ok(updated)
}

pub async fn get_giftee(santa_id: u64) -> Result<u64> {
    const GET_GIFTEE: &str = "
    SELECT giftee_id 
    FROM users
    WHERE discord_id = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_GIFTEE)?;
    let giftee: i64 = query.query_row(params![santa_id as i64], |row| row.get(0))?;
    Ok(giftee as u64)
}

pub async fn set_submission(santa_id: u64, submission_content: &str) -> Result<()> {
    const UPDATE_SUBMISSION: &str = "
    UPDATE users
    SET submitted_gift = ?1
    WHERE discord_id = ?2
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(UPDATE_SUBMISSION)?;
    query.execute(params![submission_content, santa_id as i64])?;
    Ok(())
}

pub async fn get_giftee_letter(santa_id: u64) -> Result<Option<String>> {
    const GET_LETTER: &str = "
    SELECT u2.letter 
    FROM users u1
    JOIN users u2 ON u1.giftee_id = u2.discord_id
    WHERE u1.discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_LETTER)?;
    let letter = query.query_row(params![santa_id as i64], |row| Ok(row.get(0)?))?;
    Ok(letter)
}

pub async fn get_giftee_name(santa_id: u64) -> Result<String> {
    const GET_NAME: &str = "
    SELECT u2.username
    FROM users u1
    JOIN users u2 ON u1.giftee_id = u2.discord_id
    where u1.discord_id = ?1
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_NAME)?;
    let name = query.query_row(params![santa_id as i64], |row| Ok(row.get(0)?))?;
    Ok(name)
}

pub async fn get_santa(giftee_id: u64) -> Result<u64> {
    const GET_SANTA: &str = "
    SELECT discord_id
    FROM users
    WHERE giftee_id = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_SANTA)?;
    let santa: i64 = query.query_row(params![giftee_id as i64], |row| row.get(0))?;
    Ok(santa as u64)
}

pub async fn check_if_matched(user_id: u64) -> Result<bool> {
    const GET_GIFTEE: &str = "
    SELECT giftee_id
    FROM users
    WHERE discord_id = (?1);
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let mut query = conn.prepare(GET_GIFTEE)?;
    let giftee: Option<i64> = query.query_row(params![user_id as i64], |row| row.get(0))?;
    match giftee {
        Some(_) => Ok(true),
        None => Ok(false),
    }
}

pub async fn set_match(user_id1: u64, user_id2: u64) -> Result<()> {
    const UPDATE_GIFTEE: &str = "
    UPDATE users
    SET giftee_id = ?1
    WHERE discord_id = ?2
    ";
    let mut conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let tx = conn.transaction()?;
    tx.execute(UPDATE_GIFTEE, params![user_id1 as i64, user_id2 as i64])?;
    tx.execute(UPDATE_GIFTEE, params![user_id2 as i64, user_id1 as i64])?;
    let res = tx.commit()?;
    Ok(res)
}

pub async fn remove_match(user_id: u64) -> Result<()> {
    const REMOVE_MATCH1: &str = "
    UPDATE users
    SET giftee_id = NULL
    WHERE discord_id = ?1;
    ";
    const REMOVE_MATCH2: &str = "
    UPDATE users
    SET giftee_id = NULL
    WHERE giftee_id = ?1;
    ";
    let mut conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let tx = conn.transaction()?;
    tx.execute(REMOVE_MATCH1, params![user_id as i64])?;
    tx.execute(REMOVE_MATCH2, params![user_id as i64])?;
    let res = tx.commit()?;
    Ok(res)
}

pub async fn get_all_users() -> Result<Vec<UserInfo>> {
    const GET_USERS: &str = "
    SELECT *
    FROM users
    ";

    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;

    let mut stmt = conn.prepare(GET_USERS).map_err(|e| {
        eprintln!("Failed to prepare statement: {}", e);
        e
    })?;

    let users = stmt
        .query_map([], |row| {
            Ok(UserInfo {
                discord_id: row.get::<_, i64>(0)? as u64,
                username: row.get(1)?,
                letter: row.get(2)?,
                submission: row.get(3)?,
                giftee_id: row.get::<_, Option<i64>>(4)?.map(|id| id as u64),
                is_banned: row.get(5)?,
                has_joined: row.get(6)?,
            })
        })
        .map_err(|e| {
            eprintln!("Failed to query users: {}", e);
            e
        })?
        .filter_map(|r| r.ok())
        .collect();

    Ok(users)
}

pub fn get_phase() -> rusqlite::Result<Phase> {
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    let value: Option<String> = conn
        .query_row(
            "SELECT value FROM settings WHERE key = 'PHASE'",
            [],
            |row| row.get(0),
        )
        .optional()?;
    Ok(value
        .and_then(|s| Phase::from_str(&s))
        .unwrap_or(Phase::Join))
}

pub fn set_phase(phase: Phase) -> rusqlite::Result<()> {
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(
        "INSERT INTO settings (key, value) VALUES ('PHASE', ?1)
         ON CONFLICT(key) DO UPDATE SET value = excluded.value",
        [phase.as_str()],
    )?;
    Ok(())
}

pub fn rejoin_user(user_id: u64) -> rusqlite::Result<()> {
    let sql = "
        UPDATE users
        SET has_joined = 1
        WHERE discord_id = ?1;
    ";

    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;

    let rows_updated = conn.execute(sql, rusqlite::params![user_id as i64])?;

    if rows_updated == 0 {
        eprintln!("No user found with Discord ID {}", user_id);
    }

    Ok(())
}

pub fn ban_user(user_id: u64) -> rusqlite::Result<()> {
    let sql: &str = "
    UPDATE users
    SET has_joined = 0, is_banned = 1
    WHERE discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(sql, rusqlite::params![user_id as i64])?;
    Ok(())
}

pub fn unban_user(user_id: u64) -> rusqlite::Result<()> {
    let sql: &str = "
    UPDATE users
    SET is_banned = 0
    WHERE discord_id = ?1;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(sql, rusqlite::params![user_id as i64])?;
    Ok(())
}

pub fn ban_and_reassign_user(user_id: u64) -> rusqlite::Result<()> {
    let mut conn = Connection::open(PATH)?;

    let tx = conn.transaction()?;

    let giftee_id: i64 = tx.query_row(
        "
        SELECT giftee_id
        FROM users
        WHERE discord_id = ?1
        ",
        params![user_id as i64],
        |row| row.get(0),
    )?;

    tx.execute(
        "
        UPDATE users
        SET giftee_id = ?1
        WHERE giftee_id = ?2
        ",
        params![giftee_id, user_id as i64],
    )?;

    tx.execute(
        "
        UPDATE users
        SET
            has_joined = 0,
            is_banned = 1,
            giftee_id = NULL
        WHERE discord_id = ?1
        ",
        params![user_id as i64],
    )?;

    tx.commit()?;

    Ok(())
}

pub fn reset_giftees_and_submissions() -> rusqlite::Result<()> {
    let sql = "
    UPDATE users
    SET submitted_gift = NULL, giftee_id = NULL;
    ";
    let conn = Connection::open(PATH).map_err(|e| {
        eprintln!("Failed to open database: {}", e);
        e
    })?;
    conn.execute(sql, ())?;
    Ok(())
}
pub fn match_users() -> rusqlite::Result<()> {
    let mut conn = Connection::open(PATH)?;
    let tx = conn.transaction()?;

    let user_ids: Vec<i64> = {
        let mut stmt = tx.prepare(
            "
            SELECT discord_id
            FROM users
            WHERE letter IS NOT NULL
              AND has_joined = 1
              AND is_banned = 0
            ORDER BY letter
            ",
        )?;

        stmt.query_map([], |row| row.get(0))?
            .collect::<rusqlite::Result<Vec<i64>>>()?
    };

    if user_ids.len() < 2 {
        tx.commit()?;
        return Ok(());
    }

    for i in 0..user_ids.len() {
        let user_id = user_ids[i];
        let next_user_id = user_ids[(i + 1) % user_ids.len()];

        tx.execute(
            "
            UPDATE users
            SET giftee_id = ?1
            WHERE discord_id = ?2
            ",
            params![next_user_id, user_id],
        )?;
    }

    tx.commit()?;

    Ok(())
}

pub fn is_user_banned(user_id: u64) -> rusqlite::Result<bool> {
    let conn = Connection::open(PATH)?;

    let is_banned: Option<bool> = conn
        .query_row(
            "
            SELECT is_banned
            FROM users
            WHERE discord_id = ?1
            ",
            params![user_id as i64],
            |row| row.get(0),
        )
        .optional()?;

    Ok(is_banned.unwrap_or(false))
}

pub fn get_matching_order() -> rusqlite::Result<Vec<(u64, String)>> {
    let conn = Connection::open(PATH)?;

    let (start_id, start_name): (i64, String) = conn.query_row(
        "
        SELECT discord_id, username
        FROM users
        WHERE has_joined = 1
          AND is_banned = 0
          AND giftee_id IS NOT NULL
        ORDER BY letter
        LIMIT 1
        ",
        [],
        |row| Ok((row.get(0)?, row.get(1)?)),
    )?;

    let mut result = Vec::new();

    let mut current_id = start_id;
    let mut current_name = start_name;

    loop {
        result.push((current_id as u64, current_name));

        let next: (i64, String) = conn.query_row(
            "
            SELECT discord_id, username
            FROM users
            WHERE discord_id = (
                SELECT giftee_id
                FROM users
                WHERE discord_id = ?1
            )
            ",
            params![current_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        current_id = next.0;
        current_name = next.1;

        // We've completed the circle.
        if current_id == start_id {
            break;
        }
    }

    Ok(result)
}
