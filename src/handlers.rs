use anyhow::{Context, Result};
use cloudflare::{
    endpoints::{account, dns, zone},
    framework::{apiclient::ApiClient, HttpApiClient, OrderDirection, SearchMatch},
};
use dialoguer::{theme::ColorfulTheme, Confirm, Input, MultiSelect, Select};
use dns::DnsContent;
use rand::seq::SliceRandom;
use std::net::Ipv4Addr;
use std::{collections::HashMap, str::FromStr};
use zone::{ListZonesParams, Status};

#[derive(Clone)]
pub struct ZoneInfo {
    pub zone_identifier: String,
    pub zone_name: String,
}

pub struct DnsInfo {
    pub dns_names: Vec<String>,
    dns_identifier_hashmap: HashMap<String, String>,
}

pub fn handle_list(api_client: &HttpApiClient, zone_info: ZoneInfo) -> Result<DnsInfo> {
    let dns_list_response = api_client.request(&dns::ListDnsRecords {
        zone_identifier: zone_info.zone_identifier.as_str(),
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

    let dns_info = DnsInfo {
        dns_names: dns_names_list,
        dns_identifier_hashmap,
    };

    Ok(dns_info)
}

pub fn handle_create(api_client: &HttpApiClient, zone_info: ZoneInfo) -> Result<()> {
    let ZoneInfo {
        zone_identifier,
        zone_name,
    } = zone_info;

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

    let record = format!("{}{}{}", record, ".", zone_name);

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

pub fn handle_delete(api_client: &HttpApiClient, zone_info: ZoneInfo) -> Result<()> {
    let DnsInfo {
        dns_names: dns_names_list,
        dns_identifier_hashmap,
    } = handle_list(&api_client, zone_info.clone())?;

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
                    zone_identifier: zone_info.zone_identifier.as_str(),
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

pub fn handle_account_zone(api_client: &HttpApiClient) -> Result<ZoneInfo> {
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

    let zone_info = ZoneInfo {
        zone_identifier: zone_identifier.to_owned(),
        zone_name: zone_name_selected.to_owned(),
    };
    Ok(zone_info)
}
