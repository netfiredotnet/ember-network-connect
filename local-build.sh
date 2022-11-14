#!/bin/sh

# This script utilizes docker on an aarch64 host (currently using M1 mac) to build everything locally

SCRIPT_DIR=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
OUT_DIR="$SCRIPT_DIR/out"
rm -rf "$OUT_DIR" \
  && cd "$SCRIPT_DIR" \
  && DOCKER_BUILDKIT=1 docker build --output "type=local,dest=$OUT_DIR" . \
  && cd "$OUT_DIR" \
  && tar -xvzf "build.tar.gz" \
  && rm -f "build.tar.gz" \
  && cd "$SCRIPT_DIR/ui" \
  && npm install \
  && npm run build \
  && cp -r "$SCRIPT_DIR/ui/build/" "$OUT_DIR/ui/"