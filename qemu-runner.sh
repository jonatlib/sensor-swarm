#!/bin/sh

# This script executes the compiled test binary in QEMU

qemu-system-arm \
  -M netduinoplus2 \
  -cpu cortex-m4 \
  -kernel "$1" \
  -nographic \
  -gdb tcp::3333 \
  -S \
  -chardev stdio,id=char0 \
  -semihosting-config chardev=char0 \
  -serial stdio \
  -monitor /dev/null
