use super::hidreport::HidReport;
use super::keyboard::Keyboard;
use super::led::Led;
use super::protocol::{BleOp, KeyboardOp, LedOp, MacroOp, Message, MsgType, SystemOp};
use super::serial::bluetooth_usart::BluetoothUsart;
use super::serial::{DmaUsart, Serial, Transfer};
use core::marker::Unsize;
use debug::UnwrapLog;
use nb;
use rtfm::Threshold;

#[derive(Copy, Clone, PartialEq)]
pub enum BluetoothMode {
    Unknown,
    Legacy,
    Ble,
}

pub struct Bluetooth<BUFFER: 'static + Unsize<[u8]>> {
    pub serial: Serial<BluetoothUsart, BUFFER>,
    pub rx_transfer: Option<Transfer<BUFFER>>,
    mode: BluetoothMode,
    /// 4-bit bitfield, indicating whether the BT chip has a host
    /// saved in that slot. TODO: investigate high bits (issue #37)
    saved_hosts: u8,
    /// The currently connected slot (1-4), or disconnected (0), or
    /// the current host is not saved (12)
    connected_host: u8,
}

impl<BUFFER> Bluetooth<BUFFER>
where
    BUFFER: Unsize<[u8]>,
{
    pub fn new(
        mut serial: Serial<BluetoothUsart, BUFFER>,
        rx_buffer: &'static mut BUFFER,
    ) -> Bluetooth<BUFFER> {
        let rx_transfer = serial.receive(rx_buffer);
        Bluetooth {
            serial,
            rx_transfer: Some(rx_transfer),
            mode: BluetoothMode::Unknown,
            saved_hosts: 0,
            connected_host: 0,
        }
    }

    pub fn on(&mut self) -> nb::Result<(), !> {
        self.serial.send(MsgType::Ble, BleOp::On as u8, &[])
    }

    pub fn off(&mut self) -> nb::Result<(), !> {
        self.serial.send(MsgType::Ble, BleOp::Off as u8, &[])
    }

    pub fn save_host(&mut self, host: u8) -> nb::Result<(), !> {
        // TODO: host < 4?
        self.serial
            .send(MsgType::Ble, BleOp::SaveHost as u8, &[host])
    }

    pub fn connect_host(&mut self, host: u8) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Ble, BleOp::ConnectHost as u8, &[host])
    }

    pub fn delete_host(&mut self, host: u8) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Ble, BleOp::DeleteHost as u8, &[host])
    }

    pub fn broadcast(&mut self) -> nb::Result<(), !> {
        self.serial.send(MsgType::Ble, BleOp::Broadcast as u8, &[])
    }

    pub fn enable_legacy_mode(&mut self, enabled: bool) -> nb::Result<(), !> {
        let on = if enabled { 1 } else { 0 };
        self.serial
            .send(MsgType::Ble, BleOp::LegacyMode as u8, &[on])
    }

    pub fn toggle_legacy_mode(&mut self) -> nb::Result<(), !> {
        let enabled: bool = self.mode == BluetoothMode::Ble;
        self.enable_legacy_mode(enabled)
    }

    pub fn host_list_query(&mut self) -> nb::Result<(), !> {
        self.serial
            .send(MsgType::Ble, BleOp::HostListQuery as u8, &[])
    }

    pub fn send_report(&mut self, report: &HidReport) -> nb::Result<(), !> {
        self.serial.send(
            MsgType::Keyboard,
            KeyboardOp::KeyReport as u8,
            report.as_bytes(),
        )
    }

    pub fn update_led(&self, led: &mut Led<BUFFER>) -> nb::Result<(), !> {
        led.bluetooth_mode(self.saved_hosts, self.connected_host, self.mode)
    }

    pub fn handle_message(
        &mut self,
        message: &Message,
        led: &mut Led<BUFFER>,
        keyboard: &mut Keyboard,
    ) {
        match message.msg_type {
            MsgType::System => {
                match SystemOp::from(message.operation) {
                    SystemOp::GetId => {
                        const DEVICE_TYPE_KEYBOARD: u8 = 1;
                        const DEVICE_MODEL_ANNE_PRO: u8 = 2;
                        //const DEVICE_ID = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12];

                        // send two packets
                        // nblock = 2
                        // [datalen, nblock, iblock = 0, data...]
                        // [datalen, nblock, iblock = 1, data...]

                        let data1 = [
                            10,
                            2,
                            0,
                            DEVICE_TYPE_KEYBOARD,
                            DEVICE_MODEL_ANNE_PRO,
                            1,
                            2,
                            3,
                            4,
                            5,
                            6,
                        ];
                        let data2 = [8, 2, 1, 7, 8, 9, 10, 11, 12];
                        self.serial
                            .send(MsgType::System, SystemOp::AckGetId as u8, &data1)
                            .log_error();
                        self.serial
                            .send(MsgType::System, SystemOp::AckGetId as u8, &data2)
                            .log_error();
                    }
                    SystemOp::IsSyncCode => {
                        self.serial
                            .send(MsgType::System, SystemOp::AckIsSyncCode as u8, &[1])
                            .log_error();
                    }
                    SystemOp::SetSyncCode => {
                        self.serial
                            .send(MsgType::System, SystemOp::AckIsSyncCode as u8, &[])
                            .log_error();
                    }
                    _ => {
                        debug!("msg: System {} {:?}", message.operation, message.data).ok();
                    }
                }
            }
            MsgType::Ble => {
                match BleOp::from(message.operation) {
                    BleOp::AckWakeup => {
                        // nothing to do here, this message only only lets us know
                        // that we can now safely send
                    }
                    BleOp::AckOn => {
                        // data = [0]
                        // TODO: always getting a [0] too much?
                        //debug!("bt ack on: {:?}", message.data).ok();
                    }
                    BleOp::AckOff => {
                        // data = [0]
                        //debug!("bt ack off: {:?}", message.data).ok();
                    }
                    BleOp::AckLegacyMode => {
                        // data = [0]
                        //debug!("bt ack legacy mode: {:?}", message.data).ok();
                    }
                    BleOp::AckDeleteHost => {
                        // data = [0]
                        //debug!("bt ack delete host: {:?}", message.data).ok();
                    }
                    BleOp::Pair => {
                        debug!("bt pair").ok();
                        keyboard.disable_bluetooth_mode();
                        led.bluetooth_pin_mode().log_error();
                    }
                    BleOp::Disconnect => {
                        // check this? sent after off, 14
                        debug!("bt disconnect").ok();
                    }
                    BleOp::AckHostListQuery => {
                        if message.data.len() == 3 {
                            self.saved_hosts = message.data[0];
                            self.connected_host = message.data[1];
                            self.mode = match message.data[2] {
                                0 => BluetoothMode::Ble,
                                1 => BluetoothMode::Legacy,
                                _ => BluetoothMode::Unknown,
                            };
                        }

                        if keyboard.bluetooth_mode_enabled() {
                            self.update_led(led).log_error();
                        }
                    }
                    _ => {
                        debug!("msg: Ble {} {:?}", message.operation, message.data).ok();
                    }
                }
            }
            MsgType::Led => match LedOp::from(message.operation) {
                LedOp::ThemeMode => {
                    led.set_theme(message.data[0]).log_error();
                }
                LedOp::GetUserStaticTheme => {
                    debug!("TODO: Theme Sync").ok();
                    // [data_length, num_blocks, block_i]
                    //let data = [2 + 4, 1, 0, 1, 2, 3, 4];
                    //self.serial
                    //.send(MsgType::Led, LedOp::AckGetUserStaticTheme as u8, &data)
                    //.log_error();
                }
                _ => {
                    debug!("msg: Led {} {:?}", message.operation, message.data).ok();
                }
            },
            MsgType::Keyboard => match KeyboardOp::from(message.operation) {
                KeyboardOp::UpUserLayout => {
                    debug!("TODO: Keyboard Sync").ok();
                }
                _ => {
                    debug!("msg: Keyboard {} {:?}", message.operation, message.data).ok();
                }
            },
            MsgType::Macro => match MacroOp::from(message.operation) {
                MacroOp::SyncMacro => {
                    debug!("TODO: Macro Sync").ok();
                }
                _ => {
                    debug!("msg: macro {} {:?}", message.operation, message.data).ok();
                }
            },
            _ => {
                debug!(
                    "msg: {:?} {} {:?}",
                    message.msg_type, message.operation, message.data
                )
                .ok();
            }
        }
    }

    pub fn poll(&mut self, led: &mut Led<BUFFER>, keyboard: &mut Keyboard) {
        let result = self
            .rx_transfer
            .as_mut()
            .unwrap()
            .poll(&mut self.serial.usart);
        match result {
            Err(nb::Error::WouldBlock) => {}
            Err(_) => panic!(),
            Ok(()) => {
                let buffer = self.rx_transfer.take().unwrap().finish();
                {
                    let buffer: &mut [u8] = buffer;
                    let message = Message {
                        msg_type: MsgType::from(buffer[0]),
                        operation: buffer[2],
                        data: &buffer[3..3 + buffer[1] as usize - 1],
                    };
                    self.handle_message(&message, led, keyboard);

                    if let (MsgType::Ble, BleOp::AckWakeup) =
                        (message.msg_type, message.operation.into())
                    {
                        // Wakeup acknowledged, send data
                        self.serial.usart.ack_wakeup();
                        self.serial.send_buffer_pos = 0;
                    }
                }

                self.rx_transfer = Some(self.serial.receive(buffer));
            }
        }
    }
}

pub fn rx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL6::Resources) {
    r.BLUETOOTH.poll(&mut r.LED, &mut r.KEYBOARD)
}

pub fn tx(_t: &mut Threshold, mut r: super::DMA1_CHANNEL7::Resources) {
    r.BLUETOOTH.serial.tx_interrupt();
}
