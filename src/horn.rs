use crate::comms::Priority;
use crate::device::{source_address, Device};

use fdcan::{frame::{TxFrameHeader, FrameFormat}, id::{Id, ExtendedId}};
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

pub fn horn_header(device: Device) -> TxFrameHeader {
    let j1939id = j1939::ExtendedId {
        priority: Priority::Default as u8,
        pgn: Pgn::new(PGN_HORN_MESSAGE),
        source_address: source_address(device).unwrap(),
    };

    let header = TxFrameHeader {
        len: 1,
        frame_format: FrameFormat::Fdcan,
        id: Id::Extended(ExtendedId::new(j1939id.to_bits()).unwrap()),
        bit_rate_switching: true,
        marker: None
    };
    
    header
}