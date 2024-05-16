//! A simple program to be proven inside the zkVM.

#![no_main]
sp1_zkvm::entrypoint!(main);
use celestia_types::ExtendedHeader;


pub fn main() {
    // NOTE: values of n larger than 186 will overflow the u128 type,
    // resulting in output that doesn't match fibonacci sequence.
    // However, the resulting proof will still be valid!

    println!("reading h1");
    let h1: ExtendedHeader = sp1_zkvm::io::read();
    println!("reading h2");
    let h2: ExtendedHeader = sp1_zkvm::io::read();
    let result = h1.verify(&h2);

    println!("writing result");
    sp1_zkvm::io::commit(&result.is_ok());

}
