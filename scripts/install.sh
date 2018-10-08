#!/bin/sh

set -eux

main() {
        which cargo-bloat || (cd /; cargo install cargo-bloat)
        which cargo-objcopy || (cd /; cargo install cargo-binutils)
}

main
