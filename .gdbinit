set history save on
add-auto-load-safe-path /home/andreas/.rustup/toolchains
target remote :3333
set print asm-demangle on
monitor arm semihosting enable
load
c
