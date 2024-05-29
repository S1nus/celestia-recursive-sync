use sp1_sdk::SP1CompressedProof;

fn main() {
    let genesis_proof_file = std::fs::File::open("proof0.json").unwrap(); 
    let genesis_proof: SP1CompressedProof = serde_json::from_reader(genesis_proof_file).unwrap();
    let pubs: Vec<u8> = serde_cbor::to_vec(&genesis_proof.public_values).unwrap();
    println!("{:?}", pubs);
}
