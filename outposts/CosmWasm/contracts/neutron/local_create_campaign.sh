#!/bin/bash

CONTROLLER=$(docker exec neutron neutrond keys show demowallet2 --keyring-backend test --home data/test-1/ --output json|jq .address)
echo "controller: $CONTROLLER"

CAMP=$(docker exec neutron neutrond keys show demowallet3 --keyring-backend test --home data/test-1/ --output json|jq .address)

echo "$CAMP creating a campaign at $1"

MSG="{\"create_campaign_msg\": {\"admin\": $CAMP, \"indexer\": $CAMP, \"attester\": $CAMP, \"segment_desc\": { \"kind\": \"github_all_contributors\", \"proof\": \"ed25519_signature\", \"sources\": []}, \"conversion_desc\": { \"kind\": { \"social\": \"github\"}, \"proof\": \"ed25519_signature\"}, \"payout_mech\": \"proportional_per_conversion\", \"ends_at\": 0}}"
echo "with $MSG"

HASH=$(docker exec neutron neutrond tx wasm execute $1 "$MSG" \
  --gas-prices 0.1untrn --gas 500000 --gas-adjustment 1.3 -y \
  --output json -b sync --keyring-backend test --from demowallet1 \
  --home data/test-1/ --chain-id test-1 \
  | jq -r '.txhash'
)

echo "waiting for block, thx cosmos-sdk team for removing -b block, cosmos ux sucks"
sleep 2

ID=$(docker exec neutron neutrond query tx --type=hash $HASH \
  --output json --home data/test-1/ --chain-id test-1 \
  | jq -r '.logs[0].events[] | select(.type == "wasm").attributes[] | select(.key == "campaign_id").value'
)

echo "created campaign with id: $ID"

echo "funding campaign $ID"

MSG="{\"fund_campaign_msg\": {\"id\": $ID, \"budget\": { \"fee\": {\"denom\": \"untrn\", \"amount\": \"100000\"}, \"incentives\": {\"denom\": \"untrn\", \"amount\": \"100000\"} } } }"

echo "with $MSG"

HASH=$(docker exec neutron neutrond tx wasm execute $1 "$MSG" --amount 200000untrn \
  --gas-prices 0.1untrn --gas 500000 --gas-adjustment 1.3 -y \
  --output json -b sync --keyring-backend test --from demowallet3 \
  --home data/test-1/ --chain-id test-1 \
  | jq -r '.txhash'
)

echo "waiting for block, thx cosmos-sdk team for removing -b block, cosmos ux sucks"
sleep 2

STATUS=$(docker exec neutron neutrond query tx --type=hash $HASH \
  --output json --home data/test-1/ --chain-id test-1 \
  | jq -r '.logs[0].events[] | select(.type == "wasm").attributes[] | select(.key == "campaign_status").value'
)

echo "campaign with id: $ID now in $STATUS"
