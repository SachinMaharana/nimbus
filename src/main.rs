use anyhow::{bail, Context, Result};
use dns::DnsContent;
use rand::seq::SliceRandom;
use std::{collections::HashMap, str::FromStr};
use structopt::StructOpt;
use zone::{ListZonesParams, Status};

use cloudflare::{
    endpoints::{account, dns, zone},
    framework::{
        apiclient::ApiClient, auth::Credentials, Environment, HttpApiClient, HttpApiClientConfig,
        OrderDirection, SearchMatch,
    },
};

use crate::args::{Args, Command};

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};

use std::net::Ipv4Addr;

mod args;
mod utils;

fn handle_list(
    api_client: &HttpApiClient,
    zone_identifier: String,
) -> Result<(Vec<String>, HashMap<String, String>)> {
    let dns_list_response = api_client.request(&dns::ListDnsRecords {
        zone_identifier: zone_identifier.as_str(),
        params: dns::ListDnsRecordsParams {
            direction: Some(OrderDirection::Ascending),
            ..Default::default()
        },
    })?;

    let dns_records = dns_list_response.result;

    let mut dns_with_iden: HashMap<String, String> = HashMap::new();

    for item in dns_records.iter() {
        dns_with_iden.insert(item.name.clone(), item.id.clone());
    }

    let dns_names = dns_with_iden.keys().cloned().collect::<Vec<String>>();

    Ok((dns_names, dns_with_iden))
}

fn handle_create(
    api_client: &HttpApiClient,
    zone_identifier: String,
    selected_zone: String,
) -> Result<()> {
    let words: Vec<&str> = vec!["core", "star", "gear", "pour", "rich", "food", "bond"];

    let word = words
        .choose(&mut rand::thread_rng())
        .context("Error: rand")?
        .to_owned();

    let record: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Record Name")
        .default(word.to_string())
        .interact_text()?;

    let record = format!("{}{}{}", record, ".", selected_zone);

    let ip: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("IP Addr")
        .interact_text()?;

    let ipv4 = Ipv4Addr::from_str(ip.as_str())?;

    let _response = api_client.request(&dns::CreateDnsRecord {
        zone_identifier: zone_identifier.as_str(),
        params: dns::CreateDnsRecordParams {
            ttl: None,
            name: record.as_str(),
            proxied: Some(true),
            priority: None,
            content: DnsContent::A { content: ipv4 },
        },
    });
    Ok(())
}

fn handle_delete(api_client: &HttpApiClient, zone_identifier: String) -> Result<()> {
    let (dns_names, dns_with_iden) = handle_list(&api_client, zone_identifier.to_owned())?;

    let defaults = vec![false; dns_names.len()];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick records to delete")
        .items(&dns_names[..])
        .defaults(&defaults[..])
        .interact()?;

    if selections.is_empty() {
        println!("You did not select anything :(");
    } else {
        println!("You selected these things:");
        for selection in selections {
            let dns_name = dns_names
                .get(selection)
                .context("Error: dns_name not found")?;
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Do you want to delete {} record?", dns_name))
                .interact()?
            {
                let response = api_client.request(&dns::DeleteDnsRecord {
                    zone_identifier: zone_identifier.as_str(),
                    identifier: dns_with_iden
                        .get(&dns_names[selection])
                        .context("Error: dns_with_iden hashmap")?,
                });
                utils::print_response(response);
            } else {
                println!("nevermind then. Not deleting {}", dns_name);
            }
        }
    }
    Ok(())
}

fn handle_account_zone(api_client: &HttpApiClient) -> Result<(String, String)> {
    let mut account: Vec<String> = Vec::new();

    let response = api_client.request(&account::ListAccounts { params: None });

    for i in response?.result.iter() {
        account.push(i.name.clone());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your account")
        .default(0)
        .items(&account[..])
        .interact()?;

    let response = api_client.request(&zone::ListZones {
        params: ListZonesParams {
            status: Some(Status::Active),
            search_match: Some(SearchMatch::All),
            ..Default::default()
        },
    })?;
    let mut zone_with_iden: HashMap<String, String> = HashMap::new();

    let zone = response.result;

    for item in zone.iter() {
        zone_with_iden.insert(item.name.clone(), item.id.clone());
    }

    let items = zone_with_iden.keys().cloned().collect::<Vec<String>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your zone")
        .default(0)
        .items(&items)
        .interact()?;

    let zone_identifier = zone_with_iden
        .get(&items[selection])
        .context("Error: zone_with_iden")?;

    let selected_zone = &items[selection];
    Ok((zone_identifier.to_owned(), selected_zone.to_owned()))
}

fn main() -> Result<()> {
    let args: Args = Args::from_args();

    let token: Option<String> = Some((&args.cloudflare_token).to_owned());

    let credentials = if let Some(token) = token {
        Credentials::UserAuthToken { token: token }
    } else {
        bail!("API token must be provided")
    };

    let api_client = HttpApiClient::new(
        credentials,
        HttpApiClientConfig::default(),
        Environment::Production,
    )?;

    let (zone_identifier, selected_zone) = handle_account_zone(&api_client)?;

    match args.cmd {
        Command::List => {
            let (dns_names, _) = handle_list(&api_client, zone_identifier)?;
            println!("{:?}", dns_names);
            Ok(())
        }
        Command::Create => return handle_create(&api_client, zone_identifier, selected_zone),
        Command::Delete => return handle_delete(&api_client, zone_identifier),
    }
}
