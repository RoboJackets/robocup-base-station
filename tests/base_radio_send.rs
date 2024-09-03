use std::{thread, time::Duration};

use rppal::{spi::{Spi, Bus, SlaveSelect, Mode}, gpio::Gpio, hal::Delay};

use rtic_nrf24l01::Radio;
use rtic_nrf24l01::config::*;

use packed_struct::PackedStruct;

use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::control_message::{ControlMessageBuilder, CONTROL_MESSAGE_SIZE, TriggerMode, ShootMode};

#[test]
fn test_base_radio_send_hello() {
    let mut spi = Spi::new(Bus::Spi0, SlaveSelect::Ss0, 1_000_000, Mode::Mode0).unwrap();
    let gpio = Gpio::new().unwrap();
    let csn = gpio.get(8).unwrap().into_output();
    let ce = gpio.get(22).unwrap().into_output();
    let mut delay = Delay::new();

    let mut radio = Radio::new(ce, csn);

    if let Err(err) = radio.begin(&mut spi, &mut delay) {
        println!("Configuration: {:?}", radio.get_registers(&mut spi, &mut delay));
        thread::sleep(Duration::from_millis(500));
        println!("Configuration: {:?}", radio.get_registers(&mut spi, &mut delay));
        panic!("Unable to initialize the radio: {:?}", err);
    }

    radio.set_pa_level(power_amplifier::PowerAmplifier::PALow, &mut spi, &mut delay);

    radio.set_payload_size(CONTROL_MESSAGE_SIZE as u8, &mut spi, &mut delay);

    radio.open_writing_pipe([0xC3, 0xC3, 0xC3, 0xC3, 0xC1], &mut spi, &mut delay);

    radio.open_reading_pipe(1, [0xE7, 0xE7, 0xE7, 0xE7, 0xE7], &mut spi, &mut delay);

    radio.start_listening(&mut spi, &mut delay);

    thread::sleep(Duration::from_millis(1_000));

    println!("Configuration: {:?}", radio.get_registers(&mut spi, &mut delay));
    
    radio.stop_listening(&mut spi, &mut delay);

    let control_message = ControlMessageBuilder::new()
        .team(Team::Blue)
        .robot_id(0)
        .shoot_mode(ShootMode::Chip)
        .trigger_mode(TriggerMode::OnBreakBeam)
        .body_x(20.0)
        .body_y(0.0)
        .body_w(0.0)
        .dribbler_speed(-5)
        .kick_strength(3)
        .role(1)
        .build();

    let packed_data = match control_message.pack() {
        Ok(bytes) => bytes,
        Err(_err) => panic!("Unable to pack message"),
    };

    let report = radio.write(&packed_data, &mut spi, &mut delay);

    if report {
        println!("Successfully sent report");
    } else {
        println!("Unable to send data");
    }

}