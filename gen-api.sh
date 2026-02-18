#!/usr/bin/env sh
set -eu

url="https://developer.shortcut.com/api/rest/v3/shortcut.openapi.json"
out="spec/shortcut.openapi.json"

curl -fsSL "$url" | jq . > "$out"

echo "Wrote $out"
