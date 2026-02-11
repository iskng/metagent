#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
BIN="$REPO_ROOT/target/release/mung"

cargo build --release

if [[ "$(uname -s)" == "Darwin" ]]; then
  if [[ -z "${METAGENT_SKIP_CODESIGN:-}" ]]; then
    if command -v xattr >/dev/null 2>&1; then
      xattr -d com.apple.quarantine "$BIN" 2>/dev/null || true
      xattr -d com.apple.provenance "$BIN" 2>/dev/null || true
    fi

    if command -v codesign >/dev/null 2>&1; then
      IDENTITY="${METAGENT_CODESIGN_ID:-}"
      if [[ -z "$IDENTITY" ]] && command -v security >/dev/null 2>&1; then
        IDENTITY="$(/usr/bin/security find-identity -p codesigning -v 2>/dev/null | awk -F\" '/Developer ID Application:/{print $2; exit}')"
      fi

      if [[ -n "$IDENTITY" ]]; then
        codesign --force --options runtime --timestamp -s "$IDENTITY" "$BIN"
      else
        codesign --force -s - "$BIN"
      fi
    fi
  fi
fi
