//! A simple script to generate and verify the proof of a given program.
use serde_json;
use celestia_types::ExtendedHeader;
use sp1_sdk::{ProverClient, SP1Stdin};
use std::time::Instant;

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Generate proof.
    let mut stdin = SP1Stdin::new();
    let header1_file = std::fs::File::open("zkgenesis2.json").unwrap();
    let header2_file = std::fs::File::open("nethead2.json").unwrap();
    let h1: ExtendedHeader = serde_json::from_reader(header1_file).unwrap();
    let h2: ExtendedHeader = serde_json::from_reader(header2_file).unwrap();
    
    let encoded_1 = serde_cbor::to_vec(&h1).unwrap();
    let encoded_2 = serde_cbor::to_vec(&h2).unwrap();
    
    // write the headers
    stdin.write_vec(encoded_1);
    stdin.write_vec(encoded_2);

    let client = ProverClient::new();
    let (pk, _vk) = client.setup(&ELF);


    let start_time = Instant::now();
    let mut proof = client.prove_compressed(&pk, stdin).expect("couldn't prove");
    let end_time = Instant::now();
    println!("Proving time: {:?}", end_time - start_time);
    //let mut public_values = client.execute(&ELF, stdin).unwrap();
    //let result = proof.public_values.to_vec();
    let result: bool = proof.public_values.read();
    let proof_file = std::fs::File::create("proof.json").unwrap();
    let pk_file = std::fs::File::create("pk.json").unwrap();
    serde_json::to_writer(proof_file, &proof).unwrap();

    println!("result is: {:?}", result);
}
