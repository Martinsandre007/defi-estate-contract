#!/usr/bin/env bash
# Deploy one property's contracts to a Stellar network.
#
# Usage: scripts/deploy-contracts.sh <source-identity> <property-id> <name> <symbol> <max-shares> <payout-token-id> [network]
#
# Requires the Stellar CLI and a funded identity (stellar keys generate --global <name> --network testnet --fund).
# Records the deployed contract IDs in deployments/<network>.json keyed by property id, then copy them into the
# frontend and backend contract registries (src/shared/contracts/contracts.<network>.json).
set -euo pipefail

if [ "$#" -lt 6 ]; then
  sed -n '2,9p' "$0"
  exit 2
fi

SOURCE="$1" PROPERTY_ID="$2" NAME="$3" SYMBOL="$4" MAX_SHARES="$5" PAYOUT_TOKEN="$6" NETWORK="${7:-testnet}"
ADMIN="$(stellar keys address "$SOURCE")"
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
WASM_DIR="$SCRIPT_DIR/../target/wasm32v1-none/release"
DEPLOYMENTS_FILE="$SCRIPT_DIR/../deployments/$NETWORK.json"

"$SCRIPT_DIR/build-contracts.sh" >/dev/null

# Upload the child contract wasm blobs and capture their hashes for the factory.
TOKEN_HASH="$(stellar contract upload --wasm "$WASM_DIR/property_token.wasm" --source "$SOURCE" --network "$NETWORK")"
DISTRIBUTOR_HASH="$(stellar contract upload --wasm "$WASM_DIR/yield_distributor.wasm" --source "$SOURCE" --network "$NETWORK")"

# Deploy the factory (or reuse the one already recorded for this network).
FACTORY_ID="$(jq -r '.factory // empty' "$DEPLOYMENTS_FILE" 2>/dev/null || true)"
if [ -z "$FACTORY_ID" ]; then
  FACTORY_ID="$(stellar contract deploy --wasm "$WASM_DIR/property_factory.wasm" --source "$SOURCE" --network "$NETWORK" -- \
    --admin "$ADMIN" --token_hash "$TOKEN_HASH" --distributor_hash "$DISTRIBUTOR_HASH")"
fi

# Deploy the property's token and distributor through the factory in one call.
CREATE_OUTPUT="$(stellar contract invoke --id "$FACTORY_ID" --source "$SOURCE" --network "$NETWORK" -- \
  create_property \
  --property_id "$PROPERTY_ID" \
  --name "$NAME" \
  --symbol "$SYMBOL" \
  --max_supply "$MAX_SHARES" \
  --payout_token "$PAYOUT_TOKEN")"

# The factory returns a (token, distributor) tuple; parse the two addresses.
TOKEN_ID="$(printf '%s' "$CREATE_OUTPUT" | grep -oE 'C[A-Z0-9]{55}' | sed -n '1p')"
DISTRIBUTOR_ID="$(printf '%s' "$CREATE_OUTPUT" | grep -oE 'C[A-Z0-9]{55}' | sed -n '2p')"

if [ -z "$TOKEN_ID" ] || [ -z "$DISTRIBUTOR_ID" ]; then
  echo "Failed to parse factory output: $CREATE_OUTPUT" >&2
  exit 1
fi

echo "propertyFactory:  $FACTORY_ID"
echo "propertyToken:    $TOKEN_ID"
echo "yieldDistributor: $DISTRIBUTOR_ID"

# Record the deployed IDs in deployments/<network>.json, keyed by property id.
mkdir -p "$(dirname "$DEPLOYMENTS_FILE")"
if [ ! -f "$DEPLOYMENTS_FILE" ]; then
  echo '{}' >"$DEPLOYMENTS_FILE"
fi

TMP_FILE="$(mktemp)"
trap 'rm -f "$TMP_FILE"' EXIT

jq \
  --arg factoryId "$FACTORY_ID" \
  --arg propertyId "$PROPERTY_ID" \
  --arg tokenId "$TOKEN_ID" \
  --arg distributorId "$DISTRIBUTOR_ID" \
  '.factory = $factoryId
   | .contracts = (.contracts // {})
   | .contracts[$propertyId] = {
       propertyToken: $tokenId,
       yieldDistributor: $distributorId
     }' \
  "$DEPLOYMENTS_FILE" >"$TMP_FILE"

mv "$TMP_FILE" "$DEPLOYMENTS_FILE"
trap - EXIT

echo "Recorded contract IDs in deployments/$NETWORK.json under contracts.$PROPERTY_ID"