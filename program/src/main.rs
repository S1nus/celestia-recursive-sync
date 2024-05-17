//! A simple program to be proven inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);
use celestia_types::ExtendedHeader;


pub fn main() {
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    // However, the resulting proof will still be valid!

    let h1_bytes = sp1_zkvm::io::read_vec();
    let h2_bytes = sp1_zkvm::io::read_vec();
    println!("getting header1");
    let h1: ExtendedHeader = serde_cbor::from_slice(&h1_bytes).unwrap();
    println!("getting header2");
    let h2: ExtendedHeader = serde_cbor::from_slice(&h2_bytes).unwrap();
    println!("going to verify header");
    let result = h1.verify(&h2);

    println!("writing result");
    sp1_zkvm::io::commit(&result.is_ok());

}
