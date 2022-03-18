#!/bin/bash

set -euxo pipefail

touch /.waketimed-toolbox
chmod 0444 /.waketimed-toolbox

### PACKAGES ###

dnf clean all
dnf -y update
dnf -y install \
    clippy \
    git \
    make \
    rustfmt \

dnf clean all
