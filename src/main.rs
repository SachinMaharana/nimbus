use anyhow::{bail, Result};
use dns::{DnsContent, DnsRecord, ListDnsRecords, ListDnsRecordsParams};
use std::{any::type_name, env};
use zone::{ListZonesParams, ZoneDetails};

use cloudflare::{
    endpoints::{account, dns, user, zone},
    framework::{
        apiclient::ApiClient,
        auth::Credentials,
        response::{ApiFailure, ApiResponse, ApiResult},
        Environment, HttpApiClient, HttpApiClientConfig, OrderDirection,
    },
};

use std::net::Ipv4Addr;

use serde::Serialize;

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

    // let response = api_client.request(&account::ListAccounts { params: None });
    // print_response_json(response);

    // let response = api_client.request(&user::GetUserDetails {});
    // print_response_json(response);

    // let response = api_client.request(&user::GetUserTokenStatus {});
    // print_response_json(response);

    // let response = api_client.request(&zone::ListZones {
    //     params: ListZonesParams {
    //         name: Some("sachinmaharana.com".into()),
    //         ..Default::default()
    //     },
    // });

    // let response = api_client.request(&zone::ZoneDetails {
    //     identifier: "013629251ecc87b23edc9532e02ef4ba",
    // });
    // print_response(response);

    // let response = api_client.request(&dns::ListDnsRecords {
    //     zone_identifier: "013629251ecc87b23edc9532e02ef4ba",
    //     params: dns::ListDnsRecordsParams {
    //         direction: Some(OrderDirection::Ascending),
    //         ..Default::default()
    //     },
    // });
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

    let response = api_client.request(&dns::DeleteDnsRecord {
        zone_identifier: "013629251ecc87b23edc9532e02ef4ba",
        identifier: "31102281e86b68cef38728308a5a1113",
    });
    print_response(response);

    Ok(())
}

// fn verify_token(token: String) {}

fn print_response_json<T: ApiResult>(response: ApiResponse<T>)
where
    T: Serialize,
{
    match response {
        Ok(success) => println!("{}", serde_json::to_string(&success.result).unwrap()),
        Err(e) => match e {
            ApiFailure::Error(status, errors) => {
                println!("HTTP {}", status);
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
