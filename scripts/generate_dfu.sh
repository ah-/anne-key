#!/bin/sh

set -ex

cargo objcopy --release --target thumbv7m-none-eabi --bin anne-key -- -O binary anne-key.bin
./scripts/dfu-convert.py -b 0x08004000:anne-key.bin anne-key.dfu
