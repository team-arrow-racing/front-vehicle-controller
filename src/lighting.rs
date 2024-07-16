use crate::app::send_can_frame;
use crate::comms::{MessageFormat, Priority};
use crate::device::{source_address, Device};
use bitflags::bitflags;
use fdcan::{frame::{TxFrameHeader, FrameFormat}, id::{Id, ExtendedId}};
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

pub fn lighting_message(device: Device, lamp: LampsState){
    //Construct id
    let j1939id = j1939::ExtendedId{
        priority: Priority::Default as u8,
        pgn: Pgn::new(PGN_LIGHTING_STATE),
        source_address: source_address(device).unwrap(),
    };

    //Construct header
    let header = TxFrameHeader {
        len: 2,
        frame_format: FrameFormat::Fdcan,
        id: Id::Extended(ExtendedId::new(j1939id.to_bits()).unwrap()),
        bit_rate_switching: true,
        marker: None
    };

    //Transmist frame, result unused
    let _ = send_can_frame::spawn(header, &[lamp.bits()]);
}

pub fn lighting_test(){

    //Send empty lighting message (reset all)
    lighting_message(Device::VehicleController, LampsState::empty());

    //Iterate through all lighting flags
    let mut bits: u8 = 0b00000001;
    for _i in 0..4{
       lighting_message(Device::VehicleController, LampsState{bits});
       bits = bits << 1;
    }
}