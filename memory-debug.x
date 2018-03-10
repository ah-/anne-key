/* fake larger flash so debug builds succeed
   can't actually use these on the stm32, but they
   allow cargo bloat and clippy to run cleanly */

MEMORY
{
  FLASH : ORIGIN = 0x08004000, LENGTH = 4M
  RAM : ORIGIN = 0x20000000, LENGTH = 4M
}
