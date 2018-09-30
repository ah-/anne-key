all:
	$(MAKE) dfu

build:
	rustup target add thumbv7m-none-eabi
	cargo build --release

dfu: build
	./scripts/generate_dfu.sh
	ls -l anne-key.dfu

debug: build
	arm-none-eabi-gdb target/thumbv7m-none-eabi/release/anne-key

openocd:
	openocd -f openocd.cfg

bloat:
	cargo bloat $(BLOAT_ARGS) -n 50 --target thumbv7m-none-eabi

fmt:
	rustup component add rustfmt-preview
	cargo fmt

clippy:
	rustup component add clippy-preview
	cargo clippy

clean:
	cargo clean
	rm -f anne-key.bin
	rm -f anne-key.dfu
	rm -rf _book/

.PHONY: all build clean debug openocd bloat fmt clippy
