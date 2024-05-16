//! A simple script to generate and verify the proof of a given program.
use serde_json;
use celestia_types::ExtendedHeader;
use sp1_sdk::{ProverClient, SP1Stdin};

const ELF: &[u8] = include_bytes!("../../program/elf/riscv32im-succinct-zkvm-elf");

fn main() {
    // Generate proof.
    let mut stdin = SP1Stdin::new();
    let header1_file = std::fs::File::open("zkgenesis2.json").unwrap();
    let header2_file = std::fs::File::open("nethead2.json").unwrap();
    let h1: ExtendedHeader = serde_json::from_reader(header1_file).unwrap();
    let h2: ExtendedHeader = serde_json::from_reader(header2_file).unwrap();
    
    // write the headers
    stdin.write(&h1);
    stdin.write(&h2);

    let client = ProverClient::new();
    let mut public_values = client.execute(&ELF, stdin).unwrap();
    let result: bool = public_values.read();
    println!("result is: {}", result);
}
