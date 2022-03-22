#!/bin/bash
set -euo pipefail

# The start of the range can be bumped up over time to something that
# all live branches include. Currently pointing to initial commit.
LINT_COMMITS_RANGE="214e69f8be0f8900a80c2de3255fc7938ea918c0..HEAD"
ALLOWED_TYPES=(
    build
    change
    ci
    dev
    doc
    feat
    fix
    perf
    refact
    revert
    style
    test
)
ALLOWED_SCOPES=(
    d
    ctl
    core
)
MAX_FIRST_LINE_LENGTH=75

function lint_commits_in_range() {
    while read -r rev; do
        lint_commit "$rev"
    done < <(git rev-list "$LINT_COMMITS_RANGE")
}

function lint_commit() {
    local rev="$1"
    local commit_msg=$(git log --format=%B -n 1 "$rev")
    local first_line=$(head -n1 <<<"$commit_msg")

    local first_line_length=$(wc -c <<<"$first_line")
    if [ "$first_line_length" -gt $MAX_FIRST_LINE_LENGTH ]; then
        error_header_first_line "$rev" "$first_line"
        echo "The line has $first_line_length characters, but $MAX_FIRST_LINE_LENGTH is maximum (incl. newline)."
        exit 1
    fi

    # local first_line_regex='^([^\(:]+): (.+)$'
    # The following should work once we support scopes:
    local first_line_regex='^([^\(:]+)(\([^)]+\))?!?: (.+)$'
    if [[ "$first_line" =~ $first_line_regex ]]; then
        local type="${BASH_REMATCH[1]}"
        local scope_parens="${BASH_REMATCH[2]}"
        local scope=${scope_parens/(/}
        scope=${scope/)/}
        if ! printf '%s\n' "${ALLOWED_TYPES[@]}" | grep -qxF "$type"; then
            error_header_first_line "$rev" "$first_line"
            echo "Unrecognized type '$type'. Allowed types: ${ALLOWED_TYPES[*]}"
            exit 1
        fi
        if [ -n "$scope_parens" ] && ! printf '%s\n' "${ALLOWED_SCOPES[@]}" | grep -qxF "$scope"; then
            error_header_first_line "$rev" "$first_line"
            echo "Unrecognized scope '$scope'. Allowed scopes: ${ALLOWED_SCOPES[*]}"
            exit 1
        fi
    else
        error_header_first_line "$rev" "$first_line"
        echo "The first line does not match pattern. Examples of first line formatting:"
        echo "type: summary"
        echo "type!: summary"
        echo "type(scope): summary"
        echo "type(scope)!: summary"
        echo
        echo "Allowed types: ${ALLOWED_TYPES[*]}"
        echo "Allowed scopes: ${ALLOWED_SCOPES[*]}"
        echo "Variants with exclamation mark should be used for breaking changes."
    fi

    local num_lines=$(wc -l <<<"$commit_msg")
    if [ "$num_lines" -gt 1 ]; then
        if [ "$num_lines" -lt 3 ]; then
            error_header_first_line "$rev" "$first_line"
            echo "The message has $num_lines lines. It must have either 1, or 3+."
            exit 1
        fi
        local second_line=$(sed -n 2p <<<"$commit_msg")
        if [ -n "$second_line" ]; then
            error_header_first_line "$rev" "$first_line"
            echo "The 2nd line of commit message must be empty."
            exit 1
        fi
        local third_line=$(sed -n 3p <<<"$commit_msg")
        if [ -z "$third_line" ]; then
            error_header_first_line "$rev" "$first_line"
            echo "The 3rd line of commit message must not be empty."
            exit 1
        fi
    fi
}

function error_header_first_line() {
    echo "ERROR: Commit message of $1 does not follow convention:"
    echo "$2"
    echo
}

lint_commits_in_range "$LINT_COMMITS_RANGE"