#!/bin/bash

set -euxo pipefail

touch /.waketimed-toolbox
chmod 0444 /.waketimed-toolbox

### UPDATE ###

apt-get clean
apt update -y
apt upgrade -y


### CROSS-COMPILATION ###

mkdir ~/.cargo
cat <<EOF >~/.cargo/config
[target.aarch64-unknown-linux-gnu]
linker = "/usr/bin/aarch64-linux-gnu-gcc"
EOF
dpkg --add-architecture arm64
apt update -y
apt install -y \
    libstd-rust-dev:arm64 \
    gcc-aarch64-linux-gnu \


### PACKAGES ###

apt install -y \
    cargo \
    curl \
    gcc \
    gdb \
    make \


### CLEANUP ###

apt-get clean
rm -rf $HOME/.cargo/registry
