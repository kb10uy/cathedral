use std::fmt::{Display, Formatter, Result as FmtResult};

use serde::Serialize;
use sqlx::{FromRow, Type as SqlxType};

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct SongSummary {
    pub id: i64,
    pub version_abbrev: String,
    pub genre: String,
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Song {
    pub id: i64,
    pub genre: String,
    pub title: String,
    pub artist: String,
    pub min_bpm: Option<i64>,
    pub max_bpm: i64,
    pub unlock_info: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, FromRow, Serialize)]
pub struct Diff {
    pub id: i64,
    pub play_side: PlaySide,
    pub difficulty: Difficulty,
    pub level: i64,
    pub note_type: NoteType,
    pub scratch_type: ScratchType,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, SqlxType, Serialize)]
#[sqlx(rename_all = "SCREAMING_SNAKE_CASE")]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Difficulty {
    Beginner,
    Normal,
    Hyper,
    Another,
    Leggendaria,
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
