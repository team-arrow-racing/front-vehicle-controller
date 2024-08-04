use crate::app::*;
use crate::horn::PGN_HORN_MESSAGE;
use crate::lighting::{LampsState, PGN_LIGHTING_STATE};

use fdcan::{frame::RxFrameInfo, id::Id, interrupt::Interrupt};
use rtic::Mutex;
use stm32g4xx_hal::nb::block;
use j1939::pgn::Number;

fn pgn_from_rawid(rawid: u32) -> Number {
    //Isolates bit 9-18 for the pgn
    let raw_pgn = (rawid & 0x00FF0000) >> 8;

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
    cx.shared.can.lock(|can| {
        let mut buffer = [0_u8; 8];
        if can.has_interrupt(Interrupt::RxFifo0NewMsg) {
            match can.receive0(&mut buffer) {
                Ok(rxframe) => {
                    defmt::trace!("frame received");
                    can_receive::spawn(rxframe.unwrap(), buffer).ok()
                },
                Err(_) => {
                    defmt::trace!("Error");
                    Some(())
                }
            };

            can.clear_interrupt(Interrupt::RxFifo0NewMsg);
        }
    });
}

pub fn can_rx1_pending(mut cx: can_rx1_pending::Context) {
    defmt::trace!("RX1 received");
    cx.shared.can.lock(|can| {
        let mut buffer = [0_u8; 8];

        if can.has_interrupt(Interrupt::RxFifo1NewMsg) {
            defmt::trace!("int triggered");
            match can.receive1(&mut buffer) {
                Ok(rxframe) => {
                    defmt::trace!("frame received");
                    can_receive::spawn(rxframe.unwrap(), buffer).ok()
                },
                Err(_) => {
                    defmt::trace!("Error");
                    Some(())
                }
            };

            can.clear_interrupt(Interrupt::RxFifo1NewMsg);
        }
    });
}

pub async fn can_receive(mut cx: can_receive::Context<'_>, frame: RxFrameInfo, buffer: [u8; 8]) {
    let id = frame.id;
    match id {
        Id::Standard(id) => {
            defmt::info!("Received Standard Header: {:#02x}", id.as_raw());
        },
        Id::Extended(id) => {
            defmt::info!("Received Extended Header: {:#03x}", id.as_raw());

            let pgn = pgn_from_rawid(id.as_raw());

            match pgn {
                PGN_HORN_MESSAGE => {
                    defmt::info!("Received Horn Message");
                },
                PGN_LIGHTING_STATE => {
                    defmt::info!("Received Lighting Message");
                    let lamp_state = LampsState::from_bits(buffer[0]).unwrap_or_else(LampsState::empty);

                    if lamp_state.contains(LampsState::DAYTIME){
                        defmt::info!("Daytime: ON");
                    } else{
                        defmt::info!("Daytime: OFF");
                    }

                    if lamp_state.contains(LampsState::STOP){
                        defmt::info!("Stop: ON");
                    } else{
                        defmt::info!("Stop: OFF");
                    }

                    if lamp_state.contains(LampsState::INDICATOR_LEFT){
                        defmt::info!("Left Indicator: ON");
                    } else{
                        defmt::info!("Left Indicator: OFF");
                    }

                    if lamp_state.contains(LampsState::INDICATOR_RIGHT){
                        defmt::info!("Right Indicator: ON");
                    } else{
                        defmt::info!("Right Indicator: OFF");
                    }

                    if lamp_state.contains(LampsState::HAZARD){
                        defmt::info!("Hazards: ON");
                    } else{
                        defmt::info!("Hazards: OFF")
                    }

                },
                _ => {
                    defmt::info!("Received Unknown Message");
                    trigger_led_error::spawn().ok();
                }
                
            }
        }
    }
    defmt::info!("received data: {:#02x}", buffer);
}