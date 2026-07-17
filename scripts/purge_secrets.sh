#!/bin/bash
# Purge secrets from git history
# WARNING: This rewrites git history. Force-push required after running.
# Usage: ./scripts/purge_secrets.sh

set -e

echo "=== AllBright Secret Purge Script ==="
echo "This will rewrite git history to remove:"
echo "  - dist/ and build/ directories"
echo "  - Any files matching *.key, *.pem, *.p12"
echo ""
read -p "Have you rotated ALL secrets? (yes/no): " ROTATED
if [ "$ROTATED" != "yes" ]; then
    echo "Aborting. Rotate all secrets first."
    exit 1
fi

echo ""
echo "Step 1: Removing build artifacts from git history..."
git filter-branch --force --index-filter \
    "git rm --cached --ignore-unmatch \
    dist/ \
    build/ \
    apps/dashboard/dist/ \
    apps/dashboard/build/ \
    src-tauri/dist/ \
    src-tauri/target/ \
    backend/target/ \
    *.key \
    *.pem \
    *.p12 \
    *.pfx \
    *.crt \
    *.cer \
    *.der" \
    --prune-empty --tag-name-filter cat -- --all

echo ""
echo "Step 2: Removing .env files from git history..."
git filter-branch --force --index-filter \
    "git rm --cached --ignore-unmatch \
    .env \
    .env.* \
    apps/*/.env \
    backend/.env \
    src-tauri/.env" \
    --prune-empty --tag-name-filter cat -- --all

echo ""
echo "Step 3: Cleaning up refs..."
rm -rf .git/refs/original/
git reflog expire --expire=now --all
git gc --prune=now --aggressive

echo ""
echo "Step 4: Verifying no secrets remain..."
if git log --all --pretty=format: --name-only --diff-filter=A | grep -E '^\.env|dist/|build/|\.pem|\.key|\.p12'; then
    echo "WARNING: Some files may still be in history. Review manually."
else
    echo "Clean: No secrets found in history."
fi

echo ""
echo "=== DONE ==="
echo "Next steps:"
echo "  1. Review changes: git log --oneline -10"
echo "  2. Force push: git push --force --all"
echo "  3. Contact GitHub Support to enable secret scanning"
