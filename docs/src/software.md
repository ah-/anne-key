Software
========

Bootloader
----------

Usually an STM32 starts execution via the vector table at `0x0800_0000`, but our user code only starts at `0x0800_4000` as you can see in the `memory-release.x` file.

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

Obins firmware
--------------

The AP1 has two lines of firmware:

- Official: the last version is beta 1.40.02, released on [obins'
  website](http://en.obins.net/firmware#2)
- Preview: Obins provides a new, experimental firmware based on the
  AP2 framework, with obinslab Starter integration, MagicFn, and other
  features from AP2.

  + [v0.12, from u/obins\_Tony](
   https://www.reddit.com/r/AnnePro/comments/9av9ot/preview_firmware_for_anne_pro_1based_on_the_new/)
  + [v0.13, via u/moonlight1563](
   https://www.reddit.com/r/AnnePro/comments/aqy2fe/ap1_new_013_firmware_adds_obinslab_backlight/)

# Chip-to-Chip protocols

## Bootloader-to-key

As stated above, the bootloader handles starting DFU-mode and
flashing. We will add instruction to dump and restore bootloader, in
case something overwrites it.

## Message format

```text
| Message Type (byte) | Data Length + 1 (byte) | Operation (byte) | Data (`length` bytes) + NULL |
```

The buffers for communication will be at most `1 (type) + 1 (length) + 1 (op) + 255 (data) + 1 (NULL) = 259 bytes` each.

| Type  | Value |
|-------|-------|
| Error | 1     |
| BLE   | 6     |
| LED   | 9     |

## key-to-BT

The BT chip also remembers BLE-mode for each saved host.

| Operation        | Value | Data                                                                                                                                                                                                                                                        |
|------------------|-------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| HostListQuery    | 6     | Ask the BT chip for host status                                                                                                                                                                                                                             |
| AckHostListQuery | 134   | <ol><li>saved\_hosts (byte): bitfield indicating whether the BLE chip stores a host in this slot (1 to 4)<li> connected_host (byte): integer representing the current host (1 to 4), or unsaved host (12), or disconnected (0) <li> BT mode (BLE or legacy) |
| LegacyMode       | 12    | Tell the BT chip to switch to BLE (0) or Legacy (1)                                                                                                                                                                                                         |
| AckLegacyMode    | 140   | (no data)                                                                                                                                                                                                                                                   |


## key-to-LED

| Operation | Value | Data                                                                             |
|-----------|-------|----------------------------------------------------------------------------------|
| ThemeMode | 1     | Value (0 to 18) from the theme table below                                       |
| ConfigCmd | 5     | 3 bytes, to ask for the next theme, animation speed, or brightness in that order |

LED chip
--------

Here are the themes defined in obins' LED firmware:

| Value | Description                                                      |
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
