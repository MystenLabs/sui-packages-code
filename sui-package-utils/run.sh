#!/bin/bash
set -x

mkdir -p out
rm -rf out/*
mkdir -p out/packages
SUI_PACKAGES_ROOT=~/ML/sui-packages
MAX_CHECKPOINT_BEFORE=$(jq -r .max_checkpoint_seen $SUI_PACKAGES_ROOT/action_helper.json)
RUST_BACKTRACE=1
RUST_LIB_BACKTRACE=1

rm -f Cargo.lock
cargo run --bin sui-packages-graphql-poller -- \
      --move-decompiler-path ~/ML/sui-packages/third-party/move-decompiler/move-decompiler-linux-x86_64 \
      --initial-checkpoint "$MAX_CHECKPOINT_BEFORE" \
      --packages-dir "out/packages" \
      --max-checkpoint-seen-file "out/action_helper.json"

      
