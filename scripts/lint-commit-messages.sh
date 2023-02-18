#!/bin/bash
set -euo pipefail

DIR=$(dirname $(realpath $0))

# The start of the range can be bumped up over time to something that
# all live branches include. Currently pointing to initial commit.
LINT_COMMITS_RANGE="2346e60e5e23e732240916df98294f48d23b82f7..HEAD"

function lint_commits_in_range() {
    # If the command wrapped in <() below fails, the outer script
    # won't. We run it here directly to ensure that it will fail the
    # lint if the commits range is incorrect.
    git rev-list --no-merges "$LINT_COMMITS_RANGE" > /dev/null

    while read -r rev; do
        lint_commit "$rev"
    done < <(git rev-list --no-merges "$LINT_COMMITS_RANGE")
}

function lint_commit() {
    local rev="$1"
    local commit_msg=$(git log --format=%B -n 1 "$rev")
    "$DIR"/lint-commit-message.sh <<<"$commit_msg"
}

lint_commits_in_range "$LINT_COMMITS_RANGE"
