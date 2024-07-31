#![no_main]
#![no_std]
#![allow(clippy::transmute_ptr_to_ptr)]

mod canbus;
mod init;
mod lighting;
mod horn;
mod comms;
mod device;


use canbus::*;
use init::*;

// global logger
use defmt_rtt as _;
use panic_probe as _;
use stm32g4xx_hal as hal;

use fdcan::{
    frame::RxFrameInfo, 
    FdCanControl, 
    Fifo0, 
    Fifo1, 
    NormalOperationMode, 
    Rx, 
    Tx, 
    id::StandardId, 
    frame::FrameFormat};

use hal::{
    can::Can,
    gpio::{
        gpioa::{PA1, PA2, PA3, PA4, PA5, PA15},
        gpiob::{PB0, PB1, PB2, PB3, PB14, PB7},
        Output,
        GpioExt,
		PushPull
    },
    independent_watchdog::IndependentWatchdog,
    stm32,
    nb::block,
};
use hal::prelude::*;

use rtic_monotonics::{systick::*, Monotonic};

#[rtic::app(device = stm32g4xx_hal::stm32g4::stm32g431, dispatchers = [USART1, USART2])]
mod app {
    use fdcan::{
        frame::TxFrameHeader, 
        interrupt, 
        ExternalLoopbackMode, 
        InternalLoopbackMode, 
        NormalOperationMode, 
        ReceiveOverrun};

    use super::*;
    type FdCanMode = NormalOperationMode; //InternalLoopbackMode

    pub struct Lights {
        pub left_indicator: u8,
        pub right_indicator: u8,
        pub day_light: u8
    }

    #[shared]
    pub struct Shared {
        pub fdcan1_ctrl: FdCanControl<Can<stm32::FDCAN1>, FdCanMode>,
        pub fdcan1_tx: Tx<Can<stm32::FDCAN1>, FdCanMode>,
        pub fdcan1_rx0: Rx<Can<stm32::FDCAN1>, FdCanMode, Fifo0>,
        pub fdcan1_rx1: Rx<Can<stm32::FDCAN1>, FdCanMode, Fifo1>,
        pub light_states: Lights,
        pub horn: PA15<Output<PushPull>>
    }

    #[local]
    pub struct Local {
        pub watchdog: IndependentWatchdog,
        pub led_ok: PB0<Output<PushPull>>,
        pub led_warn: PB7<Output<PushPull>>,
        pub led_error: PB14<Output<PushPull>>,
        pub left_indicator_output: PA4<Output<PushPull>>,
        pub right_indicator_output: PA5<Output<PushPull>>,
        pub day_light_output: PB3<Output<PushPull>>
    }

    #[task(local = [watchdog])]
    async fn watchdog(cx: watchdog::Context) {
        loop {
            cx.local.watchdog.feed();
            Systick::delay(80_u64.millis()).await;
        }
    }

    #[task(local = [led_ok])]
    async fn heartbeat(mut cx: heartbeat::Context){
        loop {
        cx.local.led_ok.set_high().unwrap();
        Systick::delay(500.millis()).await;
        cx.local.led_ok.set_low().unwrap();
        Systick::delay(500.millis()).await;
        }
    }

    #[task(local = [led_error])]
    async fn trigger_led_error(mut cx: trigger_led_error::Context){
        cx.local.led_error.set_high().unwrap();
    }

    #[task(local = [led_warn])]
    async fn trigger_led_warn(mut cx: trigger_led_warn::Context){
        cx.local.led_warn.set_high().unwrap();
    }

    #[task(shared = [fdcan1_tx])]
    async fn send_can_frame(mut cx: send_can_frame::Context, frame: TxFrameHeader, buffer: &[u8]){
        cx.shared.fdcan1_tx.lock(|can_tx|{
            block!(can_tx.transmit(frame, buffer)).unwrap();
        });
        defmt::info!("can frame sent!");
    }

    #[task(shared = [fdcan1_tx, fdcan1_rx1])]
    async fn can_echo_test(mut cx: can_echo_test::Context){
        cx.shared.fdcan1_tx.lock(|tx| {
            cx.shared.fdcan1_rx1.lock(|rx1|{
                let mut buffer: [u8; 8] = [0xAA, 0xAA, 0xAA, 0xAA, 0xFF, 0xFF, 0xFF, 0xFF];

                let header = TxFrameHeader {
                    len: 2 * 4,
                    id: StandardId::new(0x1).unwrap().into(),
                    frame_format: FrameFormat::Standard,
                    bit_rate_switching: false,
                    marker: None,
                };
            
                defmt::info!("Transmit initial message");
                block!(tx.transmit(header, &buffer)).unwrap();
            
                for _i in 0..5 {
                    defmt::info!("Loop");
                    if let Ok(rxheader) = rx1.receive(&mut buffer){
                        defmt::info!("You've made it!");
                        tx.transmit(rxheader.unwrap().to_tx_header(None), &buffer).unwrap();
                    }

                };
            })
        });
        
    }

    extern "Rust" {
        #[init]
        fn init(mut cx: init::Context) -> (Shared, Local);

        #[task(binds = FDCAN1_INTR0_IT, priority = 2, shared = [fdcan1_rx0])]
        fn can_rx0_pending(mut cx: can_rx0_pending::Context);

        #[task(binds = FDCAN1_INTR1_IT, priority = 2, shared = [fdcan1_rx1])]
        fn can_rx1_pending(mut cx: can_rx1_pending::Context);

        #[task(priority = 1)]
        async fn can_receive(mut cx: can_receive::Context, frame: RxFrameInfo, buffer: [u8; 8]);
    }

    #[task(priority = 1, shared = [light_states], local = [left_indicator_output])]
    async fn toggle_left_indicator(mut cx: toggle_left_indicator::Context) {
        let left_ind = cx.local.left_indicator_output;
        let time = Systick::now();
        let on: bool = (time.duration_since_epoch().to_millis() % 1000) > 500;

        // States are toggled from CAN
        cx.shared.light_states.lock(|ls| {
            let _ = left_ind.set_state(PinState::from(on && ls.left_indicator > 0));
        });
    }

    #[task(priority = 1, shared = [light_states], local = [right_indicator_output])]
    async fn toggle_right_indicator(mut cx: toggle_right_indicator::Context) {
        let right_ind: &mut PA5<Output<PushPull>> = cx.local.right_indicator_output;
        let time = Systick::now();
        let on: bool = (time.duration_since_epoch().to_millis() % 1000) > 500;

        // States are toggled from CAN
        cx.shared.light_states.lock(|ls| {
            let _ = right_ind.set_state(PinState::from(on && ls.right_indicator > 0));
        });
    }

    #[task(priority = 1, shared = [light_states], local = [day_light_output])]
    async fn toggle_day_lights(mut cx: toggle_day_lights::Context) {
        let day_light: &mut PB3<Output<PushPull>> = cx.local.day_light_output;

        cx.shared.light_states.lock(|ls| {
            let _ = day_light.set_state(PinState::from(ls.day_light > 0));
        });
    }
    
}

// same panicking *behavior* as `panic-probe` but doesn't print a panic message
// this prevents the panic message being printed *twice* when `defmt::panic` is invoked
#[defmt::panic_handler]
fn panic() -> ! {
    cortex_m::asm::udf()
}

defmt::timestamp!("{=u64:us}", {
    Systick::now().duration_since_epoch().to_micros()
});
