set history save on
target remote :3333
set print asm-demangle on
monitor arm semihosting enable
load
#b HAL_PCD_Start
#b PCD_EP_ISR_Handler
#b Src/hal_pcd.c:170
#b foo
#b USBD_LL_DataInStage
#b PCD_EP_ISR_Handler
#b USBD_LL_Reset
#b anne_key::usb_ctr
#b main.rs:187
#b main.rs:330
#b main.rs:423
c
