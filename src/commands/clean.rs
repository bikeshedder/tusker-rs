use clap::Parser;

#[derive(Debug, Parser)]
pub struct CleanArgs {
    /// Don't actually perform the clean up operation but rather show
    /// which queries need to be executed.
    dry_run: bool,
}
