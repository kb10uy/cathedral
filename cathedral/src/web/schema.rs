use crate::db::schema::{Diff, Song};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize)]
pub struct ErrorResult {
    pub reason: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongsSearchQuery {
    pub q: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongsSearchResponse {
    pub version_abbrev: String,
    pub id: i64,
    pub genre: String,
    pub title: String,
    pub artist: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct SongsShowQuery {
    pub id: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct SongsShowResponse {
    pub song: Song,
    pub diffs: Vec<Diff>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MattermostEnqueueForm {
    pub token: String,
    pub text: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct MattermostEnqueueResult {
    pub username: String,
    pub attachments: Vec<AttachmentSongInfo>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentSongInfo {
    pub title: String,
    pub footer: String,
    pub fields: Vec<AttachmentSongField>,
}

#[derive(Debug, Clone, Serialize)]
pub struct AttachmentSongField {
    pub short: bool,
    pub title: String,
    pub value: String,
}
