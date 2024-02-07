# Driver for Analog Devices AD57xx series of dual and quad channel 16/14/12bit DACs
[![crates.io](https://img.shields.io/crates/v/ad57xx.svg)](https://crates.io/crates/ad57xx)
[![Docs](https://docs.rs/ad57xx/badge.svg)](https://docs.rs/ad57xx)

Compatibility has only been tested with the AD5754. However the only difference
between the different chips is the channel count and the bit-depth. 
Readback operation is currently untested as my hardware does not support it. 
If you are in the opportunity to do so please let me know your findings.

Any contribution to this crate is welcome, as it's my first published crate any 
feedback is appreciated.

## To-do:
 - [x] Register definitions
 - [x] Write functionality for all registers
 - [x] Minimal working example with a shared bus on stm32f4xx
 - [x] Dual channel chip support (untested)
 - [ ] #\[tests\] on target
 - [ ] Testing readback functionality
 - [ ] Exclusive device struct
 - [ ] Support daisy-chain operation
 - [ ] Async support

## Usage example
```rust
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
    Mod
    1.MHz(),
    &ccdr,
);
// SPI Bus creation using embedded-hal-bus
let spi_bus = RefCell::new(spi3);

// Creating shared DAC SPI Device
let mut dac = Ad57xxShared::new_ad57x4(RefCellDevice::new(&spi_bus, spi3_dac_sync, NoDelay));

// Setup the DAC as desired.
dac.set_power(ad57xx::ad57x4::Channel::AllDacs, true).unwrap();
dac.set_output_range(ad57xx::ad57x4::Channel::AllDacs, ad57xx::OutputRange::Bipolar5V)
    .unwrap();
// Output a value (left-aligned 16 bit)
dac.set_dac_output(ad57xx::ad57x4::Channel::DacA, 0x9000).unwrap();
```
