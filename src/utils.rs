use cloudflare::framework::response::{ApiFailure, ApiResponse, ApiResult};
use serde::Serialize;

#[allow(dead_code)]
pub fn print_response_json<T: ApiResult>(response: &ApiResponse<T>)
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

pub fn print_response<T: ApiResult>(response: ApiResponse<T>) {
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
