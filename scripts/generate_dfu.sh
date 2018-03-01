#!/bin/sh

set -ex

arm-none-eabi-objcopy -O binary target/thumbv7m-none-eabi/release/anne-key anne-key.bin
./scripts/dfu-convert.py -b 0x08004000:anne-key.bin anne-key.dfu
