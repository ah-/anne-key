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

LED chip
--------

Here are the themes defined in obins' LED firmware:

| Index | Description                                                      |
|-------|------------------------------------------------------------------|
| 0     | All keys off                                                     |
| 1     | All keys red                                                     |
| 2     | All keys yellow                                                  |
| 3     | All keys green                                                   |
| 4     | All keys cyan                                                    |
| 5     | All keys blue                                                    |
| 6     | All keys purple                                                  |
| 7     | All keys pink                                                    |
| 8     | All keys orange                                                  |
| 9     | All keys white                                                   |
| 10    | France's flag                                                    |
| 11    | Italia's flag                                                    |
| 12    | Argentina's flag                                                 |
| 13    | Breathing cycle red -> yellow -> green -> cyan -> blue -> purple |
| 14    | Rolling colors                                                   |
| 15    | Pressed keys fading out                                          |
| 16    | Pressed keys light up, cycle colors after each press             |
| 17    | Pressed keys' row and column radiate outwards                    |
| 18    | All keys light up, cycle colors                                  |
