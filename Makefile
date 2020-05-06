GDB ?= arm-none-eabi-gdb

all:
	$(MAKE) dfu

build-semihosting:
	rustup component add llvm-tools-preview
	rustup target add thumbv7m-none-eabi
	cargo build --release --features use_semihosting

dfu:
	./scripts/generate_dfu.sh
	ls -l anne-key.dfu

debug: build-semihosting
	$(GDB) -x openocd.gdb target/thumbv7m-none-eabi/release/anne-key

gui-debug: build-semihosting
	gdbgui --gdb $(GDB) --gdb-args "-x openocd.gdb" target/thumbv7m-none-eabi/release/anne-key

bloat:
	cargo bloat $(BLOAT_ARGS) -n 50 --target thumbv7m-none-eabi

fmt:
	rustup component add rustfmt
	cargo fmt

clippy:
	rustup component add clippy
	cargo clippy

clean:
	cargo clean
	rm -f anne-key.bin
	rm -f anne-key.dfu
	rm -rf _book/

.PHONY: all build clean debug openocd bloat fmt clippy
