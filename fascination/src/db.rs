use crate::data::{Diff, Difficulty, NoteType, PlaySide, ScratchType, Song, Version};

use std::path::Path;

use anyhow::Result;
use sqlx::{query, Row, SqlitePool};

pub async fn open_sqlite_file(path: &Path) -> Result<SqlitePool> {
    let conn = SqlitePool::connect(&format!(
        "sqlite://{}",
        path.to_str().expect("invalid filename")
    ))
    .await?;
    Ok(conn)
}

pub async fn insert_version(pool: &SqlitePool, version: &Version) -> Result<i64> {
    let returned_id = query(
        r#"
        INSERT INTO "versions" ("name", "number", "abbrev")
        VALUES (?, ?, ?)
        RETURNING "id";
        "#,
    )
    .bind(&version.name)
    .bind(version.number as i64)
    .bind(&version.abbrev)
    .fetch_one(pool)
    .await?;

    Ok(returned_id.get("id"))
}

pub async fn insert_song(
    pool: &SqlitePool,
    version_id: i64,
    event: Option<&str>,
    song: &Song,
) -> Result<i64> {
    let returned_id = query(
        r#"
        INSERT INTO "songs" ("version_id", "genre", "title", "artist", "min_bpm", "max_bpm", "unlock_info")
        VALUES (?, ?, ?, ?, ?, ?, ?)
        RETURNING "id";
        "#,
    )
    .bind(version_id)
    .bind(&song.genre)
    .bind(&song.title)
    .bind(&song.artist)
    .bind(song.min_bpm.map(|x| x as i64))
    .bind(song.max_bpm as i64)
    .bind(event)
    .fetch_one(pool)
    .await?;

    Ok(returned_id.get("id"))
}

pub async fn insert_diffs(pool: &SqlitePool, song_id: i64, diffs: &[Diff]) -> Result<()> {
    for diff in diffs {
        let play_side = match diff.play_side {
            PlaySide::Single => "SP",
            PlaySide::Double => "DP",
        };
        let difficulty = match diff.difficulty {
            Difficulty::Beginner => "BEGINNER",
            Difficulty::Normal => "NORMAL",
            Difficulty::Hyper => "HYPER",
            Difficulty::Another => "ANOTHER",
            Difficulty::Leggendaria => "LEGGENDARIA",
        };
        let cn_type = diff.note_type.map(|nt| match nt {
            NoteType::Charge => "CN",
            NoteType::HellCharge => "HCN",
        });
        let bss_type = diff.scratch_type.map(|st| match st {
            ScratchType::Back => "BSS",
            ScratchType::HellBack => "HBSS",
            ScratchType::Multi => "MSS",
        });

        sqlx::query(
            r#"
            INSERT INTO "diffs" ("song_id", "play_side", "difficulty", "level", "cn_type", "bss_type")
            VALUES (?, ?, ?, ?, ?, ?);
            "#,
        )
        .bind(song_id)
        .bind(play_side)
        .bind(difficulty)
        .bind(diff.level as i64)
        .bind(cn_type)
        .bind(bss_type)
        .execute(pool)
        .await?;
    }

    Ok(())
}
