#!/bin/bash

DIR=$(dirname $(realpath $0))

cp "$DIR/git-hook-commit-msg" "$DIR/../.git/hooks/commit-msg"
chmod a+x "$DIR/../.git/hooks/commit-msg"
