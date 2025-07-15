//!
//! Send data every 50ms so we can benchmark the radios
//! 

use crossbeam::channel::unbounded;
use robocup_base_station::{
    BASE_AMPLIFICATION_LEVEL, RADIO_ONE_CE, RADIO_ONE_CSN, RADIO_ONE_CHANNEL, RADIO_TWO_CSN,
    RADIO_TWO_CE, RADIO_TWO_CHANNEL
};

use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{self, SimpleHalSpiDevice, Spi},
};

use robojackets_robocup_rtp::{ControlMessage, ControlMessageBuilder, RobotStatusMessage, BASE_STATION_ADDRESSES, CONTROL_MESSAGE_SIZE, ROBOT_RADIO_ADDRESSES, ROBOT_STATUS_SIZE};

use rtic_nrf24l01::Radio;

use ncomm::utils::packing::Packable;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let gpio = Gpio::new()?;

    // Initialize Radio 1
    println!("Initializing Radio 1");
    let mut spi1 = SimpleHalSpiDevice::new(
        Spi::new(
            spi::Bus::Spi0,
            spi::SlaveSelect::Ss0,
            1_000_000,
            spi::Mode::Mode0
        ).expect("Unable to iniitalize the radios' spi")
    );
    let mut delay1 = Delay::new();
    let csn = gpio.get(RADIO_ONE_CSN)
        .expect("Unable to take radio one's csn pin")
        .into_output();
    let ce = gpio.get(RADIO_ONE_CE)
        .expect("Unable to take radio one's ce pin")
        .into_output();
    let mut radio1 = Radio::new(ce, csn);
    radio1.begin(&mut spi1, &mut delay1)
        .expect("Unable to start radio one");
    radio1.set_pa_level(BASE_AMPLIFICATION_LEVEL, &mut spi1, &mut delay1);
    radio1.set_payload_size(RobotStatusMessage::len() as u8, &mut spi1, &mut delay1);
    radio1.set_channel(RADIO_ONE_CHANNEL, &mut spi1, &mut delay1);
    radio1.stop_listening(&mut spi1, &mut delay1);
    radio1.open_reading_pipe(
        1,
        BASE_STATION_ADDRESSES[0],
        &mut spi1,
        &mut delay1
    );
    radio1.open_writing_pipe(
        ROBOT_RADIO_ADDRESSES[0][0],
        &mut spi1,
        &mut delay1
    );

    // Initialize Radio 2
    println!("Initializing Radio 2");
    let mut spi2 = SimpleHalSpiDevice::new(
        Spi::new(
            spi::Bus::Spi1,
            spi::SlaveSelect::Ss0,
            1_000_000,
            spi::Mode::Mode0
        ).expect("Unable to initialize radio 2's spi")
    );
    let mut delay2 = Delay::new();
    let csn = gpio.get(RADIO_TWO_CSN)
        .expect("Unabel to take radio two's csn pin")
        .into_output();
    let ce = gpio.get(RADIO_TWO_CE)
        .expect("Unable to take radio two's ce pin")
        .into_output();
    let mut radio2 = Radio::new(ce, csn);
    radio2.begin(&mut spi2, &mut delay2)
        .expect("Unable to start radio two");
    radio2.set_pa_level(BASE_AMPLIFICATION_LEVEL, &mut spi2, &mut delay2);
    radio2.set_payload_size(RobotStatusMessage::len() as u8, &mut spi2, &mut delay2);
    radio2.set_channel(RADIO_TWO_CHANNEL, &mut spi2, &mut delay2);
    radio2.stop_listening(&mut spi2, &mut delay2);
    radio2.open_reading_pipe(
        1,
        BASE_STATION_ADDRESSES[0],
        &mut spi2,
        &mut delay2
    );
    radio2.open_writing_pipe(
        ROBOT_RADIO_ADDRESSES[0][3],
        &mut spi2,
        &mut delay2
    );

    let (interrupt_tx, interrupt_rx) = unbounded();
    ctrlc::set_handler(move || {
        interrupt_tx.send(true).unwrap();
    })?;

    let r1_irq = interrupt_rx.clone();
    let r1 = tokio::spawn(async move {
        let mut interrupted = false;
        while !interrupted {
            if let Ok(interrupt) = r1_irq.try_recv() {
                interrupted = interrupt;
            }

            // TODO: Receive Data
        }
    });

    let r2_irq = interrupt_rx.clone();
    let r2 = tokio::spawn(async move {
        let mut interrupted = false;
        while !interrupted {
            if let Ok(interrupt) = r2_irq.try_recv() {
                interrupted = interrupt;
            }

            // TODO: Receive Data
        }
    });

    r1.await?;
    r2.await?;

    Ok(())
}