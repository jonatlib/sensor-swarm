#!/bin/sh

# This script executes the compiled test binary in QEMU

qemu-system-arm \
  -M netduinoplus2 \
  -cpu cortex-m4 \
  -kernel "$1" \
  -nographic \
  -S \
  -gdb tcp::3333 \
  -chardev pty,id=char_semi \
  -semihosting-config chardev=char_semi,target=native \
  -chardev pty,id=char_serial \
  -serial chardev:char_serial \
  -monitor stdio


# This flag starts halted cpu until gdb is connected
#  -S \
