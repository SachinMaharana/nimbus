#![allow(unused_imports)]
#![allow(dead_code)]

use anyhow::{bail, Result};
use dns::{DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use env::VarError;
use rand::seq::SliceRandom;
use rand::thread_rng;
use serde_json::Value;
use std::{any::type_name, collections::HashMap, env, iter::successors, str::FromStr};
use zone::{ListZonesParams, Status, ZoneDetails};

use cloudflare::{
    endpoints::{account, dns, user, zone},
    framework::{
        apiclient::ApiClient,
        auth::Credentials,
        response::{ApiFailure, ApiResponse, ApiResult},
        Environment, HttpApiClient, HttpApiClientConfig, OrderDirection, SearchMatch,
    },
};

use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};

use std::net::Ipv4Addr;

use serde::{Deserialize, Serialize};

fn main() -> Result<()> {
    let token = env::var("CLOUDFLARE_TOKEN").ok();
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

    // let mut account: Vec<String> = Vec::new();

    // let response = api_client.request(&account::ListAccounts { params: None });

    // print_response_json(&response);

    // for i in response.unwrap().result.iter() {
    //     account.push(i.name.clone());
    // }

    // let selection = Select::with_theme(&ColorfulTheme::default())
    //     .with_prompt("Pick your account")
    //     .default(0)
    //     .items(&account[..])
    //     .interact()
    //     .unwrap();

    // println!("Enjoy your {}!", account[selection]);

    // let r = &response.unwrap();

    // print_response_json(response);

    // let response = api_client.request(&user::GetUserDetails {});
    // print_response_json(response);

    // let response = api_client.request(&user::GetUserTokenStatus {});
    // print_response_json(response);

    let response = api_client
        .request(&zone::ListZones {
            params: ListZonesParams {
                status: Some(Status::Active),
                search_match: Some(SearchMatch::All),
                ..Default::default()
            },
        })
        .unwrap();

    let mut zone_with_iden: HashMap<String, String> = HashMap::new();

    let zone = response.result;

    for item in zone.iter() {
        zone_with_iden.insert(item.name.clone(), item.id.clone());
    }

    let items = zone_with_iden.keys().cloned().collect::<Vec<String>>();

    let selection = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your zone: ")
        .default(0)
        .items(&items)
        .interact()
        .unwrap();

    println!("Enjoy your {}!", items[selection]);

    let zone_identifier = zone_with_iden.get(&items[selection]).unwrap();
    // let words: Vec<&str> = vec!["core", "star", "gear", "pour", "rich", "food", "bond"];

    // let word = words.choose(&mut rand::thread_rng()).unwrap().to_owned();

    // let record: String = Input::with_theme(&ColorfulTheme::default())
    //     .with_prompt("Record Name")
    //     .default(word.to_string())
    //     .interact_text()
    //     .unwrap();

    // let record = format!("{}{}", record, ".sachinmaharana.com".to_string());

    // let ip: String = Input::with_theme(&ColorfulTheme::default())
    //     .with_prompt("IP Addr")
    //     .interact_text()
    //     .unwrap();
    // let ipv4 = Ipv4Addr::from_str(ip.as_str()).unwrap();

    // let _response = api_client.request(&dns::CreateDnsRecord {
    //     zone_identifier: zone_identifier.as_str(),
    //     params: dns::CreateDnsRecordParams {
    //         ttl: None,
    //         name: record.as_str(),
    //         proxied: Some(true),
    //         priority: None,
    //         content: DnsContent::A { content: ipv4 },
    //     },
    // });

    // print_response(rr);

    // let response = api_client.request(&zone::ZoneDetails {
    //     identifier: "013629251ecc87b23edc9532e02ef4ba",
    // });
    // print_response(response);

    let dns_list_response = api_client
        .request(&dns::ListDnsRecords {
            zone_identifier: zone_identifier.as_str(),
            params: dns::ListDnsRecordsParams {
                direction: Some(OrderDirection::Ascending),
                ..Default::default()
            },
        })
        .unwrap();

    let dns_records = dns_list_response.result;

    let mut dns_with_iden: HashMap<String, String> = HashMap::new();

    for item in dns_records.iter() {
        dns_with_iden.insert(item.name.clone(), item.id.clone());
    }

    let dns_names = dns_with_iden.keys().cloned().collect::<Vec<String>>();

    let defaults = vec![false; dns_names.len()];
    let selections = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt("Pick your food")
        .items(&dns_names[..])
        .defaults(&defaults[..])
        .interact()
        .unwrap();

    println!("{:?}", selections);
    println!("{:?}", dns_names);

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

    // print_response(response);

    // let response = api_client.request(&dns::CreateDnsRecord {
    //     zone_identifier: "013629251ecc87b23edc9532e02ef4ba",
    //     params: dns::CreateDnsRecordParams {
    //         ttl: None,
    //         name: "ch.sachinmaharana.com",
    //         proxied: Some(true),
    //         priority: None,
    //         content: DnsContent::A {
    //             content: Ipv4Addr::new(144, 202, 47, 216),
    //         },
    //     },
    // });
    // print_response(response);

    // let response = api_client.request(&dns::DeleteDnsRecord {
    //     zone_identifier: zone_identifier.as_str(),
    //     identifier: "31102281e86b68cef38728308a5a1113",
    // });
    // print_response(response);

    Ok(())
}

#[derive(Serialize, Deserialize, Debug)]
struct Foo {
    data: String,
}

// fn verify_token(token: String) {}

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
