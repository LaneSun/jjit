#!/bin/bash
set -e

# Release script for jjit
# Usage: ./scripts/release.sh [version]

VERSION=${1:-patch}

echo "=== jjit Release Script ==="
echo

# Check if we're in a clean state
if [ -n "$(jj diff 2>/dev/null)" ]; then
    echo "Error: Working copy has uncommitted changes."
    echo "Please commit or discard changes before releasing."
    exit 1
fi

# Get current version
CURRENT_VERSION=$(grep '^version' Cargo.toml | head -1 | sed 's/.*"\(.*\)".*/\1/')
echo "Current version: $CURRENT_VERSION"

# Calculate new version
IFS='.' read -r MAJOR MINOR PATCH <<< "$CURRENT_VERSION"
case $VERSION in
    major)
        NEW_VERSION="$((MAJOR + 1)).0.0"
        ;;
    minor)
        NEW_VERSION="$MAJOR.$((MINOR + 1)).0"
        ;;
    patch|*)
        NEW_VERSION="$MAJOR.$MINOR.$((PATCH + 1))"
        ;;
esac

echo "New version: $NEW_VERSION"
echo

# Update version in Cargo.toml
sed -i "s/^version = \"$CURRENT_VERSION\"/version = \"$NEW_VERSION\"/" Cargo.toml

echo "Updated Cargo.toml version"

# Commit version bump
jj describe -m "chore(release): bump version to $NEW_VERSION"
jj commit

# Move main bookmark and push
jj bookmark set main -r @- --allow-backwards
jj git push --remote origin --bookmark main

# Create git tag
jj git push --remote origin --bookmark main --tag "refs/tags/v$NEW_VERSION"

echo
echo "=== Release v$NEW_VERSION Complete ==="
echo
echo "Next steps:"
echo "1. Publish to crates.io: cargo publish"
echo "2. Create GitHub Release with binaries (CI will handle this automatically)"
echo
echo "To publish now, run: cargo publish"
