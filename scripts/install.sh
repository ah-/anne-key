#!/bin/sh

set -eux

main() {
        rustup component list | grep 'thumbv7m.*installed' || \
            rustup target add thumbv7m-none-eabi

        rustup component list | grep 'rustfmt.*installed' || \
            rustup component add rustfmt-preview

        which cargo-bloat || cargo install cargo-bloat
}

main
