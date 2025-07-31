#!/bin/sh

# This script executes the compiled test binary in QEMU
set -e

qemu-system-arm \
  -M netduinoplus2 \
  -cpu cortex-m4 \
  -kernel "${1}" \
  -nographic \
  -gdb tcp::3333 \
  -semihosting \
  \
  -chardev stdio,id=char_semi \
  -semihosting-config chardev=char_semi,target=native \
  \
  -chardev socket,id=char_serial,host=127.0.0.1,port=8889,server=on,wait=off \
  -serial chardev:char_serial \
  \
  -monitor /dev/null | defmt-print -e "${1}"


# This flag starts halted cpu until gdb is connected
#  -S \

# Old config, where semihosing and serial went to different places (not worked)
# and monitor was enable
#   -chardev pty,id=char_semi \
#  -semihosting-config chardev=char_semi,target=native \
#  -chardev pty,id=char_serial \
#  -serial chardev:char_serial \
#  -monitor stdio
