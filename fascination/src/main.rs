mod cli;
mod data;
mod parser;

use crate::{
    cli::Arguments,
    data::{Subheader, Version},
    parser::parse_song_tr,
};

use std::fs::read_to_string;

use anyhow::{Context, Result};
use clap::Parser;
use once_cell::sync::Lazy;
use parser::{parse_subheader, parse_version};
use scraper::{Html, Selector};

static SELECTOR_TBODY: Lazy<Selector> =
    Lazy::new(|| Selector::parse("table tbody").expect("invalid selector"));
static SELECTOR_TR: Lazy<Selector> = Lazy::new(|| Selector::parse("tr").expect("invalid selector"));
static SELECTOR_TD: Lazy<Selector> = Lazy::new(|| Selector::parse("td").expect("invalid selector"));

fn main() -> Result<()> {
    let args = Arguments::parse();

    let table_html = read_to_string(&args.table_html)?;
    let table_fragment = Html::parse_fragment(&table_html);
    let tbody = table_fragment
        .select(&SELECTOR_TBODY)
        .next()
        .context("no tbody found")?;

    let mut version = if let Some(version_str) = &args.default_version {
        parse_version(version_str)?
    } else {
        Version::default()
    };
    let mut event = String::new();
    for tr in tbody.select(&SELECTOR_TR) {
        let tds: Vec<_> = tr.select(&SELECTOR_TD).collect();
        match tds.len() {
            0 => continue,
            1 => match parse_subheader(&tds)? {
                Subheader::Version(v) => {
                    version = v;
                    event.clear();
                }
                Subheader::Event(e) => {
                    event = e;
                }
            },
            13 => {
                let song = parse_song_tr(&tds)?;
                println!("{}", song.title);
            }
            _ => continue,
        }
    }

    Ok(())
}
