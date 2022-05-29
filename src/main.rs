#[macro_use] extern crate rocket;
extern crate serde;
extern crate serde_json;
extern crate bincode;

use solana_client::rpc_client::RpcClient;
use rocket::http::Status;
use rocket::serde::json::{Json, json};
use rocket::serde::{Serialize, Deserialize};
use solana_sdk::{
    clock::UnixTimestamp,
    commitment_config::CommitmentConfig,
    program_pack::Pack,
    pubkey::Pubkey,
    signature::{read_keypair_file, Keypair, Signer},
    transaction::Transaction,
};
use std::{
    collections::HashMap,
    fs::File,
    io::Write,
    time::{Duration, SystemTime, UNIX_EPOCH},
    println,
    str::FromStr
};


#[get("/")]
fn index() -> &'static str {
    "Hello, AWS!"
}

#[derive(Deserialize, Serialize)]
pub struct LicenseAccount {
    pub status: u8, // 1
    pub licensor_pubkey: Pubkey, // 32
    pub licensee_pubkey: Pubkey,   // 32
    pub asset_pubkey: Pubkey,   // 32
    pub license_amount: u64,   // 8
    pub license_start: UnixTimestamp,    // 8
    pub license_end: UnixTimestamp,
}

#[derive(Responder)]
#[response(status = 200, content_type = "json")]
struct OkResponse(String);

#[derive(Deserialize, Serialize)]
struct AuthorizeBody {
    asset_requested: String,    // URL or NFT or both?
    asset_contract_location: Pubkey,
    licensee: Pubkey,
    request_length: Duration
}

// NFT data structures version 1 from metaplex-program-library/token-metadata/program/src/state.rs

#[derive(Deserialize, Serialize)]
pub struct Creator {
    pub address: Pubkey,
    pub verified: bool,
    // In percentages, NOT basis points ;) Watch out!
    pub share: u8,
}

#[derive(Deserialize, Serialize)]
pub struct Data {
    /// The name of the asset
    pub name: String,
    /// The symbol for the asset
    pub symbol: String,
    /// URI pointing to JSON representing the asset
    pub uri: String,
    /// Royalty basis points that goes to creators in secondary sales (0-10000)
    pub seller_fee_basis_points: u16,
    /// Array of creators, optional
    pub creators: Option<Vec<Creator>>,
}

fn get_license_program_id() -> Pubkey {
    return Pubkey::from_str("Cb5q9Kd6P7xHtg6dJecEqJmHqXtGRQ25TLkziwSx3AhE").unwrap();
}

#[post("/authorize", data = "<authorize_body>")]
fn authorize(authorize_body: Json<AuthorizeBody>) -> OkResponse {  

    let cluster_url = "http://localhost:8899".to_string();

    let rpc = RpcClient::new_with_commitment(cluster_url, CommitmentConfig::confirmed());

     // access contract on blockchain

    println!("authorize_body.asset_contract_location={}", authorize_body.asset_contract_location);

    let raw_account_data_result = rpc.get_account_data(&authorize_body.asset_contract_location);

    let account_data: LicenseAccount = bincode::deserialize(&raw_account_data_result.unwrap()).unwrap();

    println!("account_data.licensor_pubkey={}", account_data.licensor_pubkey);

    let contract_address = Pubkey::create_with_seed(&authorize_body.asset_contract_location, "license", &get_license_program_id())
            .expect("Cannot get contract address");

    assert_eq!("Dnh4fDeYTGYDTwKDKAJ4etAdZbh7vEz1M47RbgCByfGy", contract_address.to_string());

    // make sure assets requested are part of NFT
    // see if requestor has access for requested time period
    // issue one or more signed URLs allowing access

    return OkResponse("Put some stuff here".to_string());
}

#[launch]
fn rocket() -> _ {
    rocket::build().mount("/", routes![index])
        .mount("/", routes![authorize])
}


#[cfg(test)]
mod test {

    use std::{
        env,
        collections::HashMap,
        fs::File,
        io::Write,
        time::{Duration, SystemTime, UNIX_EPOCH},
        str::FromStr
    };

    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    use crate::{AuthorizeBody, get_license_program_id};

    use solana_sdk::{
        clock::UnixTimestamp,
        commitment_config::CommitmentConfig,
        program_pack::Pack,
        pubkey::Pubkey,
        signature::{read_keypair_file, Keypair, Signer},
        transaction::Transaction,
    };

    fn get_rocket_client() -> Client {
        // TODO: see if we can point this client to a remote address to run test suite against server
        Client::tracked(rocket()).unwrap()
    }

    #[test]
    fn test_hello() {
        let client = get_rocket_client();
        let response = client.get("/").dispatch();
        assert_eq!(response.status(), Status::Ok);
        assert_eq!(response.into_string(), Some("Hello, AWS!".into()));
    }

    fn get_key_file_path(relative_path: &str) -> String {
        return format!("../solana-license-program/{}", relative_path)
    }

    #[test]
    fn test_authorize() {
        let client = get_rocket_client();

        let program_id = get_license_program_id();

        assert_eq!("Cb5q9Kd6P7xHtg6dJecEqJmHqXtGRQ25TLkziwSx3AhE", program_id.to_string());

        // TODO: read key files
        // make contract location

        let licensee = read_keypair_file(get_key_file_path("js/keys/licensee.json"))
            .expect("Cannot read licesee file");

        let nft_address_string = std::fs::read_to_string(get_key_file_path("js/keys/devnet_nft_account.pub.txt"))
            .expect("Could not read NFT address file");

        println!("nft_address.pubkey={}", nft_address_string);

        let nft_address = Pubkey::from_str(&nft_address_string).expect("Could not parse NFT address");

        let body = AuthorizeBody {
            asset_requested: "http://www.foo.bar".to_string(),
            asset_contract_location: nft_address,
            licensee: licensee.pubkey(),
            request_length: Duration::new(1000, 0)
        };

        let body_string = serde_json::to_string(&body).unwrap();

        let response = client.post("/authorize")
            .body(&body_string)
            .dispatch();
        assert_eq!(response.status(), Status::Ok);
    }
}