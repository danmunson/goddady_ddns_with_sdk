use std::{
    env,
    fs::read_to_string,
    io::Read,
    time::SystemTime
};
use serde::Deserialize;
use serde_json;
use godaddy_api::{
    blocking::Client,
    request_types::{
        RecordGetRequest,
        RecordReplaceTypeNameRequest
    },
    schemas::{
        GetV1DomainsDomainRecordsTypeNameTypeEnum,
        PutV1DomainsDomainRecordsTypeNameTypeEnum,
        DnsRecordCreateTypeName
    }
};
use reqwest::blocking::get;

const IP_API: &str = "https://api.ipify.org";
const EXPECTED_UPDATE_ERR: &str = "Failed deserializing into json \"serde_json::Value\": \
Error(\"EOF while parsing a value\", line: 1, column: 0) ";

#[derive(Debug, Deserialize)]
struct Config {
    api_url: String,
    domain: String,
    key: String,
    secret: String
}

impl Config {
    pub fn auth(&self) -> String {
        format!("sso-key {}:{}", &self.key, &self.secret)
    }
}

fn load_client(config: &Config) -> Client {
    let mut client = Client::default()
        .with_api_key_auth(&config.auth());
    client.base_url = config.api_url.clone();
    client
}

fn get_my_ip() -> String {
    let mut res = get(IP_API).expect("Couldn't make request to IP API");
    let mut body = String::new();
    res.read_to_string(&mut body).expect("Couldn't read IP API response");
    body
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = &args[1];
    let verbose = match args.get(2) {
        Some(arg) => arg == "--verbose",
        None => false
    };
    let config: Config = serde_json::from_str(
        &read_to_string(config_path).expect("Failed to load config"))
        .expect("Couldn't parse config string as JSON");
    let client = load_client(&config);
    
    let my_ip = get_my_ip();

    let result = client.record_get(RecordGetRequest {
        domain: config.domain.clone(),
        type_path: GetV1DomainsDomainRecordsTypeNameTypeEnum::A,
        name: "@".to_string(),
        ..Default::default()
    });
    let records = result.expect("Couldn't get A records from GoDaddy");

    if records.len() > 1 {
        panic!("Multiple A records with name @");
    } else if records.len() == 0 {
        panic!("Missing A record with name @");
    }

    let record_option = records.get(0);
    let record_ip = match record_option {
        Some(record) => &record.data,
        None => ""
    };

    if my_ip != record_ip  {
        println!("Attempting to update DNS record");
        let response = client
            .record_replace_type_name(RecordReplaceTypeNameRequest {
                domain: config.domain.clone(),
                type_path: PutV1DomainsDomainRecordsTypeNameTypeEnum::A,
                name: "@".to_string(),
                data: vec![
                    DnsRecordCreateTypeName { data : my_ip, ..Default::default() }
                ],
            });
        // go daddy API returns empty body on success
        let result = match response {
            Err(err) => {
                let err_str = format!("{}", err);
                if err_str == EXPECTED_UPDATE_ERR {
                    Ok(())
                } else {
                    println!("***{}***", err_str);
                    Err(())
                }
            },
            _ => Ok(())
        };
        let _ = result.expect("Couldn't make update request");
        println!("Updated DNS record at {:?}", SystemTime::now());
    } else if verbose {
        println!("DNS record unchanged");
    }
}
