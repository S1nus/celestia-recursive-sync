use clap::Parser;
use sp1_sdk::{ProverClient, SP1Stdin};
use celestia_types::ExtendedHeader;
use sp1_sdk::{HashableKey, SP1VerifyingKey};
use sp1_sdk::{SP1Proof, SP1ProofWithPublicValues};

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

fn write_verification_key(
    stdin: &mut SP1Stdin,
    vk: &SP1VerifyingKey,
    proof: Option<&SP1ProofWithPublicValues>,
    public_values: &Vec<u8>,
    zk_genesis: &ExtendedHeader,
    encoded_1: &[u8],
    encoded_2: &[u8]
) {
    println!("[?] Writing verification key and additional data...");
    stdin.write(&vk.hash_u32());
    stdin.write(public_values);
    stdin.write_vec(zk_genesis.header.hash().as_bytes().to_vec());
    // write header1 (nil for proof0, zk_genesis for proof1)
    stdin.write_vec(encoded_1.to_vec());
    // write header2 (genesis for proof0, h1 for proof1)
    stdin.write_vec(encoded_2.to_vec());
    
    if let Some(proof) = proof {
        if let SP1Proof::Compressed(ref proof_compressed) = proof.proof {
            stdin.write_proof(proof_compressed.clone(), vk.vk.clone());
        } else {
            panic!("Expected compressed proof");
        }
    }
}

fn read_public_values(proof: &SP1ProofWithPublicValues) -> (Vec<u8>, Vec<u8>, bool) {
    println!("[+] Reading public values from proof...");
    let mut public_values = proof.public_values.clone();
    let _vkey_out: Vec<u8> = public_values.read();
    let zk_genesis_hash: Vec<u8> = public_values.read();
    let h2_hash_out: Vec<u8> = public_values.read();
    let result: bool = public_values.read();

    println!("  -> 1st hash: {:?}", zk_genesis_hash);
    println!("  -> 2nd hash: {:?}", h2_hash_out);
    println!("  -> result: {:?}", result);

    (zk_genesis_hash, h2_hash_out, result)
}

fn read_and_encode_header(file_path: &str) -> Vec<u8> {
    println!("[?] Reading and encoding header from {}...", file_path);
    let file = std::fs::File::open(file_path).expect("Failed to open file");
    let header: Option<ExtendedHeader> = serde_json::from_reader(file).expect("Failed to parse JSON");
    serde_cbor::to_vec(&header).expect("Failed to encode header")
}

fn main() {
    // Setup the logger.
    sp1_sdk::utils::setup_logger();

    // Check for required files
    let required_files = [
        ("script/data/proof.json", "Initial proof file"),
        ("script/data/zkgenesis.json", "ZK genesis header file"),
        ("script/data/nethead2.json", "Network header file"),
        ("script/data/skipheader.json", "Skip header file"),
    ];

    for (file_path, description) in required_files.iter() {
        if !std::path::Path::new(file_path).exists() {
            eprintln!("Error: {} not found at {}. Make sure you're running the cargo command from the project root directory.", description, file_path);
            std::process::exit(1);
        }
    }

    // Load initial proof and extract public values.
    println!("[?] Loading proof from file...");
    let proof_file = std::fs::File::open("script/data/proof.json").unwrap();
    let proof: serde_json::Value = serde_json::from_reader(proof_file).unwrap();
    let proof_public_values: Vec<u8> = proof["proof"]["public_values"]
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_u64().unwrap() as u8)
        .collect();

    // Load zk genesis header
    let zk_genesis_file = std::fs::File::open("script/data/zkgenesis.json").unwrap();
    let zk_genesis: ExtendedHeader = serde_json::from_reader(zk_genesis_file).unwrap();

    // Prepare encoded headers for proof0
    let nil_h1: Option<ExtendedHeader> = None;
    let encoded_1 = serde_cbor::to_vec(&nil_h1).unwrap();
    let encoded_2 = read_and_encode_header("script/data/zkgenesis.json");
    println!("[#1] encoded len: {:?} (null genesis header)", encoded_1.len());
    println!("[#1] encoded len: {:?} (first header)", encoded_2.len());

    // Generate proof0
    println!("[?] Generating proof0...");
    // InitializeProof ProverClient
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    write_verification_key(&mut stdin, &vk, None, &proof_public_values, &zk_genesis, &encoded_1, &encoded_2);

    // Generate a proof that will be recursively verified / aggregated. 
    // Note that we use the "compressed()" method here to generate a compressed proof,
    // proof type, which is necessary for aggregation.
    let start_time = Instant::now();
    let proof0 = client.prove(&pk, stdin).compressed().run().expect("could not prove");
    let end_time = Instant::now();
    println!("[+] proof0 generation time: {:?}", end_time.duration_since(start_time));

    println!("[?] Saving proof #0 to file...");
    let mut proof0_file = std::fs::File::create("script/data/proof0.json").unwrap();
    proof0_file.write_all(serde_json::to_string(&proof0).unwrap().as_bytes()).unwrap();

    let (_zk_genesis_hash, _h2_hash_out, _result0) = read_public_values(&proof0);

    // Generate proof1
    println!("[?] Generating proof1...");
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    // Prepare encoded headers for proof1
    let encoded_1 = read_and_encode_header("script/data/zkgenesis.json");
    let encoded_2 = read_and_encode_header("script/data/nethead2.json");
    println!("[#2] encoded len: {:?} (genesis header)", encoded_1.len());
    println!("[#2] encoded len: {:?} (first header)", encoded_2.len());

    write_verification_key(&mut stdin, &vk, Some(&proof0), &proof0.public_values.to_vec(), &zk_genesis, &encoded_1, &encoded_2);

    let start_time = Instant::now();
    let proof1 = client.prove(&pk, stdin).compressed().run().expect("could not prove");
    let end_time = Instant::now();
    println!("[+] proof1 generation time: {:?}", end_time.duration_since(start_time));
    
    println!("[?] Saving proof1.json to file...");
    let mut proof1_file = std::fs::File::create("script/data/proof1.json").unwrap();
    proof1_file.write_all(serde_json::to_string(&proof1).unwrap().as_bytes()).unwrap();

    let (_zk_genesis_hash, _h2_hash_out, _result1) = read_public_values(&proof1);

    println!("\n[+] Success.");
}
