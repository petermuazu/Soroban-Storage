#!/usr/bin/env bash
# End-to-end check of soroban-storage on a real (local standalone) network.
#
# Builds the vault example with `stellar contract build`, deploys it to a
# local quickstart node and checks the TTLs the network actually records.
#
# Requires: docker, stellar-cli (v25.2+), curl.
# Usage:    scripts/e2e-local.sh            # starts the local node if needed
#           KEEP_NODE=1 scripts/e2e-local.sh # leave the node running after
set -euo pipefail

cd "$(dirname "$0")/.."

N=(--network local)
RPC=http://localhost:8000/rpc

# Must match the policies in examples/vault/src/lib.rs.
INSTANCE_EXTEND_TO=$((30 * 17280))  # days(30)
BALANCE_EXTEND_TO=$((120 * 17280))  # days(120)
COOLDOWN_EXTEND_TO=720              # hours(1)

pass() { echo "  ✅ $*"; }
fail() { echo "  ❌ $*" >&2; exit 1; }

rpc() {
  curl -s -X POST "$RPC" -H 'content-type: application/json' \
    -d "{\"jsonrpc\":\"2.0\",\"id\":1,\"method\":\"$1\"}"
}

started_node=0
if ! rpc getHealth 2>/dev/null | grep -q healthy; then
  echo "Starting local network..."
  stellar container start local >/dev/null
  started_node=1
fi
cleanup() {
  if [[ $started_node == 1 && -z "${KEEP_NODE:-}" ]]; then
    stellar container stop local >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for _ in $(seq 1 90); do
  rpc getHealth 2>/dev/null | grep -q healthy && break
  sleep 2
done
rpc getHealth | grep -q healthy || fail "local RPC did not become healthy"

latest_ledger() { rpc getLatestLedger | grep -o '"sequence":[0-9]*' | cut -d: -f2; }
wait_next_ledger() {
  local start; start=$(latest_ledger)
  while [[ $(latest_ledger) -le $start ]]; do sleep 1; done
}

# Decodes a base64 XDR uint32 (big-endian) to decimal.
u32() { echo "$1" | base64 -d | od -An -tu1 | awk '{print $1*16777216 + $2*65536 + $3*256 + $4}'; }

# `stellar contract read --output xdr` prints: key,value,last_modified,live_until
# ttl_of <read args...> prints "<last_modified_ledger> <live_until_ledger>".
ttl_of() {
  local line
  line=$(stellar contract read --id "$ID" "${N[@]}" --output xdr "$@" | head -1)
  IFS=, read -r _ _ mod until <<<"$line"
  echo "$(u32 "$mod") $(u32 "$until")"
}
entry_ttl() {
  local key
  key=$(echo "$2" | stellar xdr encode --type ScVal)
  ttl_of --durability "$1" --key-xdr "$key"
}
instance_ttl() { ttl_of --durability persistent; }

echo "Building vault example..."
stellar contract build --package vault-example >/dev/null 2>&1
WASM=target/wasm32v1-none/release/vault_example.wasm

echo "Funding accounts..."
stellar keys generate e2e-admin "${N[@]}" --fund --overwrite >/dev/null 2>&1
stellar keys generate e2e-user "${N[@]}" --fund --overwrite >/dev/null 2>&1
ADMIN=$(stellar keys address e2e-admin)
USER=$(stellar keys address e2e-user)

echo "Deploying..."
ID=$(stellar contract deploy --wasm "$WASM" --source e2e-admin "${N[@]}" -- --admin "$ADMIN" 2>/dev/null | tail -1)
echo "  contract $ID"

invoke() { stellar contract invoke --id "$ID" --source e2e-user "${N[@]}" -- "$@" 2>&1; }

echo "Checks:"

read -r mod until < <(instance_ttl)
[[ $((until - mod)) == "$INSTANCE_EXTEND_TO" ]] \
  && pass "instance TTL set to days(30) by the constructor ($((until - mod)) ledgers)" \
  || fail "instance TTL is $((until - mod)), expected $INSTANCE_EXTEND_TO"

out=$(invoke deposit --from "$USER" --amount 100)
[[ $out == *'"100"'* ]] || fail "deposit returned: $out"
BAL_KEY="{\"vec\":[{\"symbol\":\"Balance\"},{\"address\":\"$USER\"}]}"
read -r mod until < <(entry_ttl persistent "$BAL_KEY")
[[ $((until - mod)) == "$BALANCE_EXTEND_TO" ]] \
  && pass "new balance entry extended to days(120) ($((until - mod)) ledgers)" \
  || fail "balance TTL is $((until - mod)), expected $BALANCE_EXTEND_TO"
first_until=$until

wait_next_ledger
out=$(invoke deposit --from "$USER" --amount 50)
[[ $out == *'"150"'* ]] || fail "second deposit returned: $out"
read -r mod until < <(entry_ttl persistent "$BAL_KEY")
[[ $until == "$first_until" ]] \
  && pass "overwrite of a healthy entry left its TTL alone (no rent paid)" \
  || fail "overwrite changed live_until from $first_until to $until"

out=$(invoke withdraw --to "$USER" --amount 30)
[[ $out == *'"120"'* ]] || fail "withdraw returned: $out"
CD_KEY="{\"vec\":[{\"symbol\":\"Cooldown\"},{\"address\":\"$USER\"}]}"
read -r mod until < <(entry_ttl temporary "$CD_KEY")
[[ $((until - mod)) == "$COOLDOWN_EXTEND_TO" ]] \
  && pass "temporary cooldown entry lives for hours(1) ($((until - mod)) ledgers)" \
  || fail "cooldown TTL is $((until - mod)), expected $COOLDOWN_EXTEND_TO"

out=$(invoke withdraw --to "$USER" --amount 1 || true)
[[ $out == *"Error(Contract, #5)"* ]] \
  && pass "second withdraw rejected with CoolingDown (#5)" \
  || fail "expected CoolingDown error, got: $out"

out=$(invoke balance --owner "$USER")
[[ $out == *'"120"'* ]] && pass "balance reads back 120" || fail "balance returned: $out"

echo "All end-to-end checks passed."
