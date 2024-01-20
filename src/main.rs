use std::{
    env,
    fs::read_to_string,
    io::Read
};
use serde_derive::Deserialize;
use serde_json;
use godaddy_api::{
    blocking::Client,
    request_types::RecordGetRequest,
    schemas::GetV1DomainsDomainRecordsTypeNameTypeEnum
};
use reqwest::blocking::get;

const IP_API: &str = "https://api.ipify.org";

#[derive(Debug, Deserialize)]
struct Config {
    api_url: String,
    domain: String,
    poll_frequency_seconds: u32,
    key: String,
    secret: String
}

impl Config {
    pub fn auth(&self) -> String {
        let auth_token = format!("sso-key {}:{}", &self.key, &self.secret);
        println!("auth token: {}", auth_token);
        auth_token
    }
}

fn load_client(config: &Config) -> Client {
    let mut client = Client::default()
        .with_api_key_auth(&config.auth());
    client.base_url = config.api_url.clone();
    client
}

fn get_my_ip() -> String {
    let mut res = get(IP_API).expect("Couldn't make request");
    let mut body = String::new();
    res.read_to_string(&mut body).expect("Couldn't read response");
    body
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let config_path = &args[1];
    let config: Config = serde_json::from_str(
        &read_to_string(config_path).expect("Failed to load config"))
        .expect("Couldn't load config");
    let client = load_client(&config);
    
    // test
    let result = client.record_get(RecordGetRequest {
        domain: config.domain.clone(),
        type_path: GetV1DomainsDomainRecordsTypeNameTypeEnum::A,
        name: "@".to_string(),
        limit: None,
        offset: None
    });

    let records = result.expect("Couldn't get result");

    println!("{} A records", records.len());

    let my_ip = get_my_ip();
    
    println!("My IP is {}", my_ip);
}
