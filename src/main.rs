#[macro_use]
extern crate prettytable;
use anyhow::Result;
use cloudflare::framework::{auth::Credentials, Environment, HttpApiClient, HttpApiClientConfig};
use prettytable::Table;
use structopt::StructOpt;

use crate::args::{Args, Command, DnsSubCommand};

use crate::handlers::{
    handle_account_zone, handle_create, handle_delete, handle_list, DnsInfo, ZoneInfo,
};

mod args;
mod handlers;
mod utils;

struct Client {
    api_client: HttpApiClient,
}

impl Client {
    fn new(token: String) -> Result<Self> {
        let credentials = Credentials::UserAuthToken { token };

        let api_client = HttpApiClient::new(
            credentials,
            HttpApiClientConfig::default(),
            Environment::Production,
        )?;
        Ok(Client { api_client })
    }
}

fn main() -> Result<()> {
    let args: Args = Args::from_args();

    let token = args.cloudflare_token.clone();
    let api_client = Client::new(token)?.api_client;

    let zone_info: ZoneInfo = handle_account_zone(&api_client)?;
    match args.cmd {
        Command::Dns(dns) => match dns.cmd {
            DnsSubCommand::List => {
                let DnsInfo { dns_names, .. } = handle_list(&api_client, zone_info)?;
                let mut table = Table::new();
                table.set_titles(row![FYc => "DNS Records"]);
                for d in dns_names {
                    table.add_row(row![d]);
                }
                table.printstd();
                Ok(())
            }
            DnsSubCommand::Create => return handle_create(&api_client, zone_info),
            DnsSubCommand::Delete => return handle_delete(&api_client, zone_info),
        },
    }
}
