use stm32l151::{DMA1, GPIOA, RCC, USART2};
use super::DmaUsart;

pub struct BluetoothUsart {
    _usart: USART2
}

impl DmaUsart for BluetoothUsart {

    fn is_receive_pending(&self, dma: &DMA1) -> bool {
        dma.isr.read().tcif6().bit_is_set()
    }

    fn receive(&self, dma: &mut DMA1, gpioa: &mut GPIOA, length: u16, buffer: u32) {
        // wakeup complete, reset pa1
        gpioa.bsrr.write(|w| w.br1().set_bit());

        dma.ifcr.write(|w| w.cgif6().set_bit());
        dma.ccr6.modify(|_, w| { w.en().clear_bit() });
        dma.cmar6.write(|w| unsafe { w.ma().bits(buffer) });
        dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(length) });
        dma.ccr6.modify(|_, w| { w.en().set_bit() });
    }

    fn is_send_ready(&self, dma: &DMA1) -> bool {
        dma.cndtr7.read().ndt().bits() == 0
    }

    fn send(&self, dma: &mut DMA1, gpioa: &mut GPIOA, buffer: u32, _len: u16) {
        // Don't actually send anything yet, just enqueue and wait for wakeup package
        dma.ccr6.modify(|_, w| { w.en().clear_bit() });
        //dma.cmar6.write(|w| unsafe { w.ma().bits(self.receive_buffer.as_mut_ptr() as u32) });
        dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(2) });
        dma.ccr6.modify(|_, w| { w.en().set_bit() });

        dma.cmar7.write(|w| unsafe { w.ma().bits(buffer) });

        gpioa.odr.modify(|_, w| w.odr1().clear_bit());
        gpioa.odr.modify(|_, w| w.odr1().set_bit());
    }

    fn tx_interrupt(&self, dma: &mut DMA1) {
        dma.ifcr.write(|w| w.cgif7().set_bit());
        dma.ccr7.modify(|_, w| w.en().clear_bit());
    }
}

impl BluetoothUsart {
    pub fn new(usart: USART2, dma: &DMA1, gpioa: &mut GPIOA, rcc: &mut RCC) -> BluetoothUsart {
        gpioa.moder.modify(|_, w| unsafe {
            w.moder1().bits(1)
             .moder2().bits(0b10)
             .moder3().bits(0b10)
        });
        gpioa.pupdr.modify(|_, w| unsafe {
            w.pupdr1().bits(0b01)
             .pupdr2().bits(0b01)
             .pupdr3().bits(0b01)
        });
        gpioa.afrl.modify(|_, w| unsafe { w.afrl2().bits(7).afrl3().bits(7) });
        gpioa.odr.modify(|_, w| w.odr1().clear_bit());

        rcc.apb1enr.modify(|_, w| w.usart2en().set_bit());
        rcc.ahbenr.modify(|_, w| w.dma1en().set_bit());

        usart.brr.modify(|_, w| unsafe { w.bits(417) });
        usart.cr3.modify(|_, w| w.dmat().set_bit()
                                      .dmar().set_bit());
        usart.cr1.modify(|_, w| {
            w.rxneie().set_bit()
             .re().set_bit()
             .te().set_bit()
             .ue().set_bit()
        });

        dma.cpar6.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        //dma.cmar6.write(|w| unsafe { w.ma().bits(receive_ptr) });
        dma.ccr6.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .tcie().set_bit()
        });

        dma.cpar7.write(|w| unsafe { w.pa().bits(0x4000_4404) });
        //dma.cmar7.write(|w| unsafe { w.ma().bits(send_ptr) });
        dma.cndtr7.modify(|_, w| unsafe { w.ndt().bits(0x0) });
        dma.ccr7.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .dir().set_bit()
             .tcie().set_bit()
             .en().clear_bit()
        });

        BluetoothUsart { _usart: usart }
    }
}
