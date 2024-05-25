#!/bin/bash

function optimize () {
  docker run --rm -v "$1":/code \
    --mount type=volume,source="$(basename "$1")_cache",target=/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    cosmwasm/optimizer:0.15.0 ./contracts/neutron/
}

# if only --out-dir was stable
ROOT="$(dirname $(cargo locate-project --workspace --message-format plain -q))"
# echo $ROOT
TARGET="$(dirname $(cargo locate-project --message-format plain -q))"
# echo $TARGET
rm $TARGET/neutron.wasm || true
cd $ROOT && optimize $ROOT && cp ./artifacts/neutron.wasm $TARGET
cd $TARGET

OUT=$(docker exec neutron neutrond tx wasm store /contracts/neutron.wasm \
  --gas-prices 0.1untrn --gas 4500000 --gas-adjustment 1.3 -y \
  --output json -b block --keyring-backend test --from demowallet1 \
  --home data/test-1/ --chain-id test-1)

CODE=$(echo $OUT|jq -r '.logs[0].events[-1].attributes[-1].value')
echo "code id: $CODE"

CONTROLLER=$(docker exec neutron neutrond keys show demowallet2 --keyring-backend test --home data/test-1/ --output json|jq .address)
echo "controller: $CONTROLLER"

INIT="{\"chain\": \"test-1\", \"controller\": $CONTROLLER,\"oracle\": $CONTROLLER, \"fee_recipient\": $CONTROLLER}"
echo "init with: $INIT"

CONTRACT=$(docker exec neutron neutrond tx wasm instantiate $CODE "$INIT"\
  --label "filament outpost" --no-admin \
  --gas-prices 0.1untrn --gas 500000 --gas-adjustment 1.3 -y \
  --output json -b block --keyring-backend test --from demowallet1 \
  --home data/test-1/ --chain-id test-1 \
  | jq '.logs[0].events[0].attributes[0].value'
)

echo "contract at: $CONTRACT"
