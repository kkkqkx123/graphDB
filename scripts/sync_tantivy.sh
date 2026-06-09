#!/usr/bin/env bash

set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
SUBMODULE="$ROOT/crates/tantivy"
UPSTREAM_REMOTE="${UPSTREAM_REMOTE:-upstream}"
UPSTREAM_BRANCH="${UPSTREAM_BRANCH:-main}"
FEATURE_BRANCH="${FEATURE_BRANCH:-feat/add-configurable-k1-b}"
PROXY_URL="${PROXY_URL:-http://localhost:7890}"
export https_proxy="$PROXY_URL"
export http_proxy="$PROXY_URL"

git -C "$SUBMODULE" config rerere.enabled true
git -C "$SUBMODULE" config rerere.autoupdate true
git -C "$SUBMODULE" config merge.conflictstyle zdiff3

git -C "$SUBMODULE" fetch "$UPSTREAM_REMOTE"
git -C "$SUBMODULE" checkout "$FEATURE_BRANCH"

if ! git -C "$SUBMODULE" merge --no-edit "$UPSTREAM_REMOTE/$UPSTREAM_BRANCH"; then
    echo "Merge conflict detected in crates/tantivy."
    echo "Resolve the conflict inside the submodule, then run:"
    echo "  git -C crates/tantivy add <files>"
    echo "  git -C crates/tantivy merge --continue"
    exit 1
fi

(
    cd "$ROOT"
    cargo check --workspace --features server,fulltext-search,grpc,qdrant
)

git -C "$SUBMODULE" push -u origin "$FEATURE_BRANCH"

echo "Sync finished."
echo "If the submodule commit should be recorded in the root repository, run:"
echo "  git -C \"$ROOT\" add crates/tantivy"
echo "If you want the fork to use this branch by default, change the default branch on GitHub and then run:"
echo "  git -C \"$SUBMODULE\" remote set-head origin -a"
