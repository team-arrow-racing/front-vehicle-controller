#![no_main]
#![no_std]
#![allow(clippy::transmute_ptr_to_ptr)]

//NOTE: "cargo test" throws an error in the build process

//Using similar imports to main.rs to avoid issues - obviously some not needed
use panic_probe as _;
use stm32g4xx_hal as hal;
use defmt_rtt as _;

use hal::{
    can::Can,
    gpio::{
        gpioa::{PA1, PA2, PA3, PA4, PA5, PA15},
        gpiob::{PB0, PB1, PB2, PB3, PB14, PB7},
        gpioc::{PC13},
        Input,
        Output,
		PushPull
    },
    independent_watchdog::IndependentWatchdog,
    stm32::FDCAN1,
    nb::block,
};
use hal::prelude::*;


mod tests {
    use super::*;
    use defmt::assert_eq;
    use defmt_rtt as _; // global logger
    use cortex_m_rt::entry;
    use solar_car::com::{lighting, horn};
    use solar_car::device::Device;

    #[test]
    fn can_receive_lighting(){
        let header = lighting_header(Device::VehicleController);
        
        app::test_horn::spawn(true);


    }

    #[test]
    fn horn_activation_test(){

    }

    
}
