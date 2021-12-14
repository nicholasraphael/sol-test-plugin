This is a small attempt and test implementation of creating a plugin for the Solana validator.

Install Solana cli

Build this program.

`cargo build-bpf --manifest-path=./path/to/Cargo.toml --bpf-out-dir=dist/program"`

Install solana-test-validator and set up 

https://docs.solana.com/developing/test-validator

Would need to run by updating the Solana config file. Running `solana config set --config <FILEPATH>`

##### Currently there is a build issue when attempting to build program using build-bpf