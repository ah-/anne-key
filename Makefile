XARGO = xargo

all:
	$(MAKE) dfu

build:
	$(XARGO) build --release --target thumbv7m-none-eabi

dfu: build
	./scripts/generate_dfu.sh

clean:
	$(XARGO) clean
	rm -f anne-key.bin
	rm -f anne-key.dfu

.PHONY: all build clean
