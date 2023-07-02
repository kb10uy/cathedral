use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::Serialize;
use sqlx::{FromRow, Type as SqlxType};

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Version {
    #[sqlx(rename = "version_id")]
    pub id: i64,
    #[sqlx(rename = "version_name")]
    pub name: String,
    #[sqlx(rename = "version_abbrev")]
    pub abbrev: String,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Song {
    pub version_id: i64,

    #[sqlx(rename = "song_id")]
    pub id: i64,
    #[sqlx(rename = "song_genre")]
    pub genre: String,
    #[sqlx(rename = "song_title")]
    pub title: String,
    #[sqlx(rename = "song_artist")]
    pub artist: String,
    #[sqlx(rename = "song_min_bpm")]
    pub min_bpm: Option<i64>,
    #[sqlx(rename = "song_max_bpm")]
    pub max_bpm: i64,
    #[sqlx(rename = "song_unlock_info")]
    pub unlock_info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Diff {
    pub song_id: i64,

    #[sqlx(rename = "diff_play_side")]
    pub play_side: PlaySide,
    #[sqlx(rename = "diff_difficulty")]
    pub difficulty: Difficulty,
    #[sqlx(rename = "diff_level")]
    pub level: i64,
    #[sqlx(rename = "diff_note_type")]
    pub note_type: Option<NoteType>,
    #[sqlx(rename = "diff_scratch_type")]
    pub scratch_type: Option<ScratchType>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SqlxType, Serialize)]
pub enum PlaySide {
    #[sqlx(rename = "SP")]
    #[serde(rename = "SP")]
    Single,
    #[sqlx(rename = "DP")]
    #[serde(rename = "DP")]
    Double,
}

impl Display for PlaySide {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            PlaySide::Single => f.write_str("SP"),
            PlaySide::Double => f.write_str("DP"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, SqlxType, Serialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Difficulty {
    Beginner,
    Normal,
    Hyper,
    Another,
    Leggendaria,
}

impl Difficulty {
    pub fn to_emoji_str(self) -> &'static str {
        match self {
            Difficulty::Beginner => ":diff-b:",
            Difficulty::Normal => ":diff-n:",
            Difficulty::Hyper => ":diff-h:",
            Difficulty::Another => ":diff-a:",
            Difficulty::Leggendaria => ":diff-l:",
        }
    }
}

impl Display for Difficulty {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            Difficulty::Beginner => f.write_str("BEGINNER"),
            Difficulty::Normal => f.write_str("NORMAL"),
            Difficulty::Hyper => f.write_str("HYPER"),
            Difficulty::Another => f.write_str("ANOTHER"),
            Difficulty::Leggendaria => f.write_str("LEGGENDARIA"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SqlxType, Serialize)]
pub enum NoteType {
    #[sqlx(rename = "CN")]
    #[serde(rename = "CN")]
    Charge,
    #[sqlx(rename = "HCN")]
    #[serde(rename = "HCN")]
    HellCharge,
}

impl Display for NoteType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            NoteType::Charge => f.write_str("CN"),
            NoteType::HellCharge => f.write_str("HCN"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, SqlxType, Serialize)]
pub enum ScratchType {
    #[sqlx(rename = "BSS")]
    #[serde(rename = "BSS")]
    Back,
    #[sqlx(rename = "HBSS")]
    #[serde(rename = "HBSS")]
    HellBack,
    #[sqlx(rename = "MSS")]
    #[serde(rename = "MSS")]
    Multi,
}

impl Display for ScratchType {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        match self {
            ScratchType::Back => f.write_str("BSS"),
            ScratchType::HellBack => f.write_str("HBSS"),
            ScratchType::Multi => f.write_str("MSS"),
        }
    }
}
