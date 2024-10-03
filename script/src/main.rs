use core::time::Duration;
use serde_json;
use std::{fs, path::Path};
use tendermint_light_client_verifier::{
    options::Options, types::LightBlock, ProdVerifier, Verdict, Verifier,
};
mod tm_rpc_utils;
mod tm_rpc_types;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin, SP1CompressedProof};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("creating rpc client");
    let client = tm_rpc_utils::TendermintRPCClient::default();
    let peer_id = client.fetch_peer_id().await.unwrap();
    println!("getting genesis...");
    let genesis = client.fetch_light_block(1, peer_id).await.unwrap();

    let dir = fs::read_dir("needed_headers")?;
    let mut files = vec![];
    for entry in dir {
        let entry = entry?;
        let path = entry.path();
        if path.is_file() && path.extension().unwrap_or_default() == "json" {
            let filename = path
                .file_stem()
                .unwrap_or_default()
                .to_str()
                .unwrap_or_default();
            files.push(filename.to_string());
        }
    }
    files.sort_by(|a, b| a.parse::<u32>().unwrap().cmp(&b.parse::<u32>().unwrap()));



    /* 
    // deserialize first file
    let file = fs::File::open(format!("needed_headers/{}.json", files[0])).unwrap();
    let reader = std::io::BufReader::new(file);
    let first_block: LightBlock = serde_json::from_reader(reader).unwrap();

    let vp = ProdVerifier::default();
    let opt = Options {
        trust_threshold: Default::default(),
        // 2 week trusting period.
        trusting_period: Duration::from_secs(14 * 24 * 60 * 60),
        clock_drift: Default::default(),
    };
    let verify_time = first_block.time() + Duration::from_secs(20);
    let verdict = vp.verify_update_header(
        first_block.as_untrusted_state(),
        genesis.as_trusted_state(),
        &opt,
        verify_time.unwrap(),
    );
    match verdict {
        Verdict::Success => {println!("all good")},
        _ => {println!("failed")}
    }*/

    Ok(())
}