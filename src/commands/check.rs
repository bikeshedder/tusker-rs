use clap::Parser;

use crate::Backend;

#[derive(Debug, Parser)]
pub struct CheckArgs {
    /// from-backend for the diff operation
    #[arg(default_value_t = Backend::Schema)]
    from: Backend,
    /// to-backend for the diff operation
    #[arg(default_value_t = Backend::Migrations)]
    to: Backend,
    /// swaps the "from" and "to" arguments creating a reverse diff
    #[arg(long, short)]
    reverse: bool,
    /// check privilege differences (ie. grant/revoke statements)
    #[arg(long, group = "group_privileges")]
    with_privileges: bool,
    /// don't check privilege differences
    #[arg(long, group = "group_privileges")]
    without_privileges: bool,
}
