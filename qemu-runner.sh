#!/bin/sh

# This script executes the compiled test binary in QEMU

qemu-system-arm \
  -M netduinoplus2 \
  -cpu cortex-m4 \
  -kernel "$1" \
  -nographic \
  -gdb tcp::3333 \
  -semihosting \
  \
  # --- Semihosting is now directed to stdio ---
  # This makes the raw defmt bytes the main output of this script.
  -chardev stdio,id=char_semi \
  -semihosting-config chardev=char_semi,target=native \
  \
  # --- Serial is still on a socket to keep it separate ---
  -chardev socket,id=char_serial,host=127.0.0.1,port=8889,server=on,wait=off \
  -serial chardev:char_serial \
  \
  # --- Monitor is discarded ---
  -monitor /dev/null


# This flag starts halted cpu until gdb is connected
#  -S \

# Old config, where semihosing and serial went to different places (not worked)
# and monitor was enable
#   -chardev pty,id=char_semi \
#  -semihosting-config chardev=char_semi,target=native \
#  -chardev pty,id=char_serial \
#  -serial chardev:char_serial \
#  -monitor stdio
