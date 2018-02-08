#![feature(const_fn)]

use embedded_hal::digital::{InputPin, OutputPin};
use stm32l151::SYST;

// TODO: make generic?
const ROWS: usize = 5;
const COLUMNS: usize = 14;

// TODO: move into generic keyboard?
pub type KeyState = [bool; ROWS * COLUMNS];


// TODO: could this be implemented generically by passing in all the pin types as generics?
// so it doesn't need to store the pin numbers at runtime, because it knows during compilation?
// maybe define type aliases or so for rows/columns pin groups
// maybe via macro? or some type?
// maybe even as a tuple type?
// (PA0, PB6, PB7, PB8, PB9)
pub struct Keyboard<R, C> {
    /// Stores the currently pressed down keys from last sample.
    pub state: KeyState,
    row_pins: [R; ROWS],
    column_pins: [C; COLUMNS],
}

impl<R, C> Keyboard<R, C> where R: InputPin, C: OutputPin {
    pub fn new(row_pins: [R; ROWS], column_pins: [C; COLUMNS]) -> Keyboard<R, C> {
        Keyboard {
            state: [false; ROWS * COLUMNS],
            row_pins: row_pins,
            column_pins: column_pins,
        }
    }

    pub fn sample(&mut self, syst: &SYST) {
        for column in 0..COLUMNS {
            self.column_pins[column].set_high();

            // Busy wait a short while before sampling the keys
            // to let the pins settle
            let current_tick = syst.cvr.read();
            let wait_until_tick = current_tick - 100;
            while syst.cvr.read() > wait_until_tick {}

            for row in 0..ROWS {
                self.state[column + row * COLUMNS] = self.row_pins[row].is_high();
            }

            self.column_pins[column].set_low();
        }
    }
}
