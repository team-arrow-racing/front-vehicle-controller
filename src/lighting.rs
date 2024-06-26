use crate::comms::{MessageFormat, Priority};
use crate::device::{source_address, Device};
use bitflags::bitflags;
use bxcan::{ExtendedId, Frame};
use j1939::pgn::{Number, Pgn};


bitflags! {
    /// As per
    #[derive(Default)]
    pub struct LampsState: u8 {
        // indicator lamps
        const INDICATOR_LEFT = 1 << 0;
        const INDICATOR_RIGHT = 1 << 1;
        const HAZARD = Self::INDICATOR_LEFT.bits | Self::INDICATOR_RIGHT.bits;

        // daytime lamps
        const DAYTIME = 1 << 2;

        // stop lamps
        const STOP = 1 << 3;
    }
}

pub const PGN_LIGHTING_STATE: Number = Number {
    specific: Device::VehicleController as u8,
    format: MessageFormat::Lighting as u8,
    data_page: false,
    extended_data_page: false,
};

pub fn message(device: Device, lamp: LampsState, value: u8) -> Frame {
    let id = j1939::ExtendedId {
        priority: Priority::Default as u8,
        pgn: Pgn::new(PGN_LIGHTING_STATE),
        source_address: source_address(device).unwrap(),
    };

    Frame::new_data(ExtendedId::new(id.to_bits()).unwrap(), [lamp.bits(), value])
}
