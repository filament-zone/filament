#!/usr/bin/env bash
trap 'jobs -p | xargs -r kill' EXIT
echo 'Running: '\''cd crates/rollup/'\'''
cd crates/rollup/
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi
echo 'Running: '\''make clean-db'\'''
make clean-db
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi
echo 'Running: '\''cargo run --bin node'\'''
cargo run --bin node &
sleep 20
echo 'Running: '\''make test-register-outpost'\'''
make test-register-outpost
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi
echo 'Running: '\''make wait-ten-seconds'\'''
make wait-ten-seconds
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi
echo 'Running: '\''make test-outpost-registry'\'''
make test-outpost-registry
if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi
echo 'Running: '\''curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"outpost_registry_getOutpost","params":{"chain_id":"neutron-1"},"id":1}' http://127.0.0.1:12345'\'''

output=$(curl -X POST -H "Content-Type: application/json" -d '{"jsonrpc":"2.0","method":"outpost_registry_getOutpost","params":{"chain_id":"neutron-1"},"id":1}' http://127.0.0.1:12345)
expected='{"jsonrpc":"2.0","result":{"chain_id":"neutron-1"},"id":1}
'
# Either of the two must be a substring of the other. This kinda protects us
# against whitespace differences, trimming, etc.
if ! [[ $output == *"$expected"* || $expected == *"$output"* ]]; then
    echo "'$expected' not found in text:"
    echo "'$output'"
    exit 1
fi

if [ $? -ne 0 ]; then
    echo "Expected exit code 0, got $?"
    exit 1
fi
echo "All tests passed!"; exit 0
