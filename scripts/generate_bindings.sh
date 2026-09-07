#!/bin/bash
set -e

if [ ! -f .testnet-addresses ]; then
    echo "Error: .testnet-addresses not found. Run deploy_testnet.sh first!"
    exit 1
fi

source .testnet-addresses

echo "Generating TypeScript bindings..."
mkdir -p bindings

echo "Generating Factory bindings..."
stellar contract bindings typescript --network testnet --contract-id "$FACTORY_ADDRESS" --output-dir bindings/factory

echo "Generating Policy bindings..."
stellar contract bindings typescript --network testnet --contract-id "$POLICY_ADDRESS" --output-dir bindings/policy

echo "Generating Wallet bindings (from local wasm since it's dynamically deployed)..."
stellar contract bindings typescript --wasm target/wasm32v1-none/release/wallet.wasm --output-dir bindings/wallet

echo "Bindings generated in ./bindings/"
