#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "Usage: $0 <new-version>"
  echo "Example: $0 0.1.2"
  exit 1
fi

NEW_VERSION="$1"
# Strip any leading 'v' if user accidentally passed e.g. v0.1.2
NEW_VERSION="${NEW_VERSION#v}"

CURRENT_VERSION=$(sed -n 's/^version = "\(.*\)"/\1/p' Cargo.toml | head -n 1)

if [ -z "$CURRENT_VERSION" ]; then
  echo "Error: Could not determine current version from Cargo.toml"
  exit 1
fi

if [ "$CURRENT_VERSION" = "$NEW_VERSION" ]; then
  echo "Version is already $NEW_VERSION. Nothing to update."
  exit 0
fi

echo "Bumping version: $CURRENT_VERSION -> $NEW_VERSION"

# 1. Update workspace Cargo.toml
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

# 2. Update npm/package.json
sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" npm/package.json

# 3. Update npm/native/package.json
sed -i "s/\"version\": \"$CURRENT_VERSION\"/\"version\": \"$NEW_VERSION\"/" npm/native/package.json

# 4. Update npm/native/index.js if present
if [ -f "npm/native/index.js" ]; then
  sed -i "s/$CURRENT_VERSION/$NEW_VERSION/g" npm/native/index.js
fi

# 5. Update README.md
if [ -f "README.md" ]; then
  sed -i "s/ashwa = \"$CURRENT_VERSION\"/ashwa = \"$NEW_VERSION\"/" README.md
fi

# 6. Update pypi/pyproject.toml
if [ -f "pypi/pyproject.toml" ]; then
  sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" pypi/pyproject.toml
fi

# 7. Update Cargo.lock via cargo check
if command -v cargo &>/dev/null; then
  echo "Updating Cargo.lock..."
  cargo check --workspace >/dev/null 2>&1 || true
fi

echo "Successfully updated version from $CURRENT_VERSION to $NEW_VERSION across all project configs & documentation!"
