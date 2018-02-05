#![feature(const_fn)]

use stm32l151::{GPIOA, GPIOB, SYST};

const ROWS: usize = 5;
const COLUMNS: usize = 14;

pub type KeyState = [bool; ROWS * COLUMNS];

pub struct Keyboard {
    /// Stores the currently pressed down keys from last sample.
    pub state: KeyState,
}

impl Keyboard {
    pub fn new(gpioa: &mut GPIOA, gpiob: &mut GPIOB) -> Keyboard {
        let kb = Keyboard {
            state: [false; ROWS * COLUMNS],
        };
        kb.init(gpioa, gpiob);
        kb
    }

    fn init(&self, gpioa: &mut GPIOA, gpiob: &mut GPIOB) {
        gpioa.moder.modify(|_, w| unsafe {
            w.moder5().bits(1)
             .moder6().bits(1)
             .moder7().bits(1)
             .moder8().bits(1)
             .moder9().bits(1)
             .moder15().bits(1)
        });

        gpiob.moder.modify(|_, w| unsafe {
            w.moder0().bits(1)
             .moder1().bits(1)
             .moder3().bits(1)
             .moder4().bits(1)
             .moder5().bits(1)
             .moder12().bits(1)
             .moder13().bits(1)
             .moder14().bits(1)
        });

        gpioa.pupdr.modify(|_, w| unsafe {
            w.pupdr5().bits(1)
             .pupdr6().bits(1)
             .pupdr7().bits(1)
             .pupdr8().bits(1)
             .pupdr9().bits(1)
             .pupdr15().bits(1)
             .pupdr0().bits(0b10)
        });

        gpiob.pupdr.modify(|_, w| unsafe {
            w.pupdr0().bits(1)
             .pupdr1().bits(1)
             .pupdr3().bits(1)
             .pupdr4().bits(1)
             .pupdr5().bits(1)
             .pupdr12().bits(1)
             .pupdr13().bits(1)
             .pupdr14().bits(1)
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

            self.state[column              ] = gpiob.idr.read().idr9().bit_is_set() as bool;
            self.state[column +     COLUMNS] = gpiob.idr.read().idr8().bit_is_set() as bool;
            self.state[column + 2 * COLUMNS] = gpiob.idr.read().idr7().bit_is_set() as bool;
            self.state[column + 3 * COLUMNS] = gpiob.idr.read().idr6().bit_is_set() as bool;
            self.state[column + 4 * COLUMNS] = gpioa.idr.read().idr0().bit_is_set() as bool;

            self.enable_column(gpioa, gpiob, column, false);
        }
    }

    fn enable_column(&self, gpioa: &mut GPIOA, gpiob: &mut GPIOB, column: usize, on: bool) {
        match column {
            0 => gpioa.odr.modify(|_, w| w.odr5().bit(on)),
            1 => gpioa.odr.modify(|_, w| w.odr6().bit(on)),
            2 => gpioa.odr.modify(|_, w| w.odr7().bit(on)),
            3 => gpiob.odr.modify(|_, w| w.odr0().bit(on)),
            4 => gpiob.odr.modify(|_, w| w.odr1().bit(on)),
            5 => gpiob.odr.modify(|_, w| w.odr12().bit(on)),
            6 => gpiob.odr.modify(|_, w| w.odr13().bit(on)),
            7 => gpiob.odr.modify(|_, w| w.odr14().bit(on)),
            8 => gpioa.odr.modify(|_, w| w.odr8().bit(on)),
            9 => gpioa.odr.modify(|_, w| w.odr9().bit(on)),
            10 => gpioa.odr.modify(|_, w| w.odr15().bit(on)),
            11 => gpiob.odr.modify(|_, w| w.odr3().bit(on)),
            12 => gpiob.odr.modify(|_, w| w.odr4().bit(on)),
            13 => gpiob.odr.modify(|_, w| w.odr5().bit(on)),
            _ => panic!(),
        }
    }
}
