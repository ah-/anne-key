all:
	$(MAKE) dfu

build:
	rustup component add llvm-tools-preview
	rustup target add thumbv7m-none-eabi
	cargo build --release

build-semihosting:
	rustup component add llvm-tools-preview
	rustup target add thumbv7m-none-eabi
	cargo build --release --features use_semihosting

dfu: build
	./scripts/generate_dfu.sh
	ls -l anne-key.dfu

debug: build-semihosting
	arm-none-eabi-gdb target/thumbv7m-none-eabi/release/anne-key

openocd:
	openocd -f openocd.cfg

bloat:
	cargo bloat $(BLOAT_ARGS) -n 50 --target thumbv7m-none-eabi

fmt:
	rustup component add rustfmt
	cargo fmt

test:
	cd tests; cargo test --target $(shell rustup target list | grep default | cut -d ' ' -f 1)

clippy:
	rustup component add clippy
	cargo clippy

clean:
	cargo clean
	rm -f anne-key.bin
	rm -f anne-key.dfu
	rm -rf _book/

.PHONY: all build clean debug openocd bloat fmt clippy test
