#![no_main]
#![no_std]

use core::cell::RefCell;

use embedded_hal_bus::spi::{NoDelay, RefCellDevice};
use hal::prelude::*;
use hal::spi::{Mode, Spi};

use cortex_m_rt::entry;
use stm32f4xx_hal as hal;

use defmt_rtt as _;
use panic_probe as _;

#[entry]
fn main() -> ! {
    // Take peripherals and set up the clocks.
    let p = hal::pac::Peripherals::take().unwrap();
    let pc = cortex_m::Peripherals::take().unwrap();
    let rcc = p.RCC.constrain();
    let ccdr = rcc.cfgr.freeze();
    // Create a SysTick based delay
    let mut delay = cortex_m::delay::Delay::new(pc.SYST, ccdr.sysclk().raw());

    // Setup the DAC's SPI bus and SYNC pin
    let gpioc = p.GPIOC.split();
    let spi3_sclk = gpioc.pc10.into_alternate();
    let spi3_miso = gpioc.pc11.into_alternate();
    let spi3_mosi = gpioc.pc12.into_alternate();
    let gpioa = p.GPIOA.split();
    let spi3_dac_sync = gpioa
        .pa15
        .into_push_pull_output_in_state(hal::gpio::PinState::High);
    // SPI Instance initialization in MODE 2
    let spi3 = Spi::new(
        p.SPI3,
        (spi3_sclk, spi3_miso, spi3_mosi),
        Mode {
            phase: hal::spi::Phase::CaptureOnFirstTransition,
            polarity: hal::spi::Polarity::IdleHigh,
        },
        1.MHz(),
        &ccdr,
    );
    // SPI Bus creation using embedded-hal-bus
    let spi_bus = RefCell::new(spi3);

    // Creating shared DAC SPI Device
    use ad57xx::Ad57xxShared;
    let mut dac = Ad57xxShared::new(RefCellDevice::new(&spi_bus, spi3_dac_sync, NoDelay));

    // Setup the DAC as desired.
    dac.set_power(ad57xx::Channel::AllDacs, true).unwrap();
    dac.set_output_range(ad57xx::Channel::AllDacs, ad57xx::OutputRange::Bipolar5V)
        .unwrap();
    dac.set_dac_output(ad57xx::Channel::DacA, 0x9000).unwrap();
    let mut val: u16 = 0x0000;
    loop {
        delay.delay_ms(250);
        dac.set_dac_output(ad57xx::Channel::DacA, val).unwrap();

        val = val.wrapping_add(0x1000);
        continue;
    }
}
