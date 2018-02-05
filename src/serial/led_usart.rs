use stm32l151::{DMA1, GPIOA, GPIOB, RCC, USART3};
use super::DmaUsart;

pub struct LedUsart {
    _usart: USART3
}

impl DmaUsart for LedUsart {
    fn is_receive_pending(&self, dma: &DMA1) -> bool {
        dma.isr.read().tcif3().bit_is_set()
    }

    fn receive(&self, dma: &mut DMA1, _gpioa: &mut GPIOA, length: u16, buffer: u32) {
        dma.ifcr.write(|w| w.cgif3().set_bit());
        dma.ccr3.modify(|_, w| { w.en().clear_bit() });
        dma.cmar3.write(|w| unsafe { w.ma().bits(buffer) });
        dma.cndtr3.modify(|_, w| unsafe { w.ndt().bits(length) });
        dma.ccr3.modify(|_, w| { w.en().set_bit() });
    }

    fn is_send_ready(&self, dma: &DMA1) -> bool {
        dma.cndtr2.read().ndt().bits() == 0
    }

    fn send(&self, dma: &mut DMA1, _gpioa: &mut GPIOA, buffer: u32, length: u16) {
        dma.ccr2.modify(|_, w| w.en().clear_bit());
        dma.cmar2.write(|w| unsafe { w.ma().bits(buffer) });
        unsafe { dma.cndtr2.modify(|_, w| w.ndt().bits(length)) };
        dma.ccr2.modify(|_, w| w.en().set_bit());
    }

    fn tx_interrupt(&self, dma: &mut DMA1) {
        dma.ifcr.write(|w| w.cgif2().set_bit());
        dma.ccr2.modify(|_, w| w.en().clear_bit());
    }
}

impl LedUsart {
    pub fn new(usart: USART3, dma: &DMA1, gpiob: &mut GPIOB, rcc: &mut RCC) -> LedUsart {
        gpiob.moder.modify(|_, w| unsafe {
            w.moder10().bits(0b10)
             .moder11().bits(0b10)
        });
        gpiob.pupdr.modify(|_, w| unsafe {
            w.pupdr10().bits(0b01)
             .pupdr11().bits(0b01)
        });
        gpiob.afrh.modify(|_, w| unsafe { w.afrh10().bits(7).afrh11().bits(7) });

        rcc.apb1enr.modify(|_, w| w.usart3en().set_bit());
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

        dma.cpar3.write(|w| unsafe { w.pa().bits(0x4000_4804) });
        //dma.cmar3.write(|w| unsafe { w.ma().bits(receive_ptr) });
        dma.ccr3.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .tcie().set_bit()
        });

        dma.cpar2.write(|w| unsafe { w.pa().bits(0x4000_4804) });
        //dma.cmar2.write(|w| unsafe { w.ma().bits(send_ptr) });
        dma.cndtr2.modify(|_, w| unsafe { w.ndt().bits(0x0) });
        dma.ccr2.modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .dir().set_bit()
             .tcie().set_bit()
             .en().clear_bit()
        });

        LedUsart { _usart: usart }
    }
}
