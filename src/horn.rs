use crate::comms::Priority;
use crate::device::{source_address, Device};
use bxcan::{ExtendedId, Frame};
use j1939::pgn::{Number, Pgn};

#[repr(u8)]
pub enum HornMessageFormat {
    Enable = 0xEE,
}

pub const PGN_HORN_MESSAGE: Number = Number {
    specific: Device::VehicleController as u8,
    format: HornMessageFormat::Enable as u8,
    data_page: false,
    extended_data_page: false,
};

pub fn horn_message(device: Device, state: u8) -> Frame {
    let id = j1939::ExtendedId {
        priority: Priority::Default as u8,
        pgn: Pgn::new(PGN_HORN_MESSAGE),
        source_address: source_address(device).unwrap(),
    };

    Frame::new_data(ExtendedId::new(id.to_bits()).unwrap(), [state])
}