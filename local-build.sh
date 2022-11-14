#!/bin/sh
HASH="9d8b773"
DIRNAME="ember-network-connect"
SRC_DIR="/Users/brett/Git/netfiredotnet/ember-network-connect"
BAL_DIR="/Users/brett/Git/netfiredotnet/ember"
BAL_BUILD_DIR="$BAL_DIR/ember-network-connect/testbuild/"
SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
TMP_SRC_DIR="$SCRIPT_DIR/$DIRNAME"
OUT_DIR="$SCRIPT_DIR/out"
# [ ! -d "$SCRIPT_DIR/$DIRNAME" ] && cd "$SCRIPT_DIR" && git clone https://github.com/balena-os/wifi-connect.git "$DIRNAME"

# If using github
# [ ! -d "$SCRIPT_DIR/$DIRNAME" ] && cd "$SCRIPT_DIR" && git clone https://github.com/netfiredotnet/ember-network-connect.git "$DIRNAME"
# # cd "$SCRIPT_DIR/$DIRNAME" && git checkout "$HASH"
# cd "$SCRIPT_DIR/$DIRNAME" && git fetch
# cd "$SCRIPT_DIR/$DIRNAME" && git reset --hard "origin/brett-test"

# If using local
  rm -rf "$OUT_DIR" \
  && cd "$SCRIPT_DIR" \
  && DOCKER_BUILDKIT=1 docker build --output "type=local,dest=$OUT_DIR" .