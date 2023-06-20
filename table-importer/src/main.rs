use std::{env::args, fs::read_to_string};

use anyhow::{bail, ensure, Context, Result};
use dropkick::{Difficulty, NoteType, PlaySide, ScratchType, Song, Diff};
use once_cell::sync::Lazy;
use scraper::{ElementRef, Html, Selector};

static SELECTOR_TBODY: Lazy<Selector> =
    Lazy::new(|| Selector::parse("table tbody").expect("invalid selector"));
static SELECTOR_TR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr").expect("invalid selector"));
static SELECTOR_TD: Lazy<Selector> = Lazy::new(|| Selector::parse("td").expect("invalid selector"));

fn main() -> Result<()> {
    let args: Vec<_> = args().collect();
    if args.len() <= 1 {
        bail!("Usage: table-importer <table html>")
    }
    let table_html = read_to_string(&args[1])?;
    let table_fragment = Html::parse_fragment(&table_html);
    let tbody = table_fragment
        .select(&SELECTOR_TBODY)
        .next()
        .context("no tbody found")?;

    let mut version = String::new();
    let mut event = String::new();
    for tr in tbody.select(&SELECTOR_TR) {
        let tds: Vec<_> = tr.select(&SELECTOR_TD).collect();
        match tds.len() {
            0 => continue,
            1 => {
                let all_text = tds[0].text().collect::<Vec<_>>().concat();
                if all_text.trim().is_empty() {
                    continue;
                }

                // version header has a link to table top
                if all_text.contains('△') {
                    // version header
                    let link_position = all_text.find(['▲', '▼', '△']).expect("should have links");

                    version = all_text[..link_position].trim().to_string();
                    let version_abbrev = match version.as_str() {
                        // no subtitle
                        "beatmania IIDX" => &version,

                        // just "substream"
                        ss if ss.contains("substream") => "substream",

                        // 2nd ~ 10th style
                        // "beatmania IIDX ".len() == 15
                        until_10th if until_10th.contains(" style") => &until_10th[15..],

                        // IIDX RED ~
                        // trim version number by finding following space
                        since_red => {
                            let version_and_name = &since_red[15..];
                            let space_position =
                                version_and_name.find(' ').expect("should have space");
                            version_and_name[space_position..].trim()
                        }
                    };
                    event.clear();
                    println!("# {version_abbrev}");
                } else {
                    // event header
                    event = all_text.trim().to_string();
                    // println!("## {event}");
                }
            }
            13 => {
                let song = parse_song_tr(&tds)?;
                //println!("{song:?}");
                //println!();
            }
            _ => continue,
        }
    }

    Ok(())
}

fn parse_song_tr(tds: &[ElementRef]) -> Result<Song> {
    ensure!(tds.len() == 13, "malformed row");
    // SPB SPN SPH SPA SPL | DPN DPH DPA DPL | BPM GENRE TITLE ARTIST
    let raw_diffs = vec![
        parse_diff_td(PlaySide::Single, Difficulty::Beginner, tds[0]),
        parse_diff_td(PlaySide::Single, Difficulty::Normal, tds[1]),
        parse_diff_td(PlaySide::Single, Difficulty::Hyper, tds[2]),
        parse_diff_td(PlaySide::Single, Difficulty::Another, tds[3]),
        parse_diff_td(PlaySide::Single, Difficulty::Leggendaria, tds[4]),
        parse_diff_td(PlaySide::Double, Difficulty::Normal, tds[5]),
        parse_diff_td(PlaySide::Double, Difficulty::Hyper, tds[6]),
        parse_diff_td(PlaySide::Double, Difficulty::Another, tds[7]),
        parse_diff_td(PlaySide::Double, Difficulty::Leggendaria, tds[8]),
    ];
    let diffs = raw_diffs.into_iter().flatten().collect();

    let (min_bpm, max_bpm) = {
        let mut bpm_text = tds[9].text().next().unwrap_or_default().split('-');
        let first = bpm_text.next();
        let second = bpm_text.next();
        match (first, second) {
            // Indeterminate tempo
            (Some("※"), None) => (None, 0usize),

            (Some(min), Some(max)) => (Some(min.parse()?), max.parse()?),
            (Some(bpm), None) => (None, bpm.parse()?),
            _ => (None, 0usize),
        }
    };
    let genre = tds[10].text().next().unwrap_or_default().trim().to_string();
    let title = tds[11].text().next().unwrap_or_default().trim().to_string();
    let artist = tds[12].text().next().unwrap_or_default().trim().to_string();

    Ok(Song {
        genre,
        title,
        artist,
        min_bpm,
        max_bpm,
        diffs,
    })
}

fn parse_diff_td(play_side: PlaySide, difficulty: Difficulty, td: ElementRef) -> Option<Diff> {
    let mut level = 0;
    let mut note_type = None;
    let mut scratch_type = None;

    let texts: Vec<_> = td.text().collect();
    if texts.contains(&"-") {
        return None;
    }

    for text_part in texts {
        match text_part {
            "[CN]" => {
                note_type = Some(NoteType::Charge);
            }
            "[HCN]" => {
                note_type = Some(NoteType::HellCharge);
            }
            "[BSS]" => {
                scratch_type = Some(ScratchType::Back);
            }
            "[HBSS]" => {
                scratch_type = Some(ScratchType::HellBack);
            }
            "[MSS]" => {
                scratch_type = Some(ScratchType::Multi);
            }
            t if t.contains('[') => {
                eprintln!("Unknown type: {t}");
            }
            l => {
                level = l.parse().unwrap_or(0);
            }
        }
    }

    Some(Diff {
        play_side,
        difficulty,
        level,
        note_type,
        scratch_type,
    })
}
