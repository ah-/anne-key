#![feature(const_fn)]

use stm32l151::{GPIOA, GPIOB, SYST};

const ROWS: usize = 5;
const COLUMNS: usize = 14;

pub struct Keyboard {
    /// Stores the currently pressed down keys from last sample.
    pub state: [bool; ROWS * COLUMNS],
}

impl Keyboard {
    pub const fn new() -> Keyboard {
        Keyboard {
            state: [false; ROWS * COLUMNS],
        }
    }

    pub fn init(&self, p: &super::init::Peripherals) {
        p.GPIOB.moder.modify(|_, w| unsafe {
            w.moder3().bits(1)
             .moder4().bits(1)
             .moder5().bits(1)
        });

        p.GPIOA.pupdr.modify(|_, w| unsafe {
            w.pupdr0().bits(0b10)
        });

        p.GPIOB.pupdr.modify(|_, w| unsafe {
            w.pupdr3().bits(1)
             .pupdr4().bits(1)
             .pupdr5().bits(1)
             .pupdr6().bits(0b10)
             .pupdr7().bits(0b10)
             .pupdr8().bits(0b10)
             .pupdr9().bits(0b10)
        });
    }

    pub fn sample(&mut self, gpioa: &mut GPIOA, gpiob: &mut GPIOB, syst: &SYST) {
        for column in 0..COLUMNS {
            self.enable_column(gpioa, gpiob, column, true);

            // Busy wait a short while before sampling the keys
            // to let the pins settle
            let current_tick = syst.cvr.read();
            let wait_until_tick = current_tick - 100;
            while syst.cvr.read() > wait_until_tick {}

            self.state[5 * column] = gpioa.idr.read().idr0().bit_is_set() as bool;
            self.state[5 * column + 1] = gpiob.idr.read().idr6().bit_is_set() as bool;
            self.state[5 * column + 2] = gpiob.idr.read().idr7().bit_is_set() as bool;
            self.state[5 * column + 3] = gpiob.idr.read().idr8().bit_is_set() as bool;
            self.state[5 * column + 4] = gpiob.idr.read().idr9().bit_is_set() as bool;

            self.enable_column(gpioa, gpiob, column, false);
        }
    }

    fn enable_column(&self, gpioa: &mut GPIOA, gpiob: &mut GPIOB, column: usize, on: bool) {
        match column {
            0 => gpiob.odr.modify(|_, w| w.odr5().bit(on)),
            1 => gpiob.odr.modify(|_, w| w.odr4().bit(on)),
            2 => gpiob.odr.modify(|_, w| w.odr3().bit(on)),
            3 => gpioa.odr.modify(|_, w| w.odr15().bit(on)),
            4 => gpioa.odr.modify(|_, w| w.odr9().bit(on)),
            5 => gpioa.odr.modify(|_, w| w.odr8().bit(on)),
            6 => gpiob.odr.modify(|_, w| w.odr15().bit(on)),
            7 => gpiob.odr.modify(|_, w| w.odr13().bit(on)),
            8 => gpiob.odr.modify(|_, w| w.odr12().bit(on)),
            9 => gpiob.odr.modify(|_, w| w.odr1().bit(on)),
            10 => gpiob.odr.modify(|_, w| w.odr0().bit(on)),
            11 => gpioa.odr.modify(|_, w| w.odr7().bit(on)),
            12 => gpioa.odr.modify(|_, w| w.odr6().bit(on)),
            13 => gpioa.odr.modify(|_, w| w.odr5().bit(on)),
            _ => panic!(),
        }
    }
}
