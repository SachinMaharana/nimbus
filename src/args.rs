use nimbus::terminal::emoji;
use structopt::clap::AppSettings;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    version = env!("CARGO_PKG_VERSION"),
    author = env!("CARGO_PKG_AUTHORS"),
    name = format!("{}  {} nimbus", emoji::CLOUD, emoji::SPARKLE),
    about = "nimbus cli interacts with cloudflare for managing dns records"
)]
pub struct Args {
    #[structopt(
        short = "c",
        long = "cloudflare_token",
        env,
        hide_env_values = true,
        required = true,
        help = "or export env variable"
    )]
    pub cloudflare_token: String,
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
pub enum Command {
    #[structopt(name = "dns",global_settings = &[AppSettings::DisableVersion,AppSettings::DeriveDisplayOrder,AppSettings::ArgRequiredElseHelp])]
    Dns(Dns),
}

#[derive(StructOpt, Debug)]
pub struct Dns {
    #[structopt(subcommand)]
    pub cmd: DnsSubCommand,
}

#[derive(StructOpt, Debug)]
pub enum DnsSubCommand {
    /// List the dns records for a zone
    #[structopt(name = "list")]
    List,
    /// Create a dns records for a zone
    #[structopt(name = "create")]
    Create,
    /// Delete a dns records for a zone
    #[structopt(name = "delete")]
    Delete,
    /// Delete a dns records for a zone
    #[structopt(name = "update")]
    Update,
}
