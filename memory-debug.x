/* Fake larger flash so debug builds succeed.
   We can't actually fit these on the stm32, but they
   allow cargo bloat and clippy to run. */

MEMORY
{
  FLASH : ORIGIN = 0x08004000, LENGTH = 4M
  RAM : ORIGIN = 0x20000000, LENGTH = 4M
}
