use crate::app::{
            heartbeat, 
            init, 
            watchdog, 
            Lights, 
            Local, 
            Shared
};

use embedded_hal::digital::v2::OutputPin;
use stm32g4xx_hal as hal;

use hal::{
    can::CanExt,
	gpio::{GpioExt as _, Output, Speed},
	independent_watchdog::IndependentWatchdog,
	pwr::PwrExt,
	rcc::{self, Config, RccExt, SysClockSrc},
	time::{ExtU32, RateExtU32},
};

use fdcan::{
    config::NominalBitTiming,
    interrupt::*,
};

use core::num::{NonZeroU16, NonZeroU8};

use rtic_monotonics::{
    systick::*,
    Monotonic,
};


pub fn init(cx: init::Context) -> (Shared, Local) {
    defmt::info!("init");

    // Setup and start independent watchdog.
    // Initialisation must complete before the watchdog triggers
    let watchdog = {
        let mut wd = IndependentWatchdog::new(cx.device.IWDG);
        wd.start(100_u32.millis());
        wd
    };

    // configure power domain
    let pwr = cx
        .device
        .PWR
        .constrain()
        .freeze();

    // RCC
    let rcc = cx.device.RCC.constrain();
    let mut rcc = rcc.freeze(Config::new(SysClockSrc::HSE(24.MHz())), pwr);

    // GPIO
    let gpioa = cx.device.GPIOA.split(&mut rcc);
    let gpiob = cx.device.GPIOB.split(&mut rcc);
    let gpioc = cx.device.GPIOC.split(&mut rcc);
    let gpiod = cx.device.GPIOD.split(&mut rcc);

    // Status LEDs
    let led_ok = gpiob.pb0.into_push_pull_output();
    let led_warn = gpiob.pb7.into_push_pull_output();
    let led_error = gpiob.pb14.into_push_pull_output();

    let btr = NominalBitTiming {
        prescaler: NonZeroU16::new(12).unwrap(),
        seg1: NonZeroU8::new(13).unwrap(),
        seg2: NonZeroU8::new(2).unwrap(),
        sync_jump_width: NonZeroU8::new(1).unwrap(),
    };

    let can = {
        let rx = gpiob.pb8.into_alternate().set_speed(Speed::VeryHigh);
        let tx = gpiob.pb9.into_alternate().set_speed(Speed::VeryHigh);

        let mut can = cx.device.FDCAN1.fdcan(tx, rx, &rcc);
        can.set_protocol_exception_handling(false);

        can.set_nominal_bit_timing(btr);

        can.enable_interrupt_line(InterruptLine::_0, true);
        can.enable_interrupt_line(InterruptLine::_1, true);
        can.enable_interrupts(Interrupts::RX_FIFO0_NEW_MSG | Interrupts::RX_FIFO1_NEW_MSG);

        //debug mode only
        can.into_external_loopback()
    };

    // Light outputs
    let left_indicator_output = gpioa.pa4.into_push_pull_output();
    let right_indicator_output = gpioa.pa5.into_push_pull_output();
    let day_light_output = gpiob.pb3.into_push_pull_output();

    // Light states
    let light_states = Lights {
        left_indicator: 0,
        right_indicator: 0,
        day_light: 0
    };

    // Horn
    let horn = gpioa.pa15.into_push_pull_output();
    let horn_trigger = gpioc.pc13.into_pull_down_input(); //debug input only

    // Monotonics
    Systick::start(
        cx.core.SYST,
        24_000_000,
        rtic_monotonics::create_systick_token!(),
    );

    watchdog::spawn().ok();
    heartbeat::spawn().ok();
    
    (
        Shared {
            can,
            light_states,
            horn_trigger
        },
        Local {
            watchdog,
            led_ok,
            led_warn,
            led_error,
            left_indicator_output,
            right_indicator_output,
            day_light_output,
            horn
        },
    )		
}	