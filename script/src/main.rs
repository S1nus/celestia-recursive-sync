//! A simple script to generate and verify the proof of a given program.
use serde_json;
use celestia_types::ExtendedHeader;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin, SP1CompressedProof};
use std::{io::Write, time::Instant};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {

    // Initialize ProverClient
    let client = ProverClient::new();
    let (_pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    let proof_file = std::fs::File::open("proof.json").unwrap();
    let proof: SP1CompressedProof = serde_json::from_reader(proof_file).unwrap();
    let zk_genesis_file = std::fs::File::open("zkgenesis2.json").unwrap();
    let header1_file = std::fs::File::open("nethead2.json").unwrap();
    let header2_file = std::fs::File::open("skipheader.json").unwrap();
    let zk_genesis: ExtendedHeader = serde_json::from_reader(zk_genesis_file).unwrap();
    let nil_h1: Option<ExtendedHeader> = None;
    let h1: Option<ExtendedHeader> = serde_json::from_reader(header1_file).unwrap();
    let h2: ExtendedHeader = serde_json::from_reader(header2_file).unwrap();
    
    // Create first proof
    let encoded_1 = serde_cbor::to_vec(&nil_h1).unwrap();
    let encoded_2 = serde_cbor::to_vec(&zk_genesis).unwrap();
    
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
    stdin.write_proof(proof.proof, vk.vk);

    let mut public_values = client.execute(ELF, stdin).expect("could not execute");
    println!("public values: {:?}", public_values);
    let vkey_out: Vec<u8> = public_values.read();
    let zk_genesis_hash: Vec<u8> = public_values.read();
    let h2_hash_out: Vec<u8> = public_values.read();
    let result: bool = public_values.read();
    println!("vkey: {:?}", vkey_out);
    println!("zk_genesis_hash: {:?}", zk_genesis_hash);
    println!("h2_hash: {:?}", h2_hash_out);
    println!("result: {:?}", result);
}