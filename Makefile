XARGO = xargo

all:
	$(MAKE) dfu

build:
	$(XARGO) build --release --target thumbv7m-none-eabi

dfu: build
	./scripts/generate_dfu.sh

docs:
	mdbook build

serve:
	mdbook serve

clean:
	$(XARGO) clean
	rm -f anne-key.bin
	rm -f anne-key.dfu
	rm -rf book/

.PHONY: all build clean docs serve
