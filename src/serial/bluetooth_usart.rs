use embedded_hal::digital::OutputPin;
use stm32l151::{DMA1, RCC, USART2};
use hal::gpio::{Alternate, Input, Output};
use hal::gpio::gpioa::{PA1, PA2, PA3};

use super::DmaUsart;

pub struct BluetoothUsart {
    pa1: PA1<Output>,
    _pa2: PA2<Alternate>,
    _pa3: PA3<Alternate>,
    _usart: USART2,
}

impl DmaUsart for BluetoothUsart {
    fn is_receive_pending(&self, dma: &DMA1) -> bool {
        dma.isr.read().tcif6().bit_is_set()
    }

    fn receive(&mut self, dma: &mut DMA1, length: u16, buffer: u32) {
        // wakeup complete, reset pa1
        self.pa1.set_low();

        dma.ifcr.write(|w| w.cgif6().set_bit());
        dma.ccr6.modify(|_, w| { w.en().clear_bit() });
        dma.cmar6.write(|w| unsafe { w.ma().bits(buffer) });
        dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(length) });
        dma.ccr6.modify(|_, w| { w.en().set_bit() });
    }

    fn is_send_ready(&self, dma: &DMA1) -> bool {
        dma.cndtr7.read().ndt().bits() == 0
    }

    fn send(&mut self, dma: &mut DMA1, buffer: u32, _len: u16) {
        // Don't actually send anything yet, just enqueue and wait for wakeup package
        dma.ccr6.modify(|_, w| { w.en().clear_bit() });
        //dma.cmar6.write(|w| unsafe { w.ma().bits(self.receive_buffer.as_mut_ptr() as u32) });
        dma.cndtr6.modify(|_, w| unsafe { w.ndt().bits(2) });
        dma.ccr6.modify(|_, w| { w.en().set_bit() });

        dma.cmar7.write(|w| unsafe { w.ma().bits(buffer) });

        self.pa1.set_low();
        self.pa1.set_high();
    }

    fn tx_interrupt(&self, dma: &mut DMA1) {
        dma.ifcr.write(|w| w.cgif7().set_bit());
        dma.ccr7.modify(|_, w| w.en().clear_bit());
    }
}

impl BluetoothUsart {
    pub fn new(usart: USART2, pa1: PA1<Input>, pa2: PA2<Input>, pa3: PA3<Input>,
               dma: &DMA1, rcc: &mut RCC) -> BluetoothUsart {
        let mut pa1 = pa1.into_output().pull_up();
        let pa2 = pa2.into_alternate(7).pull_up();
        let pa3 = pa3.into_alternate(7).pull_up();
        pa1.set_low();

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

        BluetoothUsart { pa1: pa1, _pa2: pa2, _pa3: pa3, _usart: usart }
    }
}
