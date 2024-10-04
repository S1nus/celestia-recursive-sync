use core::time::Duration;
use serde_json;
use std::{
    io::Write,
    fs, 
    path::Path,
    time::Instant,
};
use tendermint_light_client_verifier::{
    options::Options, types::LightBlock, ProdVerifier, Verdict, Verifier,
};
mod tm_rpc_utils;
mod tm_rpc_types;
use sp1_sdk::{HashableKey, SP1VerifyingKey};
use sp1_sdk::{SP1Proof, SP1ProofWithPublicValues};
use sp1_sdk::{ProverClient, SP1Stdin};

pub const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

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

    let left_off_proof_file = std::fs::File::open("1015226_proof.json").expect("could not open left_off_proof.json");
    let mut running_proof: SP1ProofWithPublicValues = serde_json::from_reader(left_off_proof_file).expect("could not parse");

    let running_header_file = std::fs::File::open("needed_headers/1015226.json").unwrap();
    let mut running_head: Option<LightBlock> = serde_json::from_reader(running_header_file).unwrap();

    // header where i got booted off wifi
    let left_off: String = "1015226".to_string();
    let start = files.iter().position(|r| *r == left_off).unwrap()+1;

    for i in start..files.len() {
        let prover_client = ProverClient::new();
        let (pk, vk) = prover_client.setup(ELF);
        let running_proof_public_values = running_proof.public_values.to_vec();
        let mut stdin = SP1Stdin::new();
        stdin.write(&vk.hash_u32());
        stdin.write(&running_proof_public_values);
        stdin.write_vec(genesis.clone().signed_header.header().hash().as_bytes().to_vec());
        let encoded1 = serde_cbor::to_vec(&running_head).expect("failed to serialzie running head");
        stdin.write_vec(encoded1);
        let next_header_file = std::fs::File::open(format!("needed_headers/{}.json",&files[i])).expect("Could not open");
        let next_header: Option<LightBlock> = Some(serde_json::from_reader(next_header_file).expect("could not parse"));
        let encoded2 = serde_cbor::to_vec(&next_header).expect("coudl not serialize");
        stdin.write_vec(encoded2);
        let running_proof_inner = *match running_proof.proof.clone() {
            SP1Proof::Compressed(c) => c,
            _ => panic!("Not the right kind of SP1 proof")
        };
        stdin.write_proof(running_proof_inner, vk.vk);
        println!("creating proof for {}", files[i]);
        running_proof = prover_client.prove(&pk, stdin).compressed().run().expect("could not prove");
        std::fs::write(format!("{}_proof.json", files[i]), serde_json::to_string(&running_proof).expect("could not json serialize")).expect("could not write");
        running_head = next_header;

    }
    Ok(())

}