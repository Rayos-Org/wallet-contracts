#!/bin/bash
set -e

echo "Building contracts to wasm32v1-none..."
cargo build --target wasm32v1-none --release

echo "Checking for 'deployer' identity..."
if ! stellar keys ls | grep -q "deployer"; then
    echo "Creating 'deployer' identity and funding on testnet..."
    stellar keys generate deployer --network testnet
else
    echo "'deployer' identity found."
fi

FACTORY_WASM="target/wasm32v1-none/release/factory.wasm"
POLICY_WASM="target/wasm32v1-none/release/policy.wasm"
WALLET_WASM="target/wasm32v1-none/release/wallet.wasm"

echo ""
echo "======================================"
echo "Installing Wallet WASM..."
WALLET_HASH=$(stellar contract install --wasm $WALLET_WASM --source deployer --network testnet)
echo "Wallet WASM Hash: $WALLET_HASH"

echo "Deploying Factory Contract..."
FACTORY_ADDRESS=$(stellar contract deploy --wasm $FACTORY_WASM --source deployer --network testnet)
echo "Factory Address: $FACTORY_ADDRESS"

echo "Initializing Factory..."
stellar contract invoke --id $FACTORY_ADDRESS --source deployer --network testnet -- init --wasm_hash "$WALLET_HASH"

echo "Deploying Policy Contract..."
POLICY_ADDRESS=$(stellar contract deploy --wasm $POLICY_WASM --source deployer --network testnet)
echo "Policy Address: $POLICY_ADDRESS"

# We initialize the policy with the deployer as the owner for now.
DEPLOYER_PUBKEY=$(stellar keys address deployer)
echo "Initializing Policy with owner $DEPLOYER_PUBKEY..."
stellar contract invoke --id $POLICY_ADDRESS --source deployer --network testnet -- init --owner "$DEPLOYER_PUBKEY"

echo "======================================"
echo "Deployment successful!"
echo "WALLET_HASH=$WALLET_HASH"
echo "FACTORY_ADDRESS=$FACTORY_ADDRESS"
echo "POLICY_ADDRESS=$POLICY_ADDRESS"
echo "======================================"

# Save to a local file so bindings can pick it up
cat <<EOF > .testnet-addresses
FACTORY_ADDRESS=$FACTORY_ADDRESS
POLICY_ADDRESS=$POLICY_ADDRESS
EOF
