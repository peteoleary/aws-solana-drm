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
    println
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

#[post("/authorize", data = "<authorize_body>")]
fn authorize(authorize_body: Json<AuthorizeBody>) -> OkResponse {  

    let cluster_url = "http://localhost:8899".to_string();

    let rpc = RpcClient::new_with_commitment(cluster_url, CommitmentConfig::confirmed());

     // access contract on blockchain

    println!("authorize_body.asset_contract_location={}", authorize_body.asset_contract_location);

    let raw_account_data_result = rpc.get_account_data(&authorize_body.asset_contract_location);

    let account_data: Result<LicenseAccount, _> = bincode::deserialize(&raw_account_data_result.unwrap());

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
    };

    use super::rocket;
    use rocket::http::Status;
    use rocket::local::blocking::Client;

    use crate::{AuthorizeBody};

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

        let program_id = read_keypair_file(get_key_file_path("target/deploy/license-keypair.json"))
            .expect("Cannot read program_id file");

        assert_eq!("Cb5q9Kd6P7xHtg6dJecEqJmHqXtGRQ25TLkziwSx3AhE", program_id.pubkey().to_string());

        // TODO: read key files
        // make contract location

        let licensee = read_keypair_file(get_key_file_path("js/keys/licensee.json"))
            .expect("Cannot read licesee file");

        let nft_address = read_keypair_file(get_key_file_path("js/keys/nft_account.json"))
            .expect("Cannot read nft address file");

        println!("nft_address.pubkey={}", nft_address.pubkey().to_string());

        assert_eq!("CSDntK83NvpqTyndxmyi85QPeSxFuf5muzeuCafnyaUG", nft_address.pubkey().to_string());

        let contract_address = Pubkey::create_with_seed(&nft_address.pubkey(), "license", &program_id.pubkey())
            .expect("Cannot get contract address");

        assert_eq!("Dnh4fDeYTGYDTwKDKAJ4etAdZbh7vEz1M47RbgCByfGy", contract_address.to_string());


        let body = AuthorizeBody {
            asset_requested: "http://www.foo.bar".to_string(),
            asset_contract_location: contract_address,
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