# Uke Account Filter ink! Smart Contract

## Description

An ink! smart  contract used for defining rules related to accounts that message using the `uke` protocol. 

Users can define rules for whether they wish to be contacted or not, and who can contact them. They essentially can create whitelists to explicitly allow who is permitted to message that specific account, along with what data can be sent in the future.

This measure prevents a common issue with phone numbers, email, and even other apps - spam.  This contract ensures the rules are kept in place, and the user is safe from any malicious or unwanted messages.

## Requirements

- Rust & Cargo
- ink! CLI

For an extended guide, please view: https://ink.substrate.io/getting-started/setup.  This is required before running, compiling, or running tests for this repository.


## Building & Running Tests

To run the included unit tests, you can run

```sh
cargo +nightly contract test
```

To build the smart contract into a usable WASM executable, you can run

```sh
cargo +nightly contract build
```

## Deploying to Substrate Contract UI

Firstly, install the `substrate-contract-node` to your commandline using `cargo` :

```sh
cargo install contracts-node --git https://github.com/paritytech/substrate-contracts-node.git --tag v0.17.0 --force --locked
```

Build the contract:

```sh
cargo +nightly contract build
```

Start your Substrate development node: 
```sh
substrate-contracts-node --dev --tmp
```

Once it's started and you see blocks populating, navigate to https://contracts-ui.substrate.io/ and click the upper left and select `Local Node`.

You may now upload `uke_account_filter.contract` to the node via `Add New Contract` in the left pane.  Click through the UI and upload the contract to the network.


