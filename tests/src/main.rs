use sp1_sdk::SP1CompressedProof;
mod buffer;
use buffer::Buffer;

fn main() {
    let genesis_proof_file = std::fs::File::open("proof0.json").unwrap(); 
    let genesis_proof: SP1CompressedProof = serde_json::from_reader(genesis_proof_file).unwrap();
    let pubs_vec = genesis_proof.public_values.to_vec();
    println!("pubs_vec: {:?}", pubs_vec);
    let mut buffer = Buffer::from(&pubs_vec);
    println!("buffer: {:?}", buffer);
    let first_vec: Vec<u8> = buffer.read();
    println!("first_vec: {:?}", first_vec);
}
