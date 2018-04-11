all:
	$(MAKE) dfu

build:
	cargo build --release

dfu: build
	./scripts/generate_dfu.sh

debug: build
	arm-none-eabi-gdb target/thumbv7m-none-eabi/release/anne-key

openocd:
	openocd -f openocd.cfg

bloat:
	cargo bloat $(BLOAT_ARGS) -n 50

fmt:
	cargo fmt

clippy:
	cargo clippy

clean:
	cargo clean
	rm -f anne-key.bin
	rm -f anne-key.dfu
	rm -rf _book/

.PHONY: all build clean debug openocd bloat fmt clippy
