#!/bin/bash

cargo build --release
echo "Creating kernel.img..."
llvm-objcopy target/bcm2835/release/gocycling-controller -O binary target/bcm2835/release/kernel.img
