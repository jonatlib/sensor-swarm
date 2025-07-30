#!/bin/sh

# This script executes the compiled test binary in QEMU

qemu-system-arm \
  -M netduinoplus2 \
  -cpu cortex-m4 \
  -kernel "$1" \
  -nographic \
  -serial stdio \
  -monitor /dev/null
