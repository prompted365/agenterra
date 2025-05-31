#!/usr/bin/env bash
# update_petstore_fixtures.sh
#
# Downloads and pretty-prints the latest Swagger Petstore OpenAPI v2 and v3 JSON specs.
# Places them in tests/fixtures/openapi/ for local, reproducible, and human-readable testing.
#
# Usage: ./update_petstore_fixtures.sh
#
# Dependencies: curl, jq (https://stedolan.github.io/jq/)
#
# If jq is missing, you'll get a friendly error and install hint.

set -euo pipefail

# Check for jq
if ! command -v jq >/dev/null 2>&1; then
  echo "Error: jq is required but not installed. Install it with 'brew install jq' (macOS) or your package manager (Linux)." >&2
  exit 1
fi

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
OPENAPI_DIR="$ROOT_DIR/openapi"

# URLs for official Petstore specs
PETSTORE_V2_URL="https://petstore.swagger.io/v2/swagger.json"
PETSTORE_V3_URL="https://petstore3.swagger.io/api/v3/openapi.json"

# Output paths
V2_OUT="$OPENAPI_DIR/petstore.swagger.v2.json"
V3_OUT="$OPENAPI_DIR/petstore.openapi.v3.json"

# Download and pretty-print v2
curl -sSL "$PETSTORE_V2_URL" | jq . > "$V2_OUT.tmp"
mv "$V2_OUT.tmp" "$V2_OUT"
echo "Updated $V2_OUT"

# Download and pretty-print v3
curl -sSL "$PETSTORE_V3_URL" | jq . > "$V3_OUT.tmp"
mv "$V3_OUT.tmp" "$V3_OUT"
echo "Updated $V3_OUT"

echo "Petstore OpenAPI fixtures updated!"

# Reformat both files for human readability (in case of manual edits/minified JSON)
jq . "$V2_OUT" > "$V2_OUT.tmp" && mv "$V2_OUT.tmp" "$V2_OUT"
jq . "$V3_OUT" > "$V3_OUT.tmp" && mv "$V3_OUT.tmp" "$V3_OUT"
echo "Reformatted both OpenAPI fixture files for readability."
