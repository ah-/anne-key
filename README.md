Firmware for Anne Pro Keyboard written in Rust
==============================================

[![Travis Build Status](https://travis-ci.org/ah-/anne-key.svg?branch=master)](https://travis-ci.org/ah-/anne-key)


Flashing
--------

You can find the latest build on the [Releases page](https://github.com/ah-/anne-key/releases). Download `anne-key.dfu`.

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

If you want to return to the original firmware you can flash the [original firmware](http://en.obins.net/firmware) similarly with `dfu-util --alt 0 --intf 0 --download "anne pro key 1.4.dfu"`.
