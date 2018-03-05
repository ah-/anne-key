XARGO = xargo

all:
	$(MAKE) dfu

build:
	$(XARGO) build --release --target thumbv7m-none-eabi

dfu: build
	./scripts/generate_dfu.sh

debug: build
	arm-none-eabi-gdb target/thumbv7m-none-eabi/release/anne-led

clean:
	$(XARGO) clean
	rm -f anne-key.bin
	rm -f anne-key.dfu
	rm -rf book/

.PHONY: all build clean debug
