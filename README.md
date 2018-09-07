Firmware for Anne Pro Keyboard written in Rust
==============================================

[![Travis Build Status](https://travis-ci.org/ah-/anne-key.svg?branch=master)](https://travis-ci.org/ah-/anne-key)

<img src="docs/images/ferris.png" width=30%/> <img src="docs/images/anne.jpg" width=50%/>


This is an alternative firmware for the [Anne Pro Keyboard](http://en.obins.net/anne-pro), with the goal of being more stable than the original firmware and adding extra features.

Status
------

This project is still under heavy development and probably not quite ready yet to serve as your only keyboard.

Working today:

- Basic keyboard functionality
- Bluetooth (as a keyboard)
- LED control (switching on/off, changing themes)
- USB charging
- Drop in replacement as a simple firmware update
- Partial bluetooth communication with the Anne Pro App (tested with [Anne Pro Mac App](https://github.com/msvisser/AnnePro-mac))

Not yet implemented:

- USB (sends keys concurrently with BT, hangs on connect/disconnect)
- Media controls / special keys
- Uploading custom lighting settings
- Uploading custom keymaps
- Power Management
- BT setup mode with LEDs etc.

Community
---------

We hang out in the [Anne Pro Dev discord](https://discord.gg/ygssH9x). Please observe the [Rust Code of Conduct](https://www.rust-lang.org/conduct.html) within our community.

Flashing
--------

You can find the latest build on the [Releases page](https://github.com/ah-/anne-key/releases). Download `anne-key.dfu`.
If LEDs aren't working with latest build, you can try [build 209](https://github.com/ah-/anne-key/releases/tag/2018-04-12-209-master-aee0f1b)
, or the following procedure: reboot into DFU mode by holding down `Fn+Space+Esc`, then exit with `Esc`.

Then you can either follow the [obins firmware update steps](http://en.obins.net/firmware) (click Update manual) or use `dfu-util`.

### dfu-util

First you'll need to [install dfu-util](https://docs.particle.io/faq/particle-tools/installing-dfu-util/core/).

To flash your Anne Pro connect via USB, then hold down the Esc button, press the little reset switch on the back and finally release Esc.

Now your keyboard is in DfuSe mode. It should show up in dfu-util:

```
$ dfu-util -l
dfu-util 0.9

...

Found DFU: [0483:df11] ver=0200, devnum=23, cfg=1, intf=0, path="20-2", alt=2, name="@BluetoothFlash  /0x1c000000/14*256 a,192*256 g", serial="057C37553731"
Found DFU: [0483:df11] ver=0200, devnum=23, cfg=1, intf=0, path="20-2", alt=1, name="@Internal Flash  /0x0c000000/64*256 a,192*256 g", serial="057C37553731"
Found DFU: [0483:df11] ver=0200, devnum=23, cfg=1, intf=0, path="20-2", alt=0, name="@Internal Flash  /0x08000000/64*256 a,192*256 g", serial="057C37553731"
```

Then you can flash your keyboard firmware:

```
$ dfu-util --alt 0 --intf 0 --download anne-key.dfu

...

file contains 1 DFU images
parsing DFU image 1
image for alternate setting 0, (1 elements, total size = 5104)
parsing element 1, address = 0x08004000, size = 5096
Download        [=========================] 100%         5096 bytes
Download done.
done parsing DfuSe file
```

And that's it. Press the reset button again to exit the bootloader and return to normal keyboard mode and you're done!

If your keyboard is running our firmware, you can reboot to DFU mode by holding down `Fn+Space+Escape`.

If you want to return to the original firmware you can flash the [original firmware](http://en.obins.net/firmware) with:

```
$ dfu-util --alt 0 --intf 0 --download "anne pro key 1.4.dfu"
```

Documentation & Hacking
---------

You can find some documentation on hardware on [GitBooks](https://ahah.gitbooks.io/anne-pro-internals/).
Many fellow projects provide insights into the obins firmware and app protocol:

1. Reverse-engineering
- [hi-a's disassembly of the firmware and bootloader](https://hi-a.github.io/annepro-key/) ([repo](https://github.com/hi-a/annepro-key))
- [metr1xx's APK reverse engineering](https://github.com/metr1xx/anne-pro-community-app)

2. Alternate control apps
- [Blucky87's Python CLI](https://github.com/Blucky87/AnneProCLI)
- fcoury's [Node.js library](https://github.com/fcoury/node-anne-pro) and [electron app](https://github.com/fcoury/electron-anne-pro)
- [kprinssu's Windows app](https://github.com/kprinssu/anne-keyboard-windows)
- [msvisser's Mac App](https://github.com/msvisser/AnnePro-mac)

3. Alternate firmware
- qmk ports: [josecostamartins'](https://github.com/josecostamartins/qmk_firmware/commits/anne_pro) and [dwhinham's](https://github.com/dwhinham/qmk_firmware/commits/anne_pro)


To build your own firmware, you need the nightly-2018-09-06 rust
toolchain with the following components:

- rustup: to make use of the `rust-toolchain` file
- thumbv7m std: `rustup target add thumbv7m-none-eabi` within your checkout
- ARM linker: usually named `arm-none-eabi-ld`, please check with your OS

Then, `make dfu` in the top directory will build your `anne-key.dfu`.

To analyze the firmware's code size, you need [cargo-bloat](https://github.com/RazrFalcon/cargo-bloat):

- `cargo install cargo-bloat`
- `make bloat`
- `make bloat BLOAT_ARGS="--crates" # passing arguments to cargo-bloat`

Our CI requires consistent formatting, please run rustfmt before you submit PRs:

- `rustup component add rustfmt-preview`
- `make fmt`

Troubleshooting
---------

### error[E0463]: can't find crate for compiler_builtins

Run the following command before running `make dfu`:

```
rustup target add thumbv7m-none-eabi
```

### error: linker arm-none-eabi-ld not found

You need to install the ARM tools. If you're on macOS run the following:

```
brew install caskroom/cask/gcc-arm-embedded
```
