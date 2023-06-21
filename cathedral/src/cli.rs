use std::{net::SocketAddr, path::PathBuf};

use clap::Parser;

#[derive(Debug, Parser)]
pub struct Arguments {
    pub sqlite_filename: PathBuf,

    #[clap(short, long, default_value = "0.0.0.0:3000")]
    pub bind: SocketAddr,
}
