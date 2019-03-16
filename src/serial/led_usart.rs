use crate::serial::DmaUsart;
use hal::dma::dma1::{C2, C3};
use hal::gpio::gpiob::{PB10, PB11};
use hal::gpio::{Alternate, Input};
use stm32l1::stm32l151::{RCC, USART3};

pub struct LedUsart {
    _pb10: PB10<Alternate>,
    _pb11: PB11<Alternate>,
    _usart: USART3,
    dma_rx: C3,
    dma_tx: C2,
}

impl DmaUsart for LedUsart {
    fn is_receive_pending(&mut self) -> bool {
        self.dma_rx.tcif()
    }

    fn receive(&mut self, length: u16, buffer: u32) {
        self.dma_rx.cgif();
        self.dma_rx.ccr().modify(|_, w| w.en().clear_bit());
        self.dma_rx.cmar().write(|w| unsafe { w.ma().bits(buffer) });
        self.dma_rx
            .cndtr()
            .modify(|_, w| unsafe { w.ndt().bits(length) });
        self.dma_rx.ccr().modify(|_, w| w.en().set_bit());
    }

    fn is_send_ready(&mut self) -> bool {
        self.dma_tx.cndtr().read().ndt().bits() == 0
    }

    fn send(&mut self, buffer: u32, length: u16) {
        self.dma_tx.ccr().modify(|_, w| w.en().clear_bit());
        self.dma_tx.cmar().write(|w| unsafe { w.ma().bits(buffer) });
        self.dma_tx
            .cndtr()
            .modify(|_, w| unsafe { w.ndt().bits(length) });
        self.dma_tx.ccr().modify(|_, w| w.en().set_bit());
    }

    fn ack_wakeup(&mut self) {}

    fn tx_interrupt(&mut self) {
        self.dma_tx.cgif();
        self.dma_tx.ccr().modify(|_, w| w.en().clear_bit());
    }
}

impl LedUsart {
    pub fn new(
        usart: USART3,
        pb10: PB10<Input>,
        pb11: PB11<Input>,
        mut dma_rx: C3,
        mut dma_tx: C2,
        rcc: &mut RCC,
    ) -> LedUsart {
        let pb10 = pb10.into_alternate(7).pull_up();
        let pb11 = pb11.into_alternate(7).pull_up();

        rcc.apb1enr.modify(|_, w| w.usart3en().set_bit());
        rcc.ahbenr.modify(|_, w| w.dma1en().set_bit());

        usart.brr.modify(|_, w| unsafe { w.bits(417) });
        usart.cr3.modify(|_, w| w.dmat().set_bit().dmar().set_bit());
        usart.cr1.modify(|_, w| {
            w.rxneie()
                .clear_bit()
                .re()
                .set_bit()
                .te()
                .set_bit()
                .ue()
                .set_bit()
                .idleie()
                .clear_bit()
                .txeie()
                .clear_bit()
                .tcie()
                .clear_bit()
        });

        dma_rx.cpar().write(|w| unsafe { w.pa().bits(0x4000_4804) });
        dma_rx.ccr().modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc().set_bit().tcie().set_bit()
        });

        dma_tx.cpar().write(|w| unsafe { w.pa().bits(0x4000_4804) });
        dma_tx.cndtr().modify(|_, w| unsafe { w.ndt().bits(0x0) });
        dma_tx.ccr().modify(|_, w| {
            unsafe {
                w.pl().bits(2);
            }
            w.minc()
                .set_bit()
                .dir()
                .set_bit()
                .tcie()
                .set_bit()
                .en()
                .clear_bit()
        });

        LedUsart {
            _pb10: pb10,
            _pb11: pb11,
            _usart: usart,
            dma_rx,
            dma_tx,
        }
    }
}
