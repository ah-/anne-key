use bit_field::BitArray;
use embedded_hal::digital::v2::{InputPin, OutputPin};
use hal::gpio::gpioa::*;
use hal::gpio::gpiob::*;
use hal::gpio::{Input, Output};
use stm32l1::stm32l151::SYST;

pub const ROWS: usize = 5;
pub const COLUMNS: usize = 14;

type RowPins = (PB9<Input>, PB8<Input>, PB7<Input>, PB6<Input>, PA0<Input>);
type ColumnPins = (
    PA5<Output>,
    PA6<Output>,
    PA7<Output>,
    PB0<Output>,
    PB1<Output>,
    PB12<Output>,
    PB13<Output>,
    PB14<Output>,
    PA8<Output>,
    PA9<Output>,
    PA15<Output>,
    PB3<Output>,
    PB4<Output>,
    PB5<Output>,
);

/// State of the scan matrix
///
/// A 72-bit array whose most-significant 2 bits are unused.  Each
/// key's state is stored as 1 (pressed) or 0 (released) at the bit
/// indexed by the corresponding [`keycodes::KeyIndex`], namely
/// `Escape` is at bit 0 (least-significant bit), and `RCtrl` is at
/// bit 69.
///
/// This packed format is sent as-is to stock LED firmware for theme
/// activation.
pub type KeyState = [u8; (ROWS * COLUMNS + 2) / 8]; // [u8; 9]

pub struct KeyMatrix {
    /// Stores the currently pressed down keys from last sample.
    pub state: KeyState,
    row_pins: RowPins,
    column_pins: ColumnPins,
}

impl KeyMatrix {
    pub fn new(row_pins: RowPins, column_pins: ColumnPins) -> Self {
        Self {
            state: [0; 9],
            row_pins,
            column_pins,
        }
    }

    pub fn sample(&mut self, syst: &SYST) {
        for column in 0..COLUMNS {
            self.enable_column(column);

            // Busy wait a short while before sampling the keys
            // to let the pins settle
            let current_tick = syst.cvr.read();
            let wait_until_tick = current_tick - 100;
            while syst.cvr.read() > wait_until_tick {}

            self.state
                .set_bit(column, self.row_pins.0.is_high().unwrap());
            self.state
                .set_bit(column + COLUMNS, self.row_pins.1.is_high().unwrap());
            self.state
                .set_bit(column + 2 * COLUMNS, self.row_pins.2.is_high().unwrap());
            self.state
                .set_bit(column + 3 * COLUMNS, self.row_pins.3.is_high().unwrap());
            self.state
                .set_bit(column + 4 * COLUMNS, self.row_pins.4.is_high().unwrap());

            self.disable_column(column);
        }
    }

    fn enable_column(&mut self, column: usize) {
        match column {
            0 => self.column_pins.0.set_high(),
            1 => self.column_pins.1.set_high(),
            2 => self.column_pins.2.set_high(),
            3 => self.column_pins.3.set_high(),
            4 => self.column_pins.4.set_high(),
            5 => self.column_pins.5.set_high(),
            6 => self.column_pins.6.set_high(),
            7 => self.column_pins.7.set_high(),
            8 => self.column_pins.8.set_high(),
            9 => self.column_pins.9.set_high(),
            10 => self.column_pins.10.set_high(),
            11 => self.column_pins.11.set_high(),
            12 => self.column_pins.12.set_high(),
            13 => self.column_pins.13.set_high(),
            _ => Ok(()),
        }
        .unwrap()
    }

    fn disable_column(&mut self, column: usize) {
        match column {
            0 => self.column_pins.0.set_low(),
            1 => self.column_pins.1.set_low(),
            2 => self.column_pins.2.set_low(),
            3 => self.column_pins.3.set_low(),
            4 => self.column_pins.4.set_low(),
            5 => self.column_pins.5.set_low(),
            6 => self.column_pins.6.set_low(),
            7 => self.column_pins.7.set_low(),
            8 => self.column_pins.8.set_low(),
            9 => self.column_pins.9.set_low(),
            10 => self.column_pins.10.set_low(),
            11 => self.column_pins.11.set_low(),
            12 => self.column_pins.12.set_low(),
            13 => self.column_pins.13.set_low(),
            _ => Ok(()),
        }
        .unwrap()
    }
}
