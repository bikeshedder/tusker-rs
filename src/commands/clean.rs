use anyhow::Result;
use clap::Parser;

use crate::config::Config;

#[derive(Debug, Parser)]
pub struct CleanArgs {
    /// Don't actually perform the clean up operation but rather show
    /// which queries need to be executed.
    dry_run: bool,
}

pub async fn cmd(cfg: &Config, args: &CleanArgs) -> Result<()> {
    unimplemented!()
}
