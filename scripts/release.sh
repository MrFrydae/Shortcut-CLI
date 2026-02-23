#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/release.sh <semver>

Example:
  scripts/release.sh 0.0.12
EOF
}

if [[ "${1:-}" == "-h" || "${1:-}" == "--help" || $# -ne 1 ]]; then
  usage
  exit $(( $# == 1 ? 0 : 1 ))
fi

VERSION="${1#v}"
if [[ ! "${VERSION}" =~ ^[0-9]+\.[0-9]+\.[0-9]+(-[0-9A-Za-z.-]+)?(\+[0-9A-Za-z.-]+)?$ ]]; then
  echo "Invalid semver: ${VERSION}" >&2
  echo "Expected format: MAJOR.MINOR.PATCH[-PRERELEASE][+BUILD]" >&2
  exit 1
fi

ROOT="$(git rev-parse --show-toplevel)"
cd "${ROOT}"

if ! git diff --quiet || ! git diff --cached --quiet; then
  echo "Working tree must be clean before release." >&2
  exit 1
fi

BRANCH="$(git rev-parse --abbrev-ref HEAD)"
if [[ "${BRANCH}" != "main" ]]; then
  echo "Releases must be created from main (current: ${BRANCH})." >&2
  exit 1
fi

echo "Updating Cargo.toml -> ${VERSION}"
perl -i -pe 's/^version = ".*"$/version = "'"${VERSION}"'"/' Cargo.toml

echo "Regenerating Cargo.lock"
cargo generate-lockfile

if ! perl -0777 -ne 'exit((/name = "shortcut-cli"\nversion = "'"${VERSION}"'"/s)?0:1)' Cargo.lock; then
  echo "Cargo.lock does not contain shortcut-cli version ${VERSION}" >&2
  exit 1
fi

echo "Building release artifacts (local verification)"
cargo build --locked --release

echo "Committing release metadata"
git add Cargo.toml Cargo.lock
git commit -m "release: v${VERSION}"

echo "Pushing commit to main"
git push origin main

echo "Release will trigger after CI succeeds on main."
echo "Tag will be created automatically from Cargo.toml version."
echo "Workflow: https://github.com/MrFrydae/Shortcut-CLI/actions/workflows/release.yml"
