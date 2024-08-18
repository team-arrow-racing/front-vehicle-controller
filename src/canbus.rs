use crate::app::*;

//TODO: migrate to solar-car-common crate
use crate::horn::PGN_HORN_MESSAGE;
use crate::lighting::{LampsState, PGN_LIGHTING_STATE};

use fdcan::{frame::RxFrameInfo, id::Id, interrupt::Interrupt};
use rtic::Mutex;
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

pub async fn update_light_states(mut cx: update_light_states::Context<'_>, state: LampsState){
    //update states from CAN lighting frame
    cx.shared.light_states.lock(|ls|{
        ls.day_light = state.contains(LampsState::DAYTIME) as u8;
        ls.left_indicator = state.contains(LampsState::INDICATOR_LEFT) as u8;
        ls.right_indicator = state.contains(LampsState::INDICATOR_RIGHT) as u8;
    });
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

            let pgn = pgn_from_rawid(id.as_raw());

            match pgn {
                PGN_HORN_MESSAGE => {
                    set_horn::spawn(buffer[0]).ok();
                },
                PGN_LIGHTING_STATE => {
                    let lamp_state = LampsState::from_bits(buffer[0]).unwrap_or_else(LampsState::empty);
                
                    //update stored lamp states directly from CAN for every frame
                    update_light_states::spawn(lamp_state).ok();

                    //update light states whenever lighting message is received
                    toggle_day_lights::spawn().ok();
                    toggle_left_indicator::spawn().ok();
                    toggle_right_indicator::spawn().ok();

                },
                _ => {
                    defmt::info!("Received Unknown Message");
                    trigger_led_error::spawn().ok();
                }
                
            }
        }
    }
}