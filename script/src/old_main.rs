use serde_json;
use celestia_types::ExtendedHeader;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin, SP1CompressedProof};
use std::{io::Write, time::Instant};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {

    let proof_file = std::fs::File::open("proof.json").unwrap();
    let proof: SP1CompressedProof = serde_json::from_reader(proof_file).unwrap();
    println!("{:?}", proof.public_values.to_vec());

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
    let h2: ExtendedHeader = serde_json::from_reader(header2_file).unwrap();
    
    // Serialize the headers for proof0
    // First one is "nil" because there is no previous header for the genesis
    let encoded_1 = serde_cbor::to_vec(&nil_h1).unwrap();
    let encoded_2 = serde_cbor::to_vec(&zk_genesis).unwrap();
    
    // InitializeProof ProverClient
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    // write verification key
    stdin.write(&vk.hash_u32());
    // write the "public values" 
    // (they're just some junk in the first iteration of the IVC)
    stdin.write(&proof.public_values.to_vec());
    // write genesis hash
    stdin.write_vec(zk_genesis.header.hash().as_bytes().to_vec());
    // write header1 (nil)
    stdin.write_vec(encoded_1);
    // write header2 (genesis)
    stdin.write_vec(encoded_2);
    // write the proof
    // Commented out, because it will crash if the program doesn't branch to call to the precompile
    //stdin.write_proof(proof.proof, vk.vk);

    //let mut public_values = client.execute(ELF, stdin).expect("could not execute");
    let start_time = Instant::now();
    let proof0 = client.prove_compressed(&pk, stdin).expect("could not prove");
    let end_time = Instant::now();
    println!("proof0 generation time: {:?}", end_time.duration_since(start_time));
    let mut proof0_file = std::fs::File::create("proof0.json").unwrap();
    proof0_file.write_all(serde_json::to_string(&proof0).unwrap().as_bytes()).unwrap();

    let mut public_values = proof0.public_values.clone();
    let vkey_out: Vec<u8> = public_values.read();
    let zk_genesis_hash: Vec<u8> = public_values.read();
    let h2_hash_out: Vec<u8> = public_values.read();
    let result: bool = public_values.read();
    println!("proof0 success: {:?}", result);

    // Now do proof1
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    // Serialize the headers for proof1
    let encoded_1 = serde_cbor::to_vec(&zk_genesis).unwrap();
    let encoded_2 = serde_cbor::to_vec(&h1).unwrap();

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
    stdin.write_proof(proof0.proof, vk.vk);

    let start_time = Instant::now();
    let proof1 = client.prove_compressed(&pk, stdin).expect("could not prove");
    let end_time = Instant::now();
    println!("proof1 generation time: {:?}", end_time.duration_since(start_time));
    let mut proof1_file = std::fs::File::create("proof1.json").unwrap();
    proof1_file.write_all(serde_json::to_string(&proof1).unwrap().as_bytes()).unwrap();

    let mut public_values = proof1.public_values.clone();
    let vkey_out: Vec<u8> = public_values.read();
    let zk_genesis_hash: Vec<u8> = public_values.read();
    let h2_hash_out: Vec<u8> = public_values.read();
    let result: bool = public_values.read();
    println!("proof1 success: {:?}", result);
}
