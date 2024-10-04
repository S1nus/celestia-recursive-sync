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


    // Compute genesis proof
    let prover_client = ProverClient::new();
    let (pk, vk) = prover_client.setup(ELF);
    let mut stdin = SP1Stdin::new();
    stdin.write(&vk.hash_u32());
    let nul_vec: Vec<u8> = vec![];
    stdin.write(&nul_vec);
    stdin.write_vec(genesis.signed_header.header().hash().as_bytes().to_vec());
    let null_head: Option<LightBlock> = None;
    let encoded1 = serde_cbor::to_vec(&null_head).expect("Failed to cbor encode null_head");
    stdin.write_vec(encoded1);
    let encoded2 = serde_cbor::to_vec(&Some(genesis.clone())).expect("Failed to cbor encode genesis");
    println!("expect this {:?}", &genesis.signed_header.header().hash().as_bytes().to_vec());
    stdin.write_vec(encoded2);
    //stdin.write_proof(latest_proof_inner, vk.vk);

    println!("creating genesis proof");
    let genesis_proof = prover_client.prove(&pk, stdin).compressed().run().expect("could not prove");
    std::fs::write("tm_genesis_proof.json", serde_json::to_string(&genesis_proof).expect("could not json serialize")).expect("could not write");


    let genesis_proof_public_values = genesis_proof.public_values.to_vec();
    let genesis_proof_inner = *match genesis_proof.proof.clone() {
        SP1Proof::Compressed(c) => c,
        _ => panic!("Not the right kind of SP1 proof")
    };

    let prover_client = ProverClient::new();
    let (pk, vk) = prover_client.setup(ELF);
    let mut stdin = SP1Stdin::new();
    stdin.write(&vk.hash_u32());
    println!("public values... {:?}", &genesis_proof_public_values);
    stdin.write(&genesis_proof_public_values);
    stdin.write_vec(genesis.signed_header.header().hash().as_bytes().to_vec());
    println!("to be equal to this {:?}", &genesis.signed_header.header().hash().as_bytes().to_vec());
    let encoded1 = serde_cbor::to_vec(&Some(genesis.clone())).expect("Failed to cbor encode genesis_head");
    stdin.write_vec(encoded1);
    println!("trying to open {}", format!("needed_headers/{}",&files[0]));
    let next_head_file = std::fs::File::open(format!("needed_headers/{}.json",&files[0])).expect("Could not open");
    let mut running_head: Option<LightBlock> = Some(serde_json::from_reader(next_head_file).expect("could not parse"));
    let encoded2 = serde_cbor::to_vec(&running_head).expect("Failed to cbor encode genesis");
    stdin.write_vec(encoded2);
    stdin.write_proof(genesis_proof_inner, vk.vk);
    //stdin.write_proof(latest_proof_inner, vk.vk);

    println!("creating proof for {}", files[0]);
    let mut running_proof = prover_client.prove(&pk, stdin).compressed().run().expect("could not prove");
    std::fs::write(format!("{}_proof.json", files[0]), serde_json::to_string(&running_proof).expect("could not json serialize")).expect("could not write");

    for i in 1..files.len() {
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


    /*let dummy_proof_file = std::fs::File::open("dummy_proof.json").expect("could not open newest_proof.json");
    let dummy_proof: SP1ProofWithPublicValues = serde_json::from_reader(dummy_proof_file).expect("could not deserialize dummy proof");
    let proof_public_values = dummy_proof.public_values.to_vec();
    let latest_proof_inner = *match dummy_proof.proof.clone() {
        SP1Proof::Compressed(c) => c,
        _ => panic!("Not the right kind of SP1 proof")
    };
    let prover_client = ProverClient::new();
    let (pk, vk) = prover_client.setup(ELF);
    let mut stdin = SP1Stdin::new();
    stdin.write(&vk.hash_u32());
    stdin.write(&proof_public_values);
    stdin.write_vec(genesis.signed_header.header().hash().as_bytes().to_vec());
    let nul_head: Option<LightBlock> = None;
    let encoded1 = serde_cbor::to_vec(&nul_head).expect("Failed to cbor encode nul_head");
    stdin.write_vec(encoded1);
    let encoded2 = serde_cbor::to_vec(&genesis).expect("Failed to cbor encode genesis");
    stdin.write_vec(encoded2);
    //stdin.write_proof(latest_proof_inner, vk.vk);

    let resultant_proof = prover_client.prove(&pk, stdin).compressed().run().expect("could not prove");
    std::fs::write("tm_genesis_proof.json", serde_json::to_string(&resultant_proof).expect("could not json serialize")).expect("could not write");
    println!("done");*/

    /*let vp = ProdVerifier::default();
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