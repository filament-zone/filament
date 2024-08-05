#!/bin/bash

rm ~/.sov_cli_wallet/wallet_state.json

./target/release/filament-hub-cli rpc set-url http://127.0.0.1:12345
./target/release/filament-hub-cli keys import --path test-data/keys/token_deployer_private_key.json
./target/release/filament-hub-cli keys activate by-address $(jq -r '.gas_token_config.address_and_balances[0][0]' <test-data/genesis/demo/mock/bank.json)
# ./target/release/filament-hub-cli transactions import from-string core --json '{"CreateCampaign":{"origin":"neutron-1","origin_id":1,"indexer":"sov13xmcgktd6jz0t53qjrd3jr6kexgam2s628d380nj75erk37xwklsupmn0u","attester":"sov13xmcgktd6jz0t53qjrd3jr6kexgam2s628d380nj75erk37xwklsupmn0u"}}' --chain-id 1
# ./target/release/filament-hub-cli transactions import from-string core --json '{"RegisterIndexer":["sov13xmcgktd6jz0t53qjrd3jr6kexgam2s628d380nj75erk37xwklsupmn0u","ehlo"]}' --chain-id 1
./target/release/filament-hub-cli transactions import from-string core --json '{"RegisterIndexer":["sov15vspj48hpttzyvxu8kzq5klhvaczcpyxn6z6k0hwpwtzs4a6wkvqwr57gc","ehlo"]}' --chain-id 0 --max-fee 3000

