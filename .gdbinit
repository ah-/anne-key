set history save on
add-auto-load-safe-path ~/.rustup/toolchains
target extended-remote :3333
set print asm-demangle on
monitor arm semihosting enable
load
c
