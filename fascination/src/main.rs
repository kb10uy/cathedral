mod cli;
mod data;
mod db;
mod parser;

use crate::{
    cli::Arguments,
    data::Subheader,
    db::{insert_diffs, insert_song},
    parser::parse_song_tr,
};

use std::fs::read_to_string;

use anyhow::{Context, Result};
use clap::Parser;
use db::{insert_version, open_sqlite_file};
use once_cell::sync::Lazy;
use parser::{parse_subheader, parse_version};
use scraper::{Html, Selector};

static SELECTOR_TBODY: Lazy<Selector> =
    Lazy::new(|| Selector::parse("table tbody").expect("invalid selector"));
static SELECTOR_TR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr").expect("invalid selector"));
static SELECTOR_TD: Lazy<Selector> = Lazy::new(|| Selector::parse("td").expect("invalid selector"));

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    let table_html = read_to_string(&args.table_html)?;
    let table_fragment = Html::parse_fragment(&table_html);
    let tbody = table_fragment
        .select(&SELECTOR_TBODY)
        .next()
        .context("no tbody found")?;

    let sqlite_pool = open_sqlite_file(&args.sqlite_file).await?;

    let mut version_id = if let Some(version_str) = &args.default_version {
        let version = parse_version(version_str)?;
        let id = insert_version(&sqlite_pool, &version).await?;
        Some(id)
    } else {
        None
    };
    let mut event = None;

    for tr in tbody.select(&SELECTOR_TR) {
        let tds: Vec<_> = tr.select(&SELECTOR_TD).collect();
        match tds.len() {
            0 => continue,
            1 => match parse_subheader(&tds)? {
                Subheader::Version(v) => {
                    let id = insert_version(&sqlite_pool, &v).await?;
                    println!("version inserted: {} ({id})", v.name);

                    version_id = Some(id);
                    event = None;
                }
                Subheader::Event(e) => {
                    event = Some(e);
                }
            },
            13 => {
                let (song, diffs) = parse_song_tr(&tds)?;

                let song_id = insert_song(
                    &sqlite_pool,
                    version_id.context("version unset")?,
                    event.as_deref(),
                    &song,
                )
                .await?;

                insert_diffs(&sqlite_pool, song_id, &diffs).await?;
                println!(
                    "song inserted: {} ({song_id}), {} diffs",
                    song.title,
                    diffs.len()
                );
            }
            _ => continue,
        }
    }

    Ok(())
}
