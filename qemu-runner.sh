#!/bin/sh

# This script executes the compiled test binary in QEMU

qemu-system-arm \
  -M netduinoplus2 \
  -cpu cortex-m4 \
  -kernel "$1" \
  -nographic \
  -gdb tcp::3333 \
  -S \
  -serial stdio \
  -monitor /dev/null
