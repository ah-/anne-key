#!/bin/sh

set -eux

main() {
    if [[ ${TRAVIS_OS_NAME} != 'windows' ]]; then
        which cargo-bloat || (cd /; cargo install cargo-bloat)
    else
        choco install make python2
    fi
        which cargo-objcopy || (cd /; cargo install cargo-binutils)
}

main
