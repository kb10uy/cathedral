use std::path::PathBuf;

use clap::Parser;

/// Parses BEMANIwiki-style song table.
#[derive(Debug, Clone, Parser)]
pub struct Arguments {
    pub sqlite_file: PathBuf,
    pub table_html: PathBuf,

    #[clap(short = 'v', long)]
    pub default_version: Option<String>,
}
