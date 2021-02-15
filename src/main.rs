use anyhow::{bail, Result};
use dns::DnsContent;
use rand::seq::SliceRandom;
use std::{collections::HashMap, str::FromStr};
use structopt::StructOpt;
use zone::{ListZonesParams, Status};

use cloudflare::{
    endpoints::{account, dns, zone},
    framework::{
        apiclient::ApiClient,
        auth::Credentials,
        response::{ApiFailure, ApiResponse, ApiResult},
        Environment, HttpApiClient, HttpApiClientConfig, OrderDirection, SearchMatch,
    },
};

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};

use std::net::Ipv4Addr;

use serde::Serialize;

#[derive(Debug, StructOpt)]
struct Args {
    #[structopt(
        short = "c",
        long = "cloudflare_token",
        env,
        hide_env_values = true,
        required = true
    )]
    cloudflare_token: String,
    #[structopt(subcommand)]
    cmd: Command,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "baadal", about = "interact with cloudflare")]
enum Command {
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

fn handle_list(api_client: &HttpApiClient, zone_identifier: String) -> Result<()> {
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

    println!("{:?}", dns_names);
    Ok(())
}

fn handle_create(
    api_client: &HttpApiClient,
    zone_identifier: String,
    selected_zone: String,
) -> Result<()> {
    let words: Vec<&str> = vec!["core", "star", "gear", "pour", "rich", "food", "bond"];

    let word = words.choose(&mut rand::thread_rng()).unwrap().to_owned();

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
    let defaults = vec![false; dns_names.len()];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick records to delete")
        .items(&dns_names[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    if selections.is_empty() {
        println!("You did not select anything :(");
    } else {
        println!("You selected these things:");
        for selection in selections {
            if Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!(
                    "Do you want to delete {} record?",
                    dns_names.get(selection).unwrap()
                ))
                .interact()
                .unwrap()
            {
                let response = api_client.request(&dns::DeleteDnsRecord {
                    zone_identifier: zone_identifier.as_str(),
                    identifier: dns_with_iden.get(&dns_names[selection]).unwrap(),
                });
                print_response(response);
            } else {
                println!(
                    "nevermind then. Not deleting {}",
                    dns_names.get(selection).unwrap()
                );
            }
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::from_args();

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

    let mut account: Vec<String> = Vec::new();

    let response = api_client.request(&account::ListAccounts { params: None });

    for i in response.unwrap().result.iter() {
        account.push(i.name.clone());
    }

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your account")
        .default(0)
        .items(&account[..])
        .interact()
        .unwrap();

    println!("Enjoy your {}!", account[selection]);

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

    let zone_identifier = zone_with_iden.get(&items[selection]).unwrap();

    let selected_zone = &items[selection];

    match args.cmd {
        Command::List => return handle_list(&api_client, zone_identifier.to_owned()),
        Command::Create => {
            return handle_create(
                &api_client,
                zone_identifier.to_owned(),
                selected_zone.to_owned(),
            )
        }
        Command::Delete => return handle_delete(&api_client, zone_identifier.to_owned()),
    }
}

fn print_response_json<T: ApiResult>(response: &ApiResponse<T>)
where
    T: Serialize,
{
    match response {
        Ok(success) => {
            let js = serde_json::to_string(&success.result).unwrap();
            // let array: Vec<Foo> = serde_json::from_str(&success.result).unwrap();
            // for i in js.iter() {}
            println!("{}", js);
        }
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                println!("HTTP {}", status);
                for err in &errors.errors {
                    println!("Error {}: {}", err.code, err.message);
                    for (k, v) in &err.other {
                        println!("{}: {}", k, v);
                    }
                }
                for (k, v) in &errors.other {
                    println!("{}: {}", k, v);
                }
            }
            ApiFailure::Invalid(req_err) => println!("Error: {}", req_err),
        },
    }
}

fn print_response<T: ApiResult>(response: ApiResponse<T>) {
    match response {
        Ok(success) => println!("Success: {:#?}", success),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                println!("HTTP {}:", status);
                for err in errors.errors {
                    println!("Error {}: {}", err.code, err.message);
                    for (k, v) in err.other {
                        println!("{}: {}", k, v);
                    }
                }
                for (k, v) in errors.other {
                    println!("{}: {}", k, v);
                }
            }
            ApiFailure::Invalid(reqwest_err) => println!("Error: {}", reqwest_err),
        },
    }
}
