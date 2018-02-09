use embedded_hal::digital::OutputPin;
use stm32l151::{RCC, USART2};
use hal::dma::dma1::{C6, C7};
use hal::gpio::{Alternate, Input, Output};
use hal::gpio::gpioa::{PA1, PA2, PA3};

use super::DmaUsart;

pub struct BluetoothUsart {
    pa1: PA1<Output>,
    _pa2: PA2<Alternate>,
    _pa3: PA3<Alternate>,
    _usart: USART2,
    dma_rx: C6,
    dma_tx: C7,
}

impl DmaUsart for BluetoothUsart {
    fn is_receive_pending(&mut self) -> bool {
        self.dma_rx.tcif()
    }

    fn receive(&mut self, length: u16, buffer: u32) {
        // wakeup complete, reset pa1
        self.pa1.set_low();

        self.dma_rx.cgif();
        self.dma_rx.ccr().modify(|_, w| { w.en().clear_bit() });
        self.dma_rx.cmar().write(|w| unsafe { w.ma().bits(buffer) });
        self.dma_rx.cndtr().modify(|_, w| unsafe { w.ndt().bits(length) });
        self.dma_rx.ccr().modify(|_, w| { w.en().set_bit() });
    }

    fn is_send_ready(&mut self) -> bool {
        self.dma_tx.cndtr().read().ndt().bits() == 0
    }

    fn send(&mut self, buffer: u32, _len: u16) {
        // Don't actually send anything yet, just enqueue and wait for wakeup package
        self.dma_rx.ccr().modify(|_, w| { w.en().clear_bit() });
        self.dma_rx.cndtr().modify(|_, w| unsafe { w.ndt().bits(2) });
        self.dma_rx.ccr().modify(|_, w| { w.en().set_bit() });

        self.dma_tx.cmar().write(|w| unsafe { w.ma().bits(buffer) });

        self.pa1.set_low();
        self.pa1.set_high();
    }

    fn ack_wakeup(&mut self) {
        // TODO: correct length, not just hardcoded 0xb
        self.dma_tx.cndtr().modify(|_, w| unsafe { w.ndt().bits(0xb) });
        self.dma_tx.ccr().modify(|_, w| w.en().set_bit());
    }

    fn tx_interrupt(&mut self) {
        self.dma_tx.cgif();
        self.dma_tx.ccr().modify(|_, w| w.en().clear_bit());
    }
}

impl BluetoothUsart {
    pub fn new(usart: USART2, pa1: PA1<Input>, pa2: PA2<Input>, pa3: PA3<Input>,
               mut dma_rx: C6, mut dma_tx: C7, rcc: &mut RCC) -> BluetoothUsart {
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

        dma_rx.cpar().write(|w| unsafe { w.pa().bits(0x4000_4404) });
        dma_rx.ccr().modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .tcie().set_bit()
        });

        dma_tx.cpar().write(|w| unsafe { w.pa().bits(0x4000_4404) });
        dma_tx.cndtr().modify(|_, w| unsafe { w.ndt().bits(0x0) });
        dma_tx.ccr().modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit()
             .dir().set_bit()
             .tcie().set_bit()
             .en().clear_bit()
        });

        BluetoothUsart { pa1: pa1, _pa2: pa2, _pa3: pa3, _usart: usart, dma_rx: dma_rx, dma_tx: dma_tx  }
    }
}
