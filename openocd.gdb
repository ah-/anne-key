set history save on
add-auto-load-safe-path ~/.rustup/toolchains
target remote | openocd -c "gdb_port pipe; log_output openocd.log" -f openocd.cfg
set print asm-demangle on
monitor arm semihosting enable

# detect unhandled exceptions, hard faults and panics
break DefaultHandler
break HardFault
break rust_begin_unwind

load
c
