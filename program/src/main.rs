#![no_main]
sp1_zkvm::entrypoint!(main);

use sha2::{Sha256, Digest};
use celestia_types::ExtendedHeader;
mod buffer;
use buffer::Buffer;

pub fn main() {
    let vkey: [u32; 8] = sp1_zkvm::io::read();
    let byte_slice: Vec<u8> = vkey.iter()
        .flat_map(|&x| x.to_le_bytes().to_vec())
        .collect();
    let hash_of_vkey = Sha256::digest(&byte_slice);
    sp1_zkvm::io::commit(&hash_of_vkey.to_vec());

    let public_values: Vec<u8> = sp1_zkvm::io::read();
    let mut public_values_buffer = Buffer::from(&public_values);
    let public_values_digest = Sha256::digest(&public_values);
    let zk_genesis_hash = sp1_zkvm::io::read_vec();
    sp1_zkvm::io::commit(&zk_genesis_hash);

    let h1_bytes = sp1_zkvm::io::read_vec();
    let h2_bytes = sp1_zkvm::io::read_vec();
    let h1: Option<ExtendedHeader> = serde_cbor::from_slice(&h1_bytes).expect("couldn't deserialize h1");
    let h2: ExtendedHeader = serde_cbor::from_slice(&h2_bytes).expect("couldn't deserialize h2");
    sp1_zkvm::io::commit(&h2.header.hash().as_bytes().to_vec());

    match h1 {
        Some(h1) => {
            println!("it's some");
            // Ensure that we are verifying a proof of the same circuit as ourself
            let last_vkey_hash: Vec<u8> = public_values_buffer.read();
            if &last_vkey_hash != &hash_of_vkey.to_vec() {
                println!("expected vkeys to match: {:?} != {:?}", last_vkey_hash, hash_of_vkey.to_vec());
                sp1_zkvm::io::commit(&false);
                return;
            }

            // Ensure that the previous proof has the same genesis hash as the current proof
            let last_genesis_hash: Vec<u8> = public_values_buffer.read();
            if last_genesis_hash != zk_genesis_hash {
                println!("expected genesis hashes to match: {:?} != {:?}", last_genesis_hash, zk_genesis_hash);
                sp1_zkvm::io::commit(&false);
                return;
            }
            // Ensure that previous proof has the h2 hash as the current h1 hash
            let last_h2_hash: Vec<u8> = public_values_buffer.read();
            if last_h2_hash != h1.header.hash().as_bytes() {
                println!("expected previous h2 hash to match new h1: {:?} != {:?}", last_h2_hash, h1.header.hash().as_bytes());
                sp1_zkvm::io::commit(&false);
                return;
            }
            // Ensure that previous proof is valid
            let last_result: bool = public_values_buffer.read();
            if !last_result {
                println!("expected last proof to be valid: {:?}", last_result);
                sp1_zkvm::io::commit(&false);
                return;
            }

            // Verify the previous recursion layer
            // Note that to verify an SP1 proof inside SP1, you must generate a "compressed" SP1 proof (see Proof Types for more details).
            // https://github.com/succinctlabs/sp1/blob/dev/book/writing-programs/proof-aggregation.md
            println!("going to verify proof");
            // sp1_zkvm::lib::verify::verify_proof() can also be used here.
            sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest.into());

            // Perform Tendermint (Celestia consensus) verification
            println!("performing header verification");
            if h1.verify(&h2).is_ok() {
                sp1_zkvm::io::commit(&true);
            } else {
                sp1_zkvm::io::commit(&false);
            }
            println!("done with header verification");
        },
        None => {
            println!("it's none.");
            if h2.header.hash().as_bytes() == zk_genesis_hash {
                sp1_zkvm::io::commit(&true);
            } else {
                sp1_zkvm::io::commit(&false);
            }
        }
    }
}