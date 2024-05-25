#!/bin/bash

CONTROLLER=$(docker exec neutron neutrond keys show demowallet2 --keyring-backend test --home data/test-1/ --output json|jq .address)
echo "controller: $CONTROLLER"

CAMP=$(docker exec neutron neutrond keys show demowallet3 --keyring-backend test --home data/test-1/ --output json|jq .address)

echo "$CAMP creating a campaign at $1"

MSG="{\"create_campaign_msg\": {\"admin\": $CAMP, \"indexer\": $CAMP, \"attester\": $CAMP, \"segment_desc\": { \"kind\": \"github_all_contributors\", \"proof\": \"ed25519_signature\", \"sources\": []}, \"conversion_desc\": { \"kind\": { \"social\": \"github\"}, \"proof\": \"ed25519_signature\"}, \"payout_mech\": \"proportional_per_conversion\", \"ends_at\": 0}}"
echo "with $MSG"

ID=$(docker exec neutron neutrond tx wasm execute $1 "$MSG" \
  --gas-prices 0.1untrn --gas 500000 --gas-adjustment 1.3 -y \
  --output json -b block --keyring-backend test --from demowallet1 \
  --home data/test-1/ --chain-id test-1 \
  | jq '.logs[0].events[] | select(.type == "wasm").attributes[] | select(.key == "campaign_id").value'
)

echo "created campaign with id: $ID"
