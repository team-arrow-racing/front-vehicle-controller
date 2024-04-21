#![no_main]
#![no_std]
#![allow(clippy::transmute_ptr_to_ptr)]

// global logger
use defmt_rtt as _;
use panic_probe as _;
use stm32g4xx_hal as hal;

use hal::{
	gpio::{Output, Speed},
	independent_watchdog::IndependentWatchdog,
	pwr::PwrExt,
	rcc::{self, Config, RccExt, SysClockSrc},
	stm32::Peripherals,
	time::{ExtU32, RateExtU32}
};

use rtic_monotonics::{
    systick::*,
    Monotonic,
};

use core::num::{NonZeroU16, NonZeroU8};

use cortex_m_rt::entry;

#[rtic::app(device = stm32g4xx_hal::stm32g4::stm32g431, dispatchers = [USART1, USART2])]
mod app {
	use super::*;
	
	#[shared]
	pub struct Shared{}
	
	#[local]
	pub struct Local {
		pub watchdog: IndependentWatchdog,
	}
	
	#[init]
	fn init(cx: init::Context) -> (Shared, Local) {
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
		let ccdr = rcc.freeze(Config::new(SysClockSrc::HSE(24.MHz())), pwr);

		// Monotonics
		Systick::start(
			cx.core.SYST,
			24_000_000,
			rtic_monotonics::create_systick_token!(),
		);

		watchdog::spawn().ok();

		defmt::info!("Initialisation finished.");

		(
			Shared {},
			Local {watchdog},
		)		
	}	

	#[task(local = [watchdog])]
	async fn watchdog(cx: watchdog::Context) {
		loop {
			cx.local.watchdog.feed();
			Systick::delay(80_u64.millis()).await;
		}
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
