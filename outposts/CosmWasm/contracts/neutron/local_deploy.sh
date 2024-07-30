#!/bin/bash

function optimize () {
  docker run --rm -v "$1":/code \
    --mount type=volume,source="$(basename "$1")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/optimizer:0.15.0 ./contracts/neutron/
}

# if only --out-dir was stable
ROOT="$(dirname $(cargo locate-project --workspace --message-format plain -q))"
TARGET="$(dirname $(cargo locate-project --message-format plain -q))"
rm $TARGET/neutron.wasm || true
cd $ROOT && optimize $ROOT && cp ./artifacts/neutron.wasm $TARGET
cd $TARGET

HASH=$(docker exec neutron neutrond tx wasm store /contracts/neutron.wasm \
  --gas-prices 0.1untrn --gas 4500000 --gas-adjustment 1.3 -y \
  --output json -b sync --keyring-backend test --from demowallet1 \
  --home data/test-1/ --chain-id test-1 | jq -r '.txhash')

echo "waiting for block, thx cosmos-sdk team for removing -b block, cosmos ux sucks"
sleep 2

CODE=$(docker exec neutron neutrond query tx --type=hash $HASH \
  --output json --home data/test-1/ --chain-id test-1 \
  | jq -r '.logs[0].events[] | select(.type=="store_code").attributes[] | select(.key=="code_id").value'
)

echo "code id: $CODE"

CONTROLLER=$(docker exec neutron neutrond keys show demowallet2 --keyring-backend test --home data/test-1/ --output json|jq .address)
echo "controller: $CONTROLLER"

INIT="{\"chain\": \"test-1\", \"controller\": $CONTROLLER,\"oracle\": $CONTROLLER, \"fee_recipient\": $CONTROLLER}"
echo "init with: $INIT"

HASH=$(docker exec neutron neutrond tx wasm instantiate $CODE "$INIT"\
  --label "filament outpost" --no-admin \
  --gas-prices 0.1untrn --gas 500000 --gas-adjustment 1.3 -y \
  --output json -b sync --keyring-backend test --from demowallet1 \
  --home data/test-1/ --chain-id test-1 \
  | jq -r '.txhash'
)
echo "waiting for block, thx cosmos-sdk team for removing -b block, cosmos ux sucks"
sleep 2

CONTRACT=$(docker exec neutron neutrond query tx --type=hash $HASH \
  --output json --home data/test-1/ --chain-id test-1 \
  | jq -r '.logs[0].events[] | select(.type=="instantiate").attributes[] | select(.key=="_contract_address").value'
)

echo "contract at: $CONTRACT"
