#![no_main]
#![no_std]

// Setting up entry vector/panic handler and logging
use cortex_m_rt::entry;
use defmt_rtt as _;
use panic_probe as _;
// Imports for the shared bus
use core::cell::RefCell;
use embedded_hal_bus::spi::{NoDelay, RefCellDevice};
// Hal imports
use hal::prelude::*;
use hal::spi::{Mode, Spi};
use stm32f4xx_hal as hal;

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
    // We are using the ~SYNC pin as CS with ~LDAC tied to ground this puts
    // the device in individual update mode, loading the dac register on each
    // write operation
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

    use ad57xx::Ad57xxShared;
    // Create a new AD57x4 SpiDevice
    let mut dac = Ad57xxShared::new_ad57x4(RefCellDevice::new(&spi_bus, spi3_dac_sync, NoDelay));

    // Setup the DAC as desired.
    dac.set_power(ad57xx::ad57x4::Channel::AllDacs, true)
        .unwrap();
    dac.set_output_range(
        ad57xx::ad57x4::Channel::AllDacs,
        ad57xx::OutputRange::Bipolar5V,
    )
    .unwrap();
    // Output a value (left-aligned 16 bit)
    dac.set_dac_output(ad57xx::ad57x4::Channel::DacA, 0x9000)
        .unwrap();
    let mut val: u16 = 0x0000;
    loop {
        // Output a stepped voltage
        delay.delay_ms(250);
        dac.set_dac_output(ad57xx::ad57x4::Channel::DacA, val)
            .unwrap();
        val = val.wrapping_add(0x1000);
        continue;
    }
}
