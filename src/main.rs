use anyhow::Result;
use cloudflare::framework::{auth::Credentials, Environment, HttpApiClient, HttpApiClientConfig};
use structopt::StructOpt;

use crate::args::{Args, Command};

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

    let token = args.cloudflare_token;
    let api_client = Client::new(token)?.api_client;

    let zone_info: ZoneInfo = handle_account_zone(&api_client)?;

    match args.cmd {
        Command::List => {
            let DnsInfo { dns_names, .. } = handle_list(&api_client, zone_info)?;
            println!("{:?}", dns_names);
            Ok(())
        }
        Command::Create => return handle_create(&api_client, zone_info),
        Command::Delete => return handle_delete(&api_client, zone_info),
    }
}
