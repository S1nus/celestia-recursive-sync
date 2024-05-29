//! A simple script to generate and verify the proof of a given program.
use serde_json;
use celestia_types::ExtendedHeader;
use sp1_sdk::{HashableKey, ProverClient, SP1Stdin, SP1CompressedProof};
use std::{io::Write, time::Instant};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {

    // Initialize ProverClient
    let client = ProverClient::new();
    let (pk, vk) = client.setup(ELF);
    let mut stdin = SP1Stdin::new();

    let zk_genesis_file = std::fs::File::open("zkgenesis2.json").unwrap();
    let header1_file = std::fs::File::open("nethead2.json").unwrap();
    let header2_file = std::fs::File::open("skipheader.json").unwrap();
    let zk_genesis: ExtendedHeader = serde_json::from_reader(zk_genesis_file).unwrap();
    let nil_h1: Option<ExtendedHeader> = None;
    let h1: Option<ExtendedHeader> = serde_json::from_reader(header1_file).unwrap();
    let h2: ExtendedHeader = serde_json::from_reader(header2_file).unwrap();

    let genesis_proof_file = std::fs::File::open("proof0.json").unwrap(); 
    let genesis_proof: SP1CompressedProof = serde_json::from_reader(genesis_proof_file).unwrap();
    
    // Create first proof
    let encoded_1 = serde_cbor::to_vec(&zk_genesis).unwrap();
    let encoded_2 = serde_cbor::to_vec(&h1).unwrap();
    
    // write verification key
    stdin.write(&vk.hash_u32());
    // write the "public values" 
    // (they're just some junk in the first iteration of the IVC)
    println!("the vec: {:?}", &genesis_proof.public_values.to_vec());
    stdin.write(&genesis_proof.public_values.to_vec());
    // write genesis hash
    stdin.write_vec(zk_genesis.header.hash().as_bytes().to_vec());
    // write header1 (zk_genesis.json)
    stdin.write_vec(encoded_1);
    // write header2 (nethead2.json)
    stdin.write_vec(encoded_2);
    // write the proof
    stdin.write_proof(genesis_proof.proof, vk.vk);

    //let mut public_values = client.execute(ELF, stdin).expect("could not execute");
    let start_time = Instant::now();
    let proof = client.prove_compressed(&pk, stdin).expect("could not prove");
    let end_time = Instant::now();
    println!("Proof generation time: {:?}", end_time.duration_since(start_time));
    let mut first_proof_file = std::fs::File::create("proof1.json").unwrap();
    first_proof_file.write_all(serde_json::to_string(&proof).unwrap().as_bytes()).unwrap();

    let mut public_values = proof.public_values;
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

#[cfg(test)]
mod tests {
    use sp1_sdk::SP1CompressedProof;

    #[test]
    fn test_deserialize_public_values() {
        println!("running test_deserialize_public_values");
        let genesis_proof_file = std::fs::File::open("proof0.json").unwrap(); 
        let genesis_proof: SP1CompressedProof = serde_json::from_reader(genesis_proof_file).unwrap();
        let pubs: Vec<u8> = serde_cbor::to_vec(&genesis_proof.public_values).unwrap();
    }
}