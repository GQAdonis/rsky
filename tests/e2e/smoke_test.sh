#!/usr/bin/env bash
# End-to-end smoke test for the know-me.tools AT Protocol stack.
# Tests: createAccount → createSession → createRecord → getTimeline
# Usage: ./tests/e2e/smoke_test.sh [--pds https://pds.know-me.tools] [--handle alice.know-me.tools] [--password TestPass123!]
set -euo pipefail

PDS_URL="${PDS_URL:-https://pds.know-me.tools}"
APPVIEW_URL="${APPVIEW_URL:-https://appview.know-me.tools}"
RELAY_URL="${RELAY_URL:-https://relay.know-me.tools}"
TEST_HANDLE="${TEST_HANDLE:-smoketest-$(date +%s).know-me.tools}"
TEST_EMAIL="${TEST_EMAIL:-smoketest-$(date +%s)@example.com}"
TEST_PASSWORD="${TEST_PASSWORD:-SmokeTestPass$(date +%s)!}"
INVITE_CODE="${INVITE_CODE:-}"

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

pass() { echo -e "${GREEN}✓${NC} $1"; }
fail() { echo -e "${RED}✗${NC} $1"; exit 1; }
info() { echo -e "${YELLOW}→${NC} $1"; }

check_deps() {
    for cmd in curl jq; do
        command -v "$cmd" >/dev/null 2>&1 || fail "Required command not found: $cmd"
    done
}

# 1. Infrastructure health checks
check_health() {
    info "Checking infrastructure health..."

    relay_health=$(curl -sf "$RELAY_URL/_health" 2>&1 || echo "FAIL")
    if [ "$relay_health" = "ok" ]; then
        pass "Relay healthy: $RELAY_URL"
    else
        fail "Relay unhealthy: $RELAY_URL → $relay_health"
    fi

    pds_health=$(curl -sf "$PDS_URL/xrpc/_health" 2>&1 || echo "FAIL")
    if echo "$pds_health" | grep -q "version\|OK\|ok"; then
        pass "PDS healthy: $PDS_URL"
    else
        fail "PDS unhealthy: $PDS_URL → $pds_health"
    fi

    appview_health=$(curl -sf "$APPVIEW_URL/xrpc/_health" 2>&1 || echo "FAIL")
    if echo "$appview_health" | grep -q "OK\|ok\|version"; then
        pass "AppView healthy: $APPVIEW_URL"
    else
        fail "AppView unhealthy: $APPVIEW_URL → $appview_health"
    fi
}

# 2. Create account
create_account() {
    info "Creating test account: $TEST_HANDLE"

    ACCOUNT_PAYLOAD=$(jq -n \
        --arg handle "$TEST_HANDLE" \
        --arg email "$TEST_EMAIL" \
        --arg password "$TEST_PASSWORD" \
        --arg invite "$INVITE_CODE" \
        '{handle: $handle, email: $email, password: $password} + (if $invite != "" then {inviteCode: $invite} else {} end)')

    ACCOUNT_RESPONSE=$(curl -sf -X POST "$PDS_URL/xrpc/com.atproto.server.createAccount" \
        -H "Content-Type: application/json" \
        -d "$ACCOUNT_PAYLOAD" 2>&1) || fail "createAccount request failed: $ACCOUNT_RESPONSE"

    DID=$(echo "$ACCOUNT_RESPONSE" | jq -r '.did // empty')
    ACCESS_JWT=$(echo "$ACCOUNT_RESPONSE" | jq -r '.accessJwt // empty')

    [ -n "$DID" ] || fail "createAccount: no DID in response: $ACCOUNT_RESPONSE"
    [ -n "$ACCESS_JWT" ] || fail "createAccount: no accessJwt in response"

    pass "Account created: $DID"
    export DID ACCESS_JWT
}

# 3. Create session (login) with existing account
create_session() {
    info "Creating session for: $TEST_HANDLE"

    SESSION_RESPONSE=$(curl -sf -X POST "$PDS_URL/xrpc/com.atproto.server.createSession" \
        -H "Content-Type: application/json" \
        -d "{\"identifier\": \"$TEST_HANDLE\", \"password\": \"$TEST_PASSWORD\"}" 2>&1) || \
        fail "createSession request failed: $SESSION_RESPONSE"

    DID=$(echo "$SESSION_RESPONSE" | jq -r '.did // empty')
    ACCESS_JWT=$(echo "$SESSION_RESPONSE" | jq -r '.accessJwt // empty')

    [ -n "$DID" ] || fail "createSession: no DID in response: $SESSION_RESPONSE"
    [ -n "$ACCESS_JWT" ] || fail "createSession: no accessJwt in response"

    pass "Session created: $DID"
    export DID ACCESS_JWT
}

# 4. Create a post record
create_post() {
    info "Creating test post..."

    POST_TEXT="Smoke test post $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    POST_PAYLOAD=$(jq -n \
        --arg did "$DID" \
        --arg text "$POST_TEXT" \
        '{repo: $did, collection: "app.bsky.feed.post", record: {"\$type": "app.bsky.feed.post", text: $text, createdAt: (now | strftime("%Y-%m-%dT%H:%M:%S.000Z"))}}')

    POST_RESPONSE=$(curl -sf -X POST "$PDS_URL/xrpc/com.atproto.repo.createRecord" \
        -H "Content-Type: application/json" \
        -H "Authorization: Bearer $ACCESS_JWT" \
        -d "$POST_PAYLOAD" 2>&1) || fail "createRecord request failed: $POST_RESPONSE"

    POST_URI=$(echo "$POST_RESPONSE" | jq -r '.uri // empty')
    POST_CID=$(echo "$POST_RESPONSE" | jq -r '.cid // empty')

    [ -n "$POST_URI" ] || fail "createRecord: no uri in response: $POST_RESPONSE"
    [ -n "$POST_CID" ] || fail "createRecord: no cid in response"

    pass "Post created: $POST_URI"
    export POST_URI POST_CID POST_TEXT
}

# 5. Verify appview timeline (with retry for indexing delay)
verify_timeline() {
    info "Verifying post appears in appview timeline (up to 30s for indexing)..."

    FOUND=0
    for i in $(seq 1 6); do
        TIMELINE_RESPONSE=$(curl -sf "$APPVIEW_URL/xrpc/app.bsky.feed.getTimeline" \
            -H "Authorization: Bearer $ACCESS_JWT" 2>&1) || {
            echo "  Attempt $i: timeline request failed"
            sleep 5
            continue
        }

        if echo "$TIMELINE_RESPONSE" | jq -e ".feed[] | select(.post.uri == \"$POST_URI\")" >/dev/null 2>&1; then
            FOUND=1
            break
        fi
        echo "  Attempt $i: post not yet indexed, waiting 5s..."
        sleep 5
    done

    if [ "$FOUND" = "1" ]; then
        pass "Post found in timeline"
    else
        fail "Post not found in timeline after 30s. Timeline response: $(echo "$TIMELINE_RESPONSE" | jq '.feed | length') items"
    fi
}

# 6. Handle resolution check
verify_handle_resolution() {
    info "Verifying handle resolution for $TEST_HANDLE..."

    RESOLVED_DID=$(curl -sf "https://$TEST_HANDLE/.well-known/atproto-did" 2>&1 || echo "FAIL")

    if [ "$RESOLVED_DID" = "$DID" ]; then
        pass "Handle resolves correctly: $TEST_HANDLE → $DID"
    else
        # DNS TXT check as fallback
        DNS_DID=$(dig +short TXT "_atproto.${TEST_HANDLE}" 2>/dev/null | tr -d '"' | sed 's/did=//')
        if [ "$DNS_DID" = "$DID" ]; then
            pass "Handle resolves via DNS TXT: $TEST_HANDLE → $DID"
        else
            fail "Handle resolution failed: $TEST_HANDLE → got '$RESOLVED_DID', expected '$DID'"
        fi
    fi
}

main() {
    check_deps

    # Parse args
    while [[ $# -gt 0 ]]; do
        case $1 in
            --pds) PDS_URL="$2"; shift 2;;
            --appview) APPVIEW_URL="$2"; shift 2;;
            --handle) TEST_HANDLE="$2"; shift 2;;
            --password) TEST_PASSWORD="$2"; shift 2;;
            --invite) INVITE_CODE="$2"; shift 2;;
            --use-existing) USE_EXISTING=1; shift;;
            *) echo "Unknown arg: $1"; shift;;
        esac
    done

    echo "=== rsky AT Protocol Smoke Test ==="
    echo "PDS:     $PDS_URL"
    echo "AppView: $APPVIEW_URL"
    echo "Relay:   $RELAY_URL"
    echo "Handle:  $TEST_HANDLE"
    echo ""

    check_health

    if [ "${USE_EXISTING:-0}" = "1" ]; then
        create_session
    else
        create_account
    fi

    create_post
    verify_timeline

    # Handle resolution only works for handles under know-me.tools with wildcard DNS
    if echo "$TEST_HANDLE" | grep -q "know-me.tools"; then
        verify_handle_resolution || echo "  (handle resolution skipped — wildcard DNS may not be configured)"
    fi

    echo ""
    echo -e "${GREEN}=== All smoke tests passed ===${NC}"
}

main "$@"
