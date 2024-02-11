use ad57xx::{marker, Ad57xx, Ad57xxShared};
use embedded_hal::spi::SpiDevice;
use embedded_hal_mock::common::Generic;
use embedded_hal_mock::eh1::spi::{Mock as MockSpi, Transaction as MockTransaction};
use std::fmt::Debug;

#[test]
fn write_dac_b() {
    // Quad channel test
    let mut trans = [
        MockTransaction::transaction_start(),
        MockTransaction::write_vec(vec![0b00000001, 0xF0, 0x0F]),
        MockTransaction::transaction_end(),
    ];
    let mut spi = MockSpi::new(&trans);

    let mut dac = Ad57xxShared::new_ad57x4(spi);
    dac.set_dac_output(ad57xx::ad57x4::ChannelQuad::DacB, 0xF00F);
    dac.destroy().done();

    // Dual channel test
    let mut trans = [
        MockTransaction::transaction_start(),
        MockTransaction::write_vec(vec![0b00000010, 0xF0, 0x0F]),
        MockTransaction::transaction_end(),
    ];
    let mut spi = MockSpi::new(&trans);

    let mut dac = Ad57xxShared::new_ad57x2(spi);
    dac.set_dac_output(ad57xx::ad57x2::Channel::DacB, 0xF00F);
    dac.destroy().done();
}
#[test]
fn read_config() {
    let trans = [
        MockTransaction::transaction_start(),
        MockTransaction::write_vec(vec![0b10011001, 0x00, 0x00]),
        MockTransaction::transaction_end(),
        MockTransaction::transaction_start(),
        MockTransaction::transfer(vec![0b00011000, 0x00, 0x00], vec![0x04, 0x00, 0x00]),
        MockTransaction::transaction_end(),
    ];
    let mut spi = MockSpi::new(&trans);

    let mut dac = Ad57xxShared::new_ad57x4(spi);
    assert_eq!(
        u8::from(dac.get_config().unwrap()),
        u8::from(ad57xx::Config::default())
    );
    dac.destroy().done();
}
