#[macro_use]
extern crate prettytable;
use crate::args::{Args, Command, DnsSubCommand};
use anyhow::Result;
use cloudflare::endpoints::dns::DnsContent;
use cloudflare::framework::{auth::Credentials, Environment, HttpApiClient, HttpApiClientConfig};
use prettytable::{
    format::{self, Alignment},
    Cell, Row, Table,
};
use structopt::StructOpt;

use crate::handlers::{
    handle_account_zone, handle_create, handle_delete, handle_list, handle_patch, DnsInfo, ZoneInfo,
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
            DnsSubCommand::List => return show_list(&api_client, zone_info),
            DnsSubCommand::Create => return handle_create(&api_client, zone_info),
            DnsSubCommand::Delete => return handle_delete(&api_client, zone_info),
            DnsSubCommand::Update => return handle_patch(&api_client, zone_info),
        },
    }
}

fn show_list(api_client: &HttpApiClient, zone_info: ZoneInfo) -> Result<()> {
    let DnsInfo {
        dns_identifier_hashmap,
    } = handle_list(&api_client, zone_info)?;

    let mut table = Table::new();

    table.set_format(*format::consts::FORMAT_BOX_CHARS);
    table.set_titles(Row::new(vec![Cell::new_align(
        "DNS RECORDS",
        Alignment::CENTER,
    )
    .with_hspan(3)]));
    for (k, v) in dns_identifier_hashmap {
        match v.dns_content {
            DnsContent::A { content } => {
                table.add_row(row![k, content, "A"]);
            }
            DnsContent::CNAME { content } => {
                table.add_row(row![k, content, "CNAME"]);
            }
            _ => {}
        }
    }
    // print table
    table.printstd();
    Ok(())
}
