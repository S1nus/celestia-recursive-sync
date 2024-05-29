//! A simple program to be proven inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);
use celestia_types::ExtendedHeader;
use sha2::{Sha256, Digest};
mod buffer;
use buffer::Buffer;

pub fn main() {
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    // However, the resulting proof will still be valid!

    let vkey: [u32; 8] = sp1_zkvm::io::read();
    let byte_slice: &[u8] = unsafe {
        core::slice::from_raw_parts(vkey.as_ptr() as *const u8, vkey.len() * core::mem::size_of::<u32>())
    };
    let hash_of_vkey = Sha256::digest(&byte_slice);
    sp1_zkvm::io::commit(&hash_of_vkey.to_vec());

    let public_values: Vec<u8> = sp1_zkvm::io::read();
    let mut public_values_buffer = Buffer::from(&public_values);
    println!("{:?}", public_values_buffer);
    let public_values_digest = Sha256::digest(&public_values);
    let zk_genesis_hash = sp1_zkvm::io::read_vec();
    sp1_zkvm::io::commit(&zk_genesis_hash);
    let h1_bytes = sp1_zkvm::io::read_vec();
    let h2_bytes = sp1_zkvm::io::read_vec();
    println!("getting header1");
    let h1: Option<ExtendedHeader> = serde_cbor::from_slice(&h1_bytes).expect("couldn't deserialize h1");
    println!("getting header2");
    let h2: ExtendedHeader = serde_cbor::from_slice(&h2_bytes).expect("couldn't deserialize h2");
    // commit h2 hash
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
            println!("going to verify proof");
            sp1_zkvm::precompiles::verify::verify_sp1_proof(&vkey, &public_values_digest.into());

            // Perform Tendermint (Celestia consensus) verification
            println!("performing header verification");
            if h1.verify(&h2).is_ok() {
                sp1_zkvm::io::commit(&true);
            } else {
                sp1_zkvm::io::commit(&false);
            }

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