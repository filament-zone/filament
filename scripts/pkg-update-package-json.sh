#!/bin/bash

set -ex

# Ensure that jq is installed
command -v jq >/dev/null 2>&1 || { echo "Error: jq is required but not installed. Please install jq."; exit 1; }

# Use GITHUB_SHA if available, otherwise use the current git commit short hash
if [ -z "$GITHUB_SHA" ]; then
  GITHUB_SHA=$(git rev-parse --short HEAD 2>/dev/null)
  if [ $? -ne 0 ]; then
    echo "Error: Failed to retrieve Git commit hash. Ensure you are in a git repository."
    exit 1
  fi
fi

# Check if package.json exists in the expected location
if [ ! -f crates/wasm/pkg/package.json ]; then
  echo "Error: 'package.json' not found in 'crates/wasm/pkg'. Ensure the file exists."
  exit 1
fi

echo "Updating 'package.json' with GitHub registry configuration and file properties..."

jq '.name = "@filament-zone/filament" |
    .main = "filament_hub_wasm.js" |
    .types = "index.d.ts" |
    .files = ["*.js", "*.wasm", "*.ts"] |
    .repository.url = "https://github.com/filament-zone/filament" |
    .license = "MIT" |
    .version = "0.1.0-'${GITHUB_SHA:0:7}'"' \
    crates/wasm/pkg/package.json > dist/package.json

if [ $? -ne 0 ]; then
  echo "Error: Failed to update 'package.json'."
  exit 1
fi

echo "'package.json' updated successfully."

