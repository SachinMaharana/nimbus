use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Args {
    #[structopt(
        short = "c",
        long = "cloudflare_token",
        env,
        hide_env_values = true,
        required = true
    )]
    pub cloudflare_token: String,
    #[structopt(subcommand)]
    pub cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "baadal", about = "interact with cloudflare")]
pub enum Command {
    /// List the dns records for a zone
    #[structopt(name = "list")]
    List,
    /// Create a dns records for a zone
    #[structopt(name = "create")]
    Create,
    /// Delete a dns records for a zone
    #[structopt(name = "delete")]
    Delete,
}
