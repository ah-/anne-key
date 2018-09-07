#!/bin/sh

set -eux

main() {
        rustup component list | grep 'thumbv7m.*installed' || \
            rustup target add thumbv7m-none-eabi

        rustup component list | grep 'rustfmt.*installed' || \
            rustup component add rustfmt-preview

        which cargo-bloat || (cd /; cargo install cargo-bloat)

        if [ ${TRAVIS_OS_NAME} != 'osx' ]; then
            mkdir binutils
            curl -L https://www.archlinux.org/packages/community/x86_64/arm-none-eabi-binutils/download/ | tar --strip-components=1 -C binutils -x
        fi
}

main
