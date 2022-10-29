#!/bin/bash
set -euo pipefail

PROJECT_DIR=$(realpath "$(dirname $(realpath "$0"))/..")
WAKETIMED_BUILD_PROFILE=${WAKETIMED_BUILD_PROFILE:-aarch64-unknown-linux-gnu/release}
BIN_DIR=/opt/waketimed-devel/bin
CONFIG_DIR=/opt/waketimed-devel/config
SERVICE_PATH=/etc/systemd/system/waketimed-devel.service

if [ -z "${WAKETIMED_DEVICE_SSH:-}" ]; then
    echo "Set WAKETIMED_DEVICE_SSH."
    exit 1
fi

START=0
if ssh $WAKETIMED_DEVICE_SSH systemctl is-active waketimed-devel.service; then
    START=1
fi
ssh $WAKETIMED_DEVICE_SSH systemctl stop waketimed-devel || true
ssh $WAKETIMED_DEVICE_SSH mkdir -p "$BIN_DIR"
ssh $WAKETIMED_DEVICE_SSH mkdir -p "$CONFIG_DIR"
scp $PROJECT_DIR/target/$WAKETIMED_BUILD_PROFILE/waketimed $WAKETIMED_DEVICE_SSH:$BIN_DIR/waketimed
if [ "${WAKETIMED_PUSH_CONFIG:-1}" = 1 ]; then
    scp $PROJECT_DIR/waketimed/data/devel/config.yaml $WAKETIMED_DEVICE_SSH:$CONFIG_DIR/config.yaml
fi
scp $PROJECT_DIR/waketimed/data/devel/waketimed-devel.service $WAKETIMED_DEVICE_SSH:$SERVICE_PATH
rsync -av --delete $PROJECT_DIR/waketimed/data/dist/ $WAKETIMED_DEVICE_SSH:$DIST_DIR

ssh $WAKETIMED_DEVICE_SSH chmod -R u=rwx,g=rX,o=rX "$BIN_DIR"
ssh $WAKETIMED_DEVICE_SSH chmod -R u=rwX,g=rX,o=rX "$CONFIG_DIR"

ssh $WAKETIMED_DEVICE_SSH systemctl daemon-reload
if [ "$START" = 1 ]; then
    ssh $WAKETIMED_DEVICE_SSH systemctl start waketimed-devel
fi
