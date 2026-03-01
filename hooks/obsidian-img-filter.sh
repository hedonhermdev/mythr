#!/usr/bin/env bash
#
# Git smudge/clean filter for converting between Obsidian and GitHub image syntax.
#
# Clean  (local -> git): ![[image.png]]       -> ![image.png](./media/image.png)
# Smudge (git -> local): ![image.png](./media/image.png) -> ![[image.png]]
#
# Handles the Obsidian size modifier syntax: ![[image.png|300]] -> size is dropped.
#
# Usage:
#   hooks/obsidian-img-filter.sh clean  < file.md
#   hooks/obsidian-img-filter.sh smudge < file.md

set -euo pipefail

MODE="${1:-}"

if [[ "$MODE" == "clean" ]]; then
    # Obsidian -> GitHub
    # ![[filename.ext|optional_size]] -> ![filename.ext](./media/filename.ext)
    sed -E 's/!\[\[([^]|]+)(\|[0-9]+)?\]\]/![\1](.\/media\/\1)/g'

elif [[ "$MODE" == "smudge" ]]; then
    # GitHub -> Obsidian
    # ![filename.ext](./media/filename.ext) -> ![[filename.ext]]
    sed -E 's/!\[([^]]+)\]\(\.\/media\/[^)]+\)/![[\1]]/g'

else
    echo "Usage: $0 <clean|smudge>" >&2
    exit 1
fi
