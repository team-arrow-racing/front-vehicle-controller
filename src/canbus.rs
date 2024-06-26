use crate::app::*;
use crate::horn::{HornMessageFormat, PGN_HORN_MESSAGE};
use crate::lighting::{LampsState, PGN_LIGHTING_STATE};

use fdcan::{frame::RxFrameInfo, id::Id};
use rtic::Mutex;
use stm32g4xx_hal::nb::block;
use j1939::pgn::{Number, Pgn};

pub fn pgn_from_rawid(rawid: u32) -> Number {
    //Isolates bit 9-18 for the pgn
    let raw_pgn = (rawid >> 9) & 0x3FFFF; 

    //Split the PGN bitwise into sections
    let specific = (raw_pgn & 0xFF) as u8;
    let format = ((raw_pgn >> 8) & 0xFF) as u8;
    let data_page = ((raw_pgn >> 16) & 0x01) != 0;
    let extended_data_page = ((raw_pgn >> 17) & 0x01) != 0;
    
    Number {
        specific,
        format,
        data_page,
        extended_data_page,
    }
}

pub fn can_rx0_pending(mut cx: can_rx0_pending::Context) {
    defmt::trace!("RX0 received");
    cx.shared.fdcan1_rx0.lock(|rx| {
        let mut buffer = [0_u8; 8];
        let rxframe = block!(rx.receive(&mut buffer));

        if let Ok(rxframe) = rxframe {
            can_receive::spawn(rxframe.unwrap(), buffer).ok();
        }
    });
}

pub fn can_rx1_pending(mut cx: can_rx1_pending::Context) {
    defmt::trace!("RX1 received");
    cx.shared.fdcan1_rx1.lock(|rx| {
        let mut buffer = [0_u8; 8];
        let rxframe = block!(rx.receive(&mut buffer));

        if let Ok(rxframe) = rxframe {
            can_receive::spawn(rxframe.unwrap(), buffer).ok();
        }
    });
}

pub async fn can_receive(mut cx: can_receive::Context<'_>, frame: RxFrameInfo, buffer: [u8; 8]) {
    let id = frame.id;
    match id {
        Id::Standard(id) => {
            defmt::info!("Received Header: {:#02x}", id.as_raw());
        },
        Id::Extended(id) => {
            defmt::info!("Received Header: {:#03x}", id.as_raw());

            let pgn = pgn_from_rawid(id.as_raw());

            match pgn {
                PGN_HORN_MESSAGE => {
                    defmt::info!("Received Horn Message");
                },
                PGN_LIGHTING_STATE => {
                    defmt::info!("Received Lighting Message");
                },
                _ => {
                    defmt::info!("Received Unknown Message");
                }
                
            }
        }
    }
    defmt::info!("received data: {:#02x}", buffer);
}