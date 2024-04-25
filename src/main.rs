#![no_main]
#![no_std]
#![allow(clippy::transmute_ptr_to_ptr)]

mod init;
mod canbus;

use init::*;
use canbus::*;

// global logger
use defmt_rtt as _;
use panic_probe as _;
use stm32g4xx_hal as hal;

use fdcan::{frame::RxFrameInfo, FdCanControl, Fifo0, Fifo1, NormalOperationMode, Rx, Tx};
use hal::{
	can::{Can},
	gpio::{Output, Speed},
	independent_watchdog::IndependentWatchdog,
	pwr::PwrExt,
	rcc::{self, Config, RccExt, SysClockSrc},
	stm32::{self, Peripherals, interrupt, Interrupt},
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
	type FdCanMode = NormalOperationMode;

	#[shared]
	pub struct Shared {
		pub fdcan1_rx0: Rx<Can<stm32::FDCAN1>, FdCanMode, Fifo0>,
		pub fdcan1_rx1: Rx<Can<stm32::FDCAN1>, FdCanMode, Fifo1>
	}
	
	#[local]
	pub struct Local {
		pub watchdog: IndependentWatchdog,
	}	

	#[task(local = [watchdog])]
	async fn watchdog(cx: watchdog::Context) {
		loop {
			cx.local.watchdog.feed();
			Systick::delay(80_u64.millis()).await;
		}
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
