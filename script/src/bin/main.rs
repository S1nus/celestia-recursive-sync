use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use celestia_types::ExtendedHeader;
use sp1_sdk::HashableKey;
use sp1_sdk::SP1Proof;

use std::io::Write;
use std::time::Instant;

/// The ELF (executable and linkable format) file for the Succinct RISC-V zkVM.
pub const ELF: &[u8] = include_bytes!("../../../elf/riscv32im-succinct-zkvm-elf");

/// The arguments for the command.
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(long)]
    execute: bool,

    #[clap(long)]
    prove: bool,
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    println!("Loading proof from file");
    let proof_file = std::fs::File::open("proof.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(proof_file).unwrap();
    // Reading public values from the proof directly since the types changed.
    let proof_public_values = proof["proof"]["public_values"].clone();
    // println!("proof_public_values: {:?}", proof_public_values);

    println!("Loading headers");
    let zk_genesis_file = std::fs::File::open("zkgenesis.json").unwrap();
    let header1_file = std::fs::File::open("nethead2.json").unwrap();
    let header2_file = std::fs::File::open("skipheader.json").unwrap();
    // The header we treat as the genesis
    // zk genesis height = 1798987
    let zk_genesis: ExtendedHeader = serde_json::from_reader(zk_genesis_file).unwrap();
    // a "nil" header
    let nil_h1: Option<ExtendedHeader> = None;
    // h1 height = 1798988
    let h1: Option<ExtendedHeader> = serde_json::from_reader(header1_file).unwrap();
    // h2 height = 1798998 (skips by 10 blocks)
    let _h2: ExtendedHeader = serde_json::from_reader(header2_file).unwrap();
    // Serialize the headers for proof0
    // First one is "nil" because there is no previous header for the genesis
    let encoded_1 = serde_cbor::to_vec(&nil_h1).unwrap();
    let encoded_2 = serde_cbor::to_vec(&zk_genesis).unwrap();

    println!("Initializing ProverClient #1");
    // InitializeProof ProverClient
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    // write verification key
    stdin.write(&vk.hash_u32());
    // write the "public values" 
    // (they're just some junk in the first iteration of the IVC)
    stdin.write(&proof_public_values);
    // write genesis hash
    stdin.write_vec(zk_genesis.header.hash().as_bytes().to_vec());
    // write header1 (nil)
    stdin.write_vec(encoded_1);
    // write header2 (genesis)
    stdin.write_vec(encoded_2);
    // write the proof
    // Commented out, because it will crash if the program doesn't branch to call to the precompile
    //stdin.write_proof(proof.proof, vk.vk);

    let start_time = Instant::now();
    // Generate a proof that will be recursively verified / aggregated. Note that we use the "compressed"
    // proof type, which is necessary for aggregation.
    println!("Generating proof #0");
    let proof0 = client.prove(&pk, stdin).compressed().run().expect("could not prove");
    let end_time = Instant::now();
    println!("proof0 generation time: {:?}", end_time.duration_since(start_time));
    println!("Saving proof #0 to file");
    let mut proof0_file = std::fs::File::create("proof0.json").unwrap();
    proof0_file.write_all(serde_json::to_string(&proof0).unwrap().as_bytes()).unwrap();

    let mut public_values = proof0.public_values.clone();
    println!("Reading public values from proof0");
    let _vkey_out: Vec<u8> = public_values.read();
    let zk_genesis_hash: Vec<u8> = public_values.read();
    let h2_hash_out: Vec<u8> = public_values.read();
    let result: bool = public_values.read();
    println!("zk_genesis_hash: {:?}", zk_genesis_hash);
    println!("h2_hash_out: {:?}", h2_hash_out);
    println!("proof0 success: {:?}", result);

    // PROOF 1
    println!("Initializing ProverClient #2");
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    // Serialize the headers for proof1
    println!("Serializing headers for proof1");
    let encoded_1 = serde_cbor::to_vec(&zk_genesis).unwrap();
    let encoded_2 = serde_cbor::to_vec(&h1).unwrap();

    println!("Writing verification key");
    // write verification key
    stdin.write(&vk.hash_u32());
    // write the public values
    stdin.write(&proof0.public_values.to_vec());
    // write genesis hash
    stdin.write_vec(zk_genesis.header.hash().as_bytes().to_vec());
    // write header1 (nil)
    stdin.write_vec(encoded_1);
    // write header2 (genesis)
    stdin.write_vec(encoded_2);
    // write the proof
    let SP1Proof::Compressed(proof0_compressed) = proof0.proof else {
        panic!()
    };
    println!("Writing proof");
    stdin.write_proof(proof0_compressed, vk.vk.clone());

    let start_time = Instant::now();
    println!("Generating proof #1");
    let proof1 = client.prove(&pk, stdin).compressed().run().expect("could not prove");
    let end_time = Instant::now();
    println!("proof1 generation time: {:?}", end_time.duration_since(start_time));
    println!("Saving proof #1 to file");
    let mut proof1_file = std::fs::File::create("proof1.json").unwrap();
    proof1_file.write_all(serde_json::to_string(&proof1).unwrap().as_bytes()).unwrap();

    println!("Reading public values from proof1");
    let mut public_values = proof1.public_values.clone();
    let _vkey_out: Vec<u8> = public_values.read();
    let zk_genesis_hash: Vec<u8> = public_values.read();
    let h2_hash_out: Vec<u8> = public_values.read();
    let result: bool = public_values.read();
    println!("zk_genesis_hash: {:?}", zk_genesis_hash);
    println!("h2_hash_out: {:?}", h2_hash_out);
    println!("proof1 success: {:?}", result);

    println!("Successfully generated proof!");

    // proof.save("proof.json").unwrap();

    // Verify the proof.
    // client.verify(&proof, &vk).expect("failed to verify proof");
    // println!("Successfully verified proof!");
}
