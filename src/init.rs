use crate::app::{init, watchdog, Local, Shared};

use stm32g4xx_hal as hal;

use hal::{
    can::CanExt,
	gpio::{GpioExt as _, Output, Speed},
	independent_watchdog::IndependentWatchdog,
	pwr::PwrExt,
	rcc::{self, Config, RccExt, SysClockSrc},
	time::{ExtU32, RateExtU32}
};

use fdcan::{
    config::NominalBitTiming,
    filter::{StandardFilter, StandardFilterSlot},
    frame::{FrameFormat, TxFrameHeader},
    id::StandardId,
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

    let gpiob = cx.device.GPIOB.split(&mut rcc);

    let btr = NominalBitTiming {
        prescaler: NonZeroU16::new(12).unwrap(),
        seg1: NonZeroU8::new(13).unwrap(),
        seg2: NonZeroU8::new(2).unwrap(),
        sync_jump_width: NonZeroU8::new(1).unwrap(),
    };

    let (fdcan1_ctrl, fdcan1_tx, fdcan1_rx0, fdcan1_rx1) = {
        let rx = gpiob.pb8.into_alternate().set_speed(Speed::VeryHigh);
        let tx = gpiob.pb9.into_alternate().set_speed(Speed::VeryHigh);

        let mut can = cx.device.FDCAN1.fdcan(tx, rx, &rcc);
        can.set_protocol_exception_handling(false);

        can.set_nominal_bit_timing(btr);

        can.set_standard_filter(
            StandardFilterSlot::_0,
            StandardFilter::accept_all_into_fifo0(),
        );

        can.into_normal().split()
    };

    // Monotonics
    Systick::start(
        cx.core.SYST,
        24_000_000,
        rtic_monotonics::create_systick_token!(),
    );

    watchdog::spawn().ok();

    defmt::info!("Initialisation finished.");

    (
        Shared {
            fdcan1_rx0,
            fdcan1_rx1
        },
        Local {watchdog},
    )		
}	