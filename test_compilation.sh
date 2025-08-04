#!/bin/bash
echo "Testing compilation before USB cleanup..."
cargo check --target thumbv7em-none-eabihf
if [ $? -eq 0 ]; then
    echo "✓ Compilation successful"
else
    echo "✗ Compilation failed"
    exit 1
fi