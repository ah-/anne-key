Hardware
========

![Anne](images/anne-white.jpg)

The Anne Pro contains 3 MCUs to handle each of the Keyboard, LED and BLE operations:

- Keyboard: STM32L151C8T6
- LED: STM32L151C8T6
- BLE: TI CC2541

The USB charging circuit is a BQ24075.

You can find some more information including pin assignments at https://github.com/hi-a/annepro-key

Flashing and debugging
----------------------

To develop it's best to directly flash via the debug pins instead of DFU, so you get full debugging support and even working stdout to your host machine.

Any ST-Link v2 programmer will do. You can find them cheap on ebay, or if you already have a STM32 Nucleo board you can use the programmer of that.

All the debug pins are exposed and easily accessible. Just solder on some wires and connect to your programmer:

![PCB](images/pcb.png)

OpenOCD
-------

Once your programmer is connected start `openocd -f anne.cfg`.
