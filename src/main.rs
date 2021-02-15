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

    let (zone_identifier, zone_name_selected) = handle_account_zone(&api_client)?;

    match args.cmd {
        Command::List => {
            let (dns_names_list, _) = handle_list(&api_client, zone_identifier)?;
            println!("{:?}", dns_names_list);
            Ok(())
        }
        Command::Create => return handle_create(&api_client, zone_identifier, zone_name_selected),
        Command::Delete => return handle_delete(&api_client, zone_identifier),
    }
}

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

    let mut dns_identifier_hashmap: HashMap<String, String> = HashMap::new();

    for record in dns_records.iter() {
        dns_identifier_hashmap.insert(record.name.clone(), record.id.clone());
    }

    let dns_names_list = dns_identifier_hashmap
        .keys()
        .cloned()
        .collect::<Vec<String>>();

    Ok((dns_names_list, dns_identifier_hashmap))
}

fn handle_create(
    api_client: &HttpApiClient,
    zone_identifier: String,
    zone_name_selected: String,
) -> Result<()> {
    let random_record_name: Vec<&str> =
        vec!["core", "star", "gear", "pour", "rich", "food", "bond"];

    let random_record = random_record_name
        .choose(&mut rand::thread_rng())
        .context("Error: rand")?
        .to_owned();

    let record: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Record Name")
        .default(random_record.to_string())
        .interact_text()?;

    let record = format!("{}{}{}", record, ".", zone_name_selected);

    let ip: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("IP Addr")
        .interact_text()?;

    let ipv4 = Ipv4Addr::from_str(ip.as_str())?;

    let _response = api_client
        .request(&dns::CreateDnsRecord {
            zone_identifier: zone_identifier.as_str(),
            params: dns::CreateDnsRecordParams {
                ttl: None,
                name: record.as_str(),
                proxied: Some(true),
                priority: None,
                content: DnsContent::A { content: ipv4 },
            },
        })
        .context("Error creating dns record")?;

    println!("{} created successfully", record);
    Ok(())
}

fn handle_delete(api_client: &HttpApiClient, zone_identifier: String) -> Result<()> {
    let (dns_names_list, dns_identifier_hashmap) =
        handle_list(&api_client, zone_identifier.to_owned())?;

    let defaults = vec![false; dns_names_list.len()];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick records to delete")
        .items(&dns_names_list[..])
        .defaults(&defaults[..])
        .interact()?;

    if selections.is_empty() {
        println!("You did not select anything :(");
    } else {
        println!("You selected these things:");
        for selection in selections {
            let dns_name = dns_names_list
                .get(selection)
                .context("Error: dns_name not found")?;
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Do you want to delete {} record?", dns_name))
                .interact()?
            {
                let _delete_dns_response = api_client.request(&dns::DeleteDnsRecord {
                    zone_identifier: zone_identifier.as_str(),
                    identifier: dns_identifier_hashmap
                        .get(&dns_names_list[selection])
                        .context("Error: dns_identifier_hashmap hashmap")?,
                });
                println!("DNS {} successfully deleted", dns_name);
            } else {
                println!("nevermind then. Not deleting {}", dns_name);
            }
        }
    }
    Ok(())
}

fn handle_account_zone(api_client: &HttpApiClient) -> Result<(String, String)> {
    let mut account_list: Vec<String> = Vec::new();

    let account_response = api_client.request(&account::ListAccounts { params: None });

    for account in account_response?.result.iter() {
        account_list.push(account.name.clone());
    }

    let _selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your account")
        .default(0)
        .items(&account_list[..])
        .interact()?;

    let zone_response = api_client.request(&zone::ListZones {
        params: ListZonesParams {
            status: Some(Status::Active),
            search_match: Some(SearchMatch::All),
            ..Default::default()
        },
    })?;

    let mut zone_identifier_hashmap: HashMap<String, String> = HashMap::new();

    let zone_list = zone_response.result;

    for zone in zone_list.iter() {
        zone_identifier_hashmap.insert(zone.name.clone(), zone.id.clone());
    }

    let zone_names = zone_identifier_hashmap
        .keys()
        .cloned()
        .collect::<Vec<String>>();

    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your zone")
        .default(0)
        .items(&zone_names)
        .interact()?;

    let zone_identifier = zone_identifier_hashmap
        .get(&zone_names[selected])
        .context("Error: zone_identifier_hashmap")?;

    let zone_name_selected = &zone_names[selected];
    Ok((zone_identifier.to_owned(), zone_name_selected.to_owned()))
}
