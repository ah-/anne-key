Software
========

Bootloader
----------

Usually an STM32 starts execution via the vector table at `0x0800_0000`, but our user code only starts at `0x0800_4000` as you can see in the `memory.x` file.

This is due to the factory bootloader that lives at `0x0800_0000`. The bootloader checks on startup if the escape key is pressed, and if so it puts the Anne Pro into DFU mode. Otherwise it jumps to our code at `0x0800_4000`.

The bootloader on the main Keyboard STM32 also handles flashing the LED controller. If you're flashing LED, the commands are first sent to the keyboard controller, which forwards them.

It also advertises support for flashing the Bluetooth chip, but this seems to be broken.

### Memory Map
```
Address

             .......................... 
             .                        . 
             .          ...           .
             .                        .
             +------------------------+
             |                        |
             |          RAM           |
             |                        |
0x2000_0000  +------------------------+
             .                        .
             .          ...           .
             .                        .
             .                        .
             +------------------------+
             |                        |
             |         FLASH          |
             |    (Rust lives here)   |
             |                        |
0x0800_4000  +------------------------+
             |       BOOTLOADER       |
0x0800_0000  +------------------------+
             .                        .
             .          ...           .
             .                        .
0x0000_0000  ..........................
```
