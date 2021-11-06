use std::net::IpAddr;
use std::path::PathBuf;

use structopt::StructOpt;

#[derive(StructOpt, Debug)]
pub struct Options {
    #[structopt(short = "s", long, default_value = ".")]
    pub storage: PathBuf,
    #[structopt(short = "w", long)]
    pub watch_storage: bool,
    #[structopt(short = "a", long, default_value = "127.0.0.1")]
    pub address: IpAddr,
    #[structopt(short = "p", long, default_value = "8080")]
    pub port: u16,
}
