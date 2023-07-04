use std::{
    fmt::{Display, Formatter, Result as FmtResult},
    num::ParseIntError,
    ops::RangeInclusive,
    str::FromStr,
};

use serde::Serialize;
use sqlx::{FromRow, Type as SqlxType};
use thiserror::Error as ThisError;

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

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FilterQuery {
    /// `versions.number = ?`
    VersionNumber(i64),

    /// `diffs.play_side = ?`
    PlaySide(PlaySide),

    /// `diffs.difficulty = ?`
    Difficulty(Difficulty),

    /// `diffs.level = ?`
    Level(i64),

    /// `(songs.min_bpm IS NOT NULL) = ?`
    Soflan(bool),

    /// `songs.cn_type = ?`
    Note(NoteType),

    /// `songs.cn_type = ?`
    Scratch(ScratchType),

    /// `songs.max_bpm BETWEEN ? AND ?`
    BpmRange(RangeInclusive<i64>),
}

impl FilterQuery {
    pub fn where_clause_str(&self) -> &'static str {
        match self {
            FilterQuery::VersionNumber(_) => "versions.number = ?",
            FilterQuery::PlaySide(_) => "diffs.play_side = ?",
            FilterQuery::Difficulty(_) => "diffs.difficulty = ?",
            FilterQuery::Level(_) => "diffs.level = ?",
            FilterQuery::Soflan(_) => "(songs.min_bpm IS NOT NULL) = ?",
            FilterQuery::Note(_) => "songs.cn_type = ?",
            FilterQuery::Scratch(_) => "songs.bss_type = ?",
            FilterQuery::BpmRange(_) => "songs.max_bpm BETWEEN ? and ?",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, ThisError)]
pub enum FilterQueryError {
    #[error("invalid query format")]
    InvalidFormat,

    #[error("unknown query type: {0}")]
    UnknownQuery(String),

    #[error("invalid query value: {0}")]
    InvalidValue(String),

    #[error("invalid number")]
    InvalidNumber(#[from] ParseIntError),
}

impl FromStr for FilterQuery {
    type Err = FilterQueryError;

    fn from_str(s: &str) -> Result<FilterQuery, FilterQueryError> {
        let mut parts = s.split(':');
        let Some(qtype) = parts.next() else {
            return Err(FilterQueryError::InvalidFormat);
        };
        let value = parts.next().unwrap_or_default();

        match qtype {
            "v" | "version" => {
                let number: i64 = value.parse()?;
                Ok(FilterQuery::VersionNumber(number))
            }
            "p" | "play" => match value {
                "s" | "sp" => Ok(FilterQuery::PlaySide(PlaySide::Single)),
                "d" | "dp" => Ok(FilterQuery::PlaySide(PlaySide::Double)),
                _ => Err(FilterQueryError::InvalidValue(value.into())),
            },
            "d" | "diff" => match value {
                "b" | "beginner" => Ok(FilterQuery::Difficulty(Difficulty::Beginner)),
                "n" | "normal" => Ok(FilterQuery::Difficulty(Difficulty::Normal)),
                "h" | "hyper" => Ok(FilterQuery::Difficulty(Difficulty::Hyper)),
                "a" | "another" => Ok(FilterQuery::Difficulty(Difficulty::Another)),
                "l" | "leggendaria" => Ok(FilterQuery::Difficulty(Difficulty::Leggendaria)),
                _ => Err(FilterQueryError::InvalidValue(value.into())),
            },
            "l" | "level" => {
                let level: i64 = value.parse()?;
                Ok(FilterQuery::Level(level))
            }
            "f" | "soflan" => match value {
                "y" | "yes" | "t" | "true" => Ok(FilterQuery::Soflan(true)),
                "n" | "no" | "f" | "false" => Ok(FilterQuery::Soflan(false)),
                _ => Err(FilterQueryError::InvalidValue(value.into())),
            },
            "n" | "note" => match value {
                "c" | "cn" => Ok(FilterQuery::Note(NoteType::Charge)),
                "h" | "hcn" => Ok(FilterQuery::Note(NoteType::Charge)),
                _ => Err(FilterQueryError::InvalidValue(value.into())),
            },
            "s" | "scratch" => match value {
                "b" | "bss" => Ok(FilterQuery::Scratch(ScratchType::Back)),
                "h" | "hbss" => Ok(FilterQuery::Scratch(ScratchType::HellBack)),
                "m" | "mss" => Ok(FilterQuery::Scratch(ScratchType::Multi)),
                _ => Err(FilterQueryError::InvalidValue(value.into())),
            },
            "b" | "bpm" => {
                let mut bpms = value.split('-');
                let first = bpms.next();
                let second = bpms.next();
                let range = match (first, second) {
                    (Some(fixed), None) => {
                        let v = fixed.parse()?;
                        v..=v
                    }
                    (Some(""), Some("")) => i64::MIN..=i64::MAX,
                    (Some(""), Some(upper)) => i64::MIN..=(upper.parse()?),
                    (Some(lower), Some("")) => (lower.parse()?)..=i64::MAX,
                    (Some(lower), Some(upper)) => (lower.parse()?)..=(upper.parse()?),
                    (None, _) => unreachable!("at least one element"),
                };
                Ok(FilterQuery::BpmRange(range))
            }
            _ => Err(FilterQueryError::UnknownQuery(qtype.into())),
        }
    }
}
