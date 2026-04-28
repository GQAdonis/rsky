#!/usr/bin/env bash
# Validates PDS environment configuration before deploy or locally.
# Usage: PDS_HOSTNAME=pds.know-me.tools ... ./scripts/verify-config.sh
# Or:    kubectl exec -n atproto statefulset/rsky-pds -- env | ./scripts/verify-config.sh --from-env
set -euo pipefail

ERRORS=0
WARNINGS=0

fail()    { echo "  FAIL: $*"; ((ERRORS++)); }
warn()    { echo "  WARN: $*"; ((WARNINGS++)); }
ok()      { echo "    OK: $*"; }

require_var() {
    local var="$1"
    local val="${!var:-}"
    if [[ -z "$val" ]]; then
        fail "$var is not set"
    else
        ok "$var is set"
    fi
}

echo "=== rsky-pds config verification ==="
echo ""

echo "--- Required env vars ---"
for var in \
    PDS_HOSTNAME \
    PDS_SERVICE_DID \
    PDS_SERVICE_HANDLE_DOMAINS \
    PDS_ADMIN_PASS \
    PDS_JWT_KEY_K256_PRIVATE_KEY_HEX \
    PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX \
    PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX \
    DATABASE_URL; do
    require_var "$var"
done
echo ""

echo "--- Cross-variable consistency ---"
HOSTNAME="${PDS_HOSTNAME:-}"
DOMAINS="${PDS_SERVICE_HANDLE_DOMAINS:-}"
if [[ -n "$HOSTNAME" && -n "$DOMAINS" ]]; then
    COVERED=false
    IFS=',' read -ra DOMAIN_LIST <<< "$DOMAINS"
    for d in "${DOMAIN_LIST[@]}"; do
        d="${d#.}"  # strip leading dot
        d="${d// /}"  # strip spaces
        if [[ "$HOSTNAME" == *"$d" ]]; then
            COVERED=true
            break
        fi
    done
    if $COVERED; then
        ok "PDS_HOSTNAME ($HOSTNAME) is covered by PDS_SERVICE_HANDLE_DOMAINS ($DOMAINS)"
    else
        fail "PDS_HOSTNAME ($HOSTNAME) is NOT covered by PDS_SERVICE_HANDLE_DOMAINS ($DOMAINS) — handle registration will fail"
    fi
fi

SERVICE_DID="${PDS_SERVICE_DID:-}"
if [[ -n "$HOSTNAME" && -n "$SERVICE_DID" ]]; then
    if [[ "$SERVICE_DID" == "did:web:$HOSTNAME" ]]; then
        ok "PDS_SERVICE_DID matches did:web:PDS_HOSTNAME"
    else
        warn "PDS_SERVICE_DID ($SERVICE_DID) does not match did:web:$HOSTNAME — intentional?"
    fi
fi

echo ""
echo "--- Invite code setting ---"
INVITE_REQUIRED="${PDS_INVITE_REQUIRED:-true}"
if [[ "$INVITE_REQUIRED" == "false" ]]; then
    ok "PDS_INVITE_REQUIRED=false (open registration)"
else
    ok "PDS_INVITE_REQUIRED=true (invite-only)"
fi

echo ""
echo "--- Key format checks ---"
for var in \
    PDS_JWT_KEY_K256_PRIVATE_KEY_HEX \
    PDS_PLC_ROTATION_KEY_K256_PRIVATE_KEY_HEX \
    PDS_REPO_SIGNING_KEY_K256_PRIVATE_KEY_HEX; do
    val="${!var:-}"
    if [[ -n "$val" ]]; then
        len="${#val}"
        if [[ "$len" -eq 64 ]]; then
            ok "$var is 64 hex chars (correct)"
        else
            fail "$var is $len chars, expected 64"
        fi
    fi
done

echo ""
echo "==================================="
if [[ $ERRORS -gt 0 ]]; then
    echo "RESULT: $ERRORS error(s), $WARNINGS warning(s) — DO NOT DEPLOY"
    exit 1
elif [[ $WARNINGS -gt 0 ]]; then
    echo "RESULT: 0 errors, $WARNINGS warning(s) — review before deploying"
    exit 0
else
    echo "RESULT: All checks passed"
    exit 0
fi
