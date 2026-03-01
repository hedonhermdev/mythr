#!/usr/bin/env bash
#
# Sets up git filters for Obsidian <-> GitHub image syntax conversion.
# Run this once after cloning the repo.

set -euo pipefail

REPO_ROOT="$(git rev-parse --show-toplevel)"

git config filter.obsidian-img.clean 'hooks/obsidian-img-filter.sh clean'
git config filter.obsidian-img.smudge 'hooks/obsidian-img-filter.sh smudge'

# Re-apply the smudge filter to existing files so the working tree
# reflects Obsidian syntax even if they were checked out before setup.
git checkout -- "${REPO_ROOT}/log/"*.md 2>/dev/null || true

echo "Done. Git filters for Obsidian image syntax are now active."
