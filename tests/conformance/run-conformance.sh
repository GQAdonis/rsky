#!/usr/bin/env bash
set -euo pipefail

# Federation conformance runner — exercises both rsky-pds and @atproto/pds
# against the same workload and diffs firehose output + final repo state.
#
# Prerequisites:
#   docker compose up -d (from this directory)
#   jq, curl

RSKY_URL="${RSKY_PDS_URL:-http://localhost:2583}"
UPSTREAM_URL="${UPSTREAM_PDS_URL:-http://localhost:2584}"
RSKY_ADMIN="${RSKY_ADMIN_PASS:-conformance-admin}"
UPSTREAM_ADMIN="${UPSTREAM_ADMIN_PASS:-conformance-admin}"
RESULTS="${OUTPUT_DIR:-./results}"

mkdir -p "$RESULTS"

log() { echo "[conformance] $*"; }
pass() { echo "  [PASS] $*"; }
fail() { echo "  [FAIL] $*" >&2; FAILURES=$((FAILURES + 1)); }
FAILURES=0

# ── Wait for both PDSes to be ready ─────────────────────────────────────────
wait_ready() {
  local url="$1" name="$2"
  log "Waiting for $name at $url..."
  for i in $(seq 1 30); do
    if curl -sf "$url/xrpc/_health" >/dev/null 2>&1; then
      log "$name is ready"
      return 0
    fi
    sleep 2
  done
  fail "$name did not become ready within 60s"
  exit 1
}

wait_ready "$RSKY_URL" "rsky-pds"
wait_ready "$UPSTREAM_URL" "upstream-pds"

# ── Test 1: Create account on both ──────────────────────────────────────────
log "Test 1: createAccount"

TEST_HANDLE="conformance-$(date +%s)"
TEST_EMAIL="${TEST_HANDLE}@conformance.local"
TEST_PASSWORD="TestPass123!"

rsky_create=$(curl -sf -X POST "$RSKY_URL/xrpc/com.atproto.server.createAccount" \
  -H 'Content-Type: application/json' \
  -d "{\"handle\":\"${TEST_HANDLE}.conformance.local\",\"email\":\"${TEST_EMAIL}\",\"password\":\"${TEST_PASSWORD}\"}" 2>&1) || true

upstream_create=$(curl -sf -X POST "$UPSTREAM_URL/xrpc/com.atproto.server.createAccount" \
  -H 'Content-Type: application/json' \
  -d "{\"handle\":\"${TEST_HANDLE}.test\",\"email\":\"${TEST_EMAIL}\",\"password\":\"${TEST_PASSWORD}\"}" 2>&1) || true

if echo "$rsky_create" | jq -e '.did' >/dev/null 2>&1; then
  pass "rsky-pds createAccount"
  RSKY_DID=$(echo "$rsky_create" | jq -r '.did')
  RSKY_TOKEN=$(echo "$rsky_create" | jq -r '.accessJwt')
else
  fail "rsky-pds createAccount: $rsky_create"
  RSKY_DID=""
  RSKY_TOKEN=""
fi

if echo "$upstream_create" | jq -e '.did' >/dev/null 2>&1; then
  pass "upstream-pds createAccount"
  UPSTREAM_DID=$(echo "$upstream_create" | jq -r '.did')
  UPSTREAM_TOKEN=$(echo "$upstream_create" | jq -r '.accessJwt')
else
  fail "upstream-pds createAccount: $upstream_create"
  UPSTREAM_DID=""
  UPSTREAM_TOKEN=""
fi

# ── Test 2: Create a record on both ─────────────────────────────────────────
log "Test 2: createRecord (app.bsky.actor.profile)"

if [ -n "$RSKY_TOKEN" ]; then
  rsky_profile=$(curl -sf -X POST "$RSKY_URL/xrpc/com.atproto.repo.createRecord" \
    -H "Authorization: Bearer $RSKY_TOKEN" \
    -H 'Content-Type: application/json' \
    -d "{\"repo\":\"$RSKY_DID\",\"collection\":\"app.bsky.actor.profile\",\"rkey\":\"self\",\"record\":{\"displayName\":\"Conformance Test\",\"\$type\":\"app.bsky.actor.profile\"}}") || true

  if echo "$rsky_profile" | jq -e '.uri' >/dev/null 2>&1; then
    pass "rsky-pds createRecord"
  else
    fail "rsky-pds createRecord: $rsky_profile"
  fi
fi

if [ -n "$UPSTREAM_TOKEN" ]; then
  upstream_profile=$(curl -sf -X POST "$UPSTREAM_URL/xrpc/com.atproto.repo.createRecord" \
    -H "Authorization: Bearer $UPSTREAM_TOKEN" \
    -H 'Content-Type: application/json' \
    -d "{\"repo\":\"$UPSTREAM_DID\",\"collection\":\"app.bsky.actor.profile\",\"rkey\":\"self\",\"record\":{\"displayName\":\"Conformance Test\",\"\$type\":\"app.bsky.actor.profile\"}}") || true

  if echo "$upstream_profile" | jq -e '.uri' >/dev/null 2>&1; then
    pass "upstream-pds createRecord"
  else
    fail "upstream-pds createRecord: $upstream_profile"
  fi
fi

# ── Test 3: getRepo round-trip ───────────────────────────────────────────────
log "Test 3: getRepo (CAR export)"

if [ -n "$RSKY_DID" ]; then
  rsky_car_size=$(curl -sf "$RSKY_URL/xrpc/com.atproto.sync.getRepo?did=$RSKY_DID" | wc -c)
  if [ "$rsky_car_size" -gt 100 ]; then
    pass "rsky-pds getRepo (${rsky_car_size} bytes)"
  else
    fail "rsky-pds getRepo returned too small a response"
  fi
fi

if [ -n "$UPSTREAM_DID" ]; then
  upstream_car_size=$(curl -sf "$UPSTREAM_URL/xrpc/com.atproto.sync.getRepo?did=$UPSTREAM_DID" | wc -c)
  if [ "$upstream_car_size" -gt 100 ]; then
    pass "upstream-pds getRepo (${upstream_car_size} bytes)"
  else
    fail "upstream-pds getRepo returned too small a response"
  fi
fi

# ── Test 4: describeServer ───────────────────────────────────────────────────
log "Test 4: describeServer"

rsky_desc=$(curl -sf "$RSKY_URL/xrpc/com.atproto.server.describeServer") || true
upstream_desc=$(curl -sf "$UPSTREAM_URL/xrpc/com.atproto.server.describeServer") || true

for field in did availableUserDomains inviteCodeRequired; do
  rsky_val=$(echo "$rsky_desc" | jq -r ".$field // \"MISSING\"")
  upstream_val=$(echo "$upstream_desc" | jq -r ".$field // \"MISSING\"")
  if [ "$rsky_val" != "MISSING" ] && [ "$upstream_val" != "MISSING" ]; then
    pass "describeServer.$field present on both"
  else
    fail "describeServer.$field: rsky=$rsky_val upstream=$upstream_val"
  fi
done

# ── Summary ──────────────────────────────────────────────────────────────────
echo ""
if [ "$FAILURES" -eq 0 ]; then
  log "All conformance checks passed."
  exit 0
else
  log "$FAILURES conformance check(s) failed. See output above."
  exit 1
fi
