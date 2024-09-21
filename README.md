# Cryptographic Sync Project

This project implements a cryptographic synchronization system using [SP1](https://github.com/succinctlabs/sp1), a zero-knowledge virtual machine for RISC-V programs.

Reimplementation of [celestia-recursive-sync](https://github.com/S1nus/celestia-recursive-sync/tree/main) using SP1.

## Requirements

- [Rust](https://rustup.rs/)
- [SP1](https://docs.succinct.xyz/getting-started/install.html)

## Project Structure

The project consists of two main components:

1. `program`: Contains the RISC-V program that performs the cryptographic operations.
2. `script`: Contains the Rust script to build, execute, and generate proofs for the program.

## Running the Project

```
cd program
cargo build
cd ../script
cargo run
```