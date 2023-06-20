#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoteType {
    Charge,
    HellCharge,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScratchType {
    Back,
    HellBack,
    Multi,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Difficulty {
    Beginner,
    Normal,
    Hyper,
    Another,
    Leggendaria,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlaySide {
    Single,
    Double,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Diff {
    pub play_side: PlaySide,
    pub difficulty: Difficulty,
    pub level: usize,
    pub note_type: Option<NoteType>,
    pub scratch_type: Option<ScratchType>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Song {
    pub genre: String,
    pub title: String,
    pub artist: String,
    pub min_bpm: Option<usize>,
    pub max_bpm: usize,
    pub diffs: Vec<Diff>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Version {
    pub name: String,
    pub abbrev: String,
    pub songs: Vec<Song>,
}
