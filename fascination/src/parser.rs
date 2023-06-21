use crate::data::{Diff, Difficulty, NoteType, PlaySide, ScratchType, Song, Subheader, Version};

use anyhow::{ensure, Context, Result};
use scraper::ElementRef;

pub fn parse_song_tr(tds: &[ElementRef]) -> Result<(Song, Vec<Diff>)> {
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
    let title = tds[11]
        .text()
        .collect::<Vec<_>>()
        .concat()
        .trim()
        .to_string();
    let artist = tds[12].text().next().unwrap_or_default().trim().to_string();

    Ok((
        Song {
            genre,
            title,
            artist,
            min_bpm,
            max_bpm,
        },
        diffs,
    ))
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
                // eprintln!("Unknown type: {t}");
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

pub fn parse_subheader(tds: &[ElementRef]) -> Result<Subheader> {
    let all_text = tds[0].text().collect::<Vec<_>>().concat();
    ensure!(!all_text.trim().is_empty(), "invalid subheader text");

    // version header has a link to table top
    if all_text.contains('△') {
        // version header
        let link_position = all_text.find(['▲', '▼', '△']).expect("should have links");

        let version_str = all_text[..link_position].trim();
        let version = parse_version(version_str)?;
        Ok(Subheader::Version(version))
    } else {
        // event header
        Ok(Subheader::Event(all_text.trim().to_string()))
    }
}

pub fn parse_version(version_str: &str) -> Result<Version> {
    let (version_abbrev, number) = match version_str {
        // no subtitle
        "beatmania IIDX" => ("1st style", 1),

        // just "substream"
        substream if substream.contains("substream") => ("substream", 1),

        // 2nd ~ 10th style
        // "beatmania IIDX ".len() == 15
        until_10th if until_10th.contains(" style") => {
            let abbrev = &until_10th[15..];
            let number_end = ["nd", "rd", "th"]
                .iter()
                .flat_map(|s| abbrev.find(s))
                .next()
                .context("2nd~10th style parse error")?;
            let number = abbrev[..number_end]
                .trim()
                .parse()
                .context("2nd~10th style parse error")?;
            (abbrev, number)
        }

        // IIDX RED ~
        // trim version number by finding following space
        since_red => {
            let number_and_name = &since_red[15..].trim();
            let number_end = number_and_name.find(' ').expect("should have space");
            let abbrev = number_and_name[number_end..].trim();
            let number = number_and_name[..number_end]
                .trim()
                .parse()
                .context("RED~ parse error")?;
            (abbrev, number)
        }
    };

    Ok(Version {
        name: version_str.to_string(),
        abbrev: version_abbrev.to_string(),
        number,
    })
}
