#!/bin/sh

# Create distribution directory and copy relevant files
echo "Preparing the distribution directory..."
mkdir -p dist || { echo "Failed to create 'dist' directory"; exit 1; }

echo "Copying WebAssembly and JavaScript files to 'dist'..."
cp crates/wasm/pkg/filament_hub_wasm* dist/ 2>/dev/null
if [ $? -ne 0 ]; then
  echo "Error: Failed to copy WASM and JavaScript files. Check if 'crates/wasm/pkg' contains the expected files."
  exit 1
fi

echo "Copying TypeScript definitions to 'dist'..."
cp bindings/*.ts dist/ 2>/dev/null
if [ $? -ne 0 ]; then
  echo "Error: Failed to copy TypeScript definition files. Check if 'bindings' directory contains .ts files."
  exit 1
fi

echo "Distribution directory prepared successfully."

