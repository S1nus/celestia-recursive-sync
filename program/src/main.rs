//! A simple program to be proven inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);
use sha2::{Sha256, Digest};
mod buffer;
use buffer::Buffer;
use core::time::Duration;
use tendermint_light_client_verifier::{
    options::Options, types::LightBlock, ProdVerifier, Verdict, Verifier,
};

pub fn main() {
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    // However, the resulting proof will still be valid!

    let vkey: [u32; 8] = sp1_zkvm::io::read();
    let byte_slice: &[u8] = unsafe {
        core::slice::from_raw_parts(vkey.as_ptr() as *const u8, vkey.len() * core::mem::size_of::<u32>())
    };
    let hash_of_vkey = Sha256::digest(byte_slice);
    sp1_zkvm::io::commit(&hash_of_vkey.to_vec());

    let public_values: Vec<u8> = sp1_zkvm::io::read();
    let mut public_values_buffer = Buffer::from(&public_values);
    let public_values_digest = Sha256::digest(&public_values);

    let genesis_hash = sp1_zkvm::io::read_vec();
    sp1_zkvm::io::commit(&genesis_hash);

    let h1_bytes = sp1_zkvm::io::read_vec();
    let h2_bytes = sp1_zkvm::io::read_vec();
    let h1: Option<LightBlock> = serde_cbor::from_slice(&h1_bytes).expect("couldn't deserialize h1");
    let h2: LightBlock = serde_cbor::from_slice(&h2_bytes).expect("couldn't deserialize h2");
    // commit h2 hash
    sp1_zkvm::io::commit(&h2.signed_header.header().hash().as_bytes().to_vec());

    match h1 {
        Some(h1) => {

            // Ensure that we are verifying a proof of the same circuit as ourself
            let last_vkey_hash: Vec<u8> = public_values_buffer.read();
            if last_vkey_hash != hash_of_vkey.to_vec() {
                panic!("not valid!");
            }
            // Ensure that the previous proof has the same genesis hash as the current proof
            let last_genesis_hash: Vec<u8> = public_values_buffer.read();
            if last_genesis_hash != genesis_hash {
                panic!("not valid!");
            }
            // Ensure that previous proof has the h2 hash as the current h1 hash
            let last_h2_hash: Vec<u8> = public_values_buffer.read();
            if last_h2_hash != h1.signed_header.header().hash().as_bytes() {
                panic!("not valid!");
            }
            // Ensure that previous proof is valid
            let last_result: bool = public_values_buffer.read();
            if !last_result {
                panic!("not valid!");
            }

            // Verify the previous recursion layer
            sp1_zkvm::lib::verify::verify_sp1_proof(&vkey, &public_values_digest.into());

            // Perform Tendermint (Celestia consensus) verification
            let vp = ProdVerifier::default();
            let opt = Options {
                trust_threshold: Default::default(),
                // 2 week trusting period.
                trusting_period: Duration::from_secs(14 * 24 * 60 * 60),
                clock_drift: Default::default(),
            };
            let verify_time = h2.time() + Duration::from_secs(20);
            let verdict = vp.verify_update_header(
                h2.as_untrusted_state(),
                h1.as_trusted_state(),
                &opt,
                verify_time.unwrap(),
            );
            match verdict {
                Verdict::Success => {
                    sp1_zkvm::io::commit(&true);
                },
                _ => {
                    panic!("verification failed");
                }
            }

        },
        None => {
            if h2.signed_header.header().hash().as_bytes() == genesis_hash {
                sp1_zkvm::io::commit(&true);
            } else {
                panic!("expected h2 == genesis hash");
            }
        }
    }

}