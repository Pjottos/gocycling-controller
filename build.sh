#!/bin/bash

TARGET_DIR=target/thumbv6m-none-eabi/release

cargo build --release
echo "Creating uf2..."
elf2uf2 $TARGET_DIR/gocycling-controller $TARGET_DIR/gocycling-controller.uf2
