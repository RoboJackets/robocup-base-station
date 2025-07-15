//!
//! My attempt at rewriting the base station using the tokio runtime
//! 

use clap::Parser;

use crossbeam::channel::unbounded;
use ncomm::utils::packing::Packable;
use quanta::{Clock, Instant};
use robocup_base_station::{
    BASE_AMPLIFICATION_LEVEL, RADIO_ONE_CE, RADIO_ONE_CSN, RADIO_ONE_CHANNEL, RADIO_TWO_CSN,
    RADIO_TWO_CE, RADIO_TWO_CHANNEL
};
use robojackets_robocup_rtp::{ControlMessage, ControlMessageBuilder, RobotStatusMessage, BASE_STATION_ADDRESSES, CONTROL_MESSAGE_SIZE, ROBOT_RADIO_ADDRESSES, ROBOT_STATUS_SIZE};
use rppal::{
    gpio::Gpio,
    hal::Delay,
    spi::{self, SimpleHalSpiDevice, Spi},
};
use rtic_nrf24l01::Radio;
use tokio::{net::UdpSocket, time};
use std::time::Duration;

const RECEIVE_DURATION_US: u64 = 5_000;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The IPv4 Address of the field computer
    #[arg(default_value_t = String::from("10.42.0.1"))]
    pub field_computer_address: String,

    /// The port to look for control messages on
    #[arg(default_value_t = 8000)]
    pub control_message_port: u16,

    /// THe port to send robot statuses over
    #[arg(default_value_t = 8001)]
    pub robot_status_port: u16,

    /// The port to send alive robots messages over
    #[arg(default_value_t = 8002)]
    pub alive_robots_port: u16,

    /// The maximum timeout before we assume a robot is dead
    #[arg(default_value_t = 500)]
    pub timeout_ms: u128,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // parse args
    let args = Args::parse();

    // Initialize peripherals
    println!("Initializing Peripherals");
    let gpio = Gpio::new()
        .expect("Unable to take the GPIO peripherals");

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

    // Bind UDP Sockets
    println!("Binding Sockets");
    let control_message_socket = UdpSocket::bind(format!("0.0.0.0:{}", args.control_message_port)).await?;
    let robot_status_socket = UdpSocket::bind(format!("0.0.0.0:{}", args.robot_status_port)).await?;
    let alive_robots_socket = UdpSocket::bind(format!("0.0.0.0:{}", args.alive_robots_port)).await?;

    let (r0_ctrl_tx, r0_ctrl_rx) = unbounded::<ControlMessage>();
    let (r1_ctrl_tx, r1_ctrl_rx) = unbounded::<ControlMessage>();
    let (r2_ctrl_tx, r2_ctrl_rx) = unbounded::<ControlMessage>();
    let (r3_ctrl_tx, r3_ctrl_rx) = unbounded::<ControlMessage>();
    let (r4_ctrl_tx, r4_ctrl_rx) = unbounded::<ControlMessage>();
    let (r5_ctrl_tx, r5_ctrl_rx) = unbounded::<ControlMessage>();

    let (r0_status_tx, r0_status_rx) = unbounded::<RobotStatusMessage>();
    let (r1_status_tx, r1_status_rx) = unbounded::<RobotStatusMessage>();
    let (r2_status_tx, r2_status_rx) = unbounded::<RobotStatusMessage>();
    let (r3_status_tx, r3_status_rx) = unbounded::<RobotStatusMessage>();
    let (r4_status_tx, r4_status_rx) = unbounded::<RobotStatusMessage>();
    let (r5_status_tx, r5_status_rx) = unbounded::<RobotStatusMessage>();

    let (r0_time_tx, r0_time_rx) = unbounded::<Instant>();
    let (r1_time_tx, r1_time_rx) = unbounded::<Instant>();
    let (r2_time_tx, r2_time_rx) = unbounded::<Instant>();
    let (r3_time_tx, r3_time_rx) = unbounded::<Instant>();
    let (r4_time_tx, r4_time_rx) = unbounded::<Instant>();
    let (r5_time_tx, r5_time_rx) = unbounded::<Instant>();

    let (interrupt_tx, interrupt_rx) = unbounded();
    ctrlc::set_handler(move || {
        interrupt_tx.send(true).unwrap();
    }).expect("Unable to set ctrl-c handler");

    let ctrl_irq = interrupt_rx.clone();
    let control_message_relay = tokio::spawn(async move {
        let mut recv_buffer = [0u8; CONTROL_MESSAGE_SIZE];
        let mut interrupted = false;
        while !interrupted {
            if let Ok(interrupt) = ctrl_irq.try_recv() {
                interrupted = interrupt;
            }

            if let Ok(n_bytes) = control_message_socket.recv(&mut recv_buffer).await {
                if n_bytes == CONTROL_MESSAGE_SIZE {
                    if let Ok(data) = ControlMessage::unpack(&recv_buffer) {
                        let _ = match data.robot_id {
                            0 => r0_ctrl_tx.send(data),
                            1 => r1_ctrl_tx.send(data),
                            2 => r2_ctrl_tx.send(data),
                            3 => r3_ctrl_tx.send(data),
                            4 => r4_ctrl_tx.send(data),
                            _ => r5_ctrl_tx.send(data),
                        };
                    }
                }
            }
        }
    });

    let status_irq = interrupt_rx.clone();
    let status_receivers = [
        r0_status_rx,
        r1_status_rx,
        r2_status_rx,
        r3_status_rx,
        r4_status_rx,
        r5_status_rx
    ];
    let robot_status_send_address = format!("{}:{}", args.field_computer_address.clone(), args.robot_status_port);
    let robot_status_relay = tokio::spawn(async move {
        let mut send_buffer = [0u8; ROBOT_STATUS_SIZE];
        let mut interrupted = false;

        while !interrupted {
            if let Ok(interrupt) = status_irq.try_recv() {
                interrupted = interrupt;
            }

            for robot_id in 0..6 {
                if let Ok(status) = status_receivers[robot_id].try_recv() {
                    status.pack(&mut send_buffer).unwrap();
                    let _ = robot_status_socket.send_to(
                        &send_buffer,
                        &robot_status_send_address,
                    ).await;
                }
            }
        }
    });

    let alive_irq = interrupt_rx.clone();
    let alive_rxs = [
        r0_time_rx,
        r1_time_rx,
        r2_time_rx,
        r3_time_rx,
        r4_time_rx,
        r5_time_rx,
    ];
    let alive_robots_send_address = format!("{}:{}", args.field_computer_address.clone(), args.alive_robots_port);
    let alive_robots_relay = tokio::spawn(async move {
        let mut send_buffer = [0u8; 2];
        let mut interrupted = false;
        let mut last_received = [None; 6];
        let ref_clock = Clock::new();

        while !interrupted {
            if let Ok(interrupt) = alive_irq.try_recv() {
                interrupted = interrupt;
            }

            send_buffer[0] = 0;
            for robot_id in 0..6 {
                while let Ok(time) = alive_rxs[robot_id].try_recv() {
                    last_received[robot_id] = Some(time);
                }

                if let Some(last_time) = last_received[robot_id] {
                    if ref_clock.now() < last_time + Duration::from_millis(args.timeout_ms as u64) {
                        send_buffer[0] |= 1 << robot_id;
                    }
                }
            }

            let _ = alive_robots_socket.send_to(
                &send_buffer,
                &alive_robots_send_address
            );

            time::sleep(Duration::from_millis(args.timeout_ms as u64)).await;
        }
    });

    let radio1_irq = interrupt_rx.clone();
    let radio_one_rxs = [
        r0_ctrl_rx,
        r1_ctrl_rx,
        r2_ctrl_rx
    ];
    let radio_one_txs = [
        (r0_status_tx, r0_time_tx),
        (r1_status_tx, r1_time_tx),
        (r2_status_tx, r2_time_tx)
    ];
    let radio_one = tokio::spawn(async move {
        let mut send_buffer = [0u8; CONTROL_MESSAGE_SIZE];
        let mut rx_buffer = [0u8; ROBOT_STATUS_SIZE];
        let mut interrupted = false;
        let mut robot_data = [
            ControlMessageBuilder::new()
                .robot_id(0)
                .build(),
            ControlMessageBuilder::new()
                .robot_id(1)
                .build(),
            ControlMessageBuilder::new()
                .robot_id(2)
                .build()
        ];
        let ref_clock = Clock::new();
        while !interrupted {
            if let Ok(interrupt) = radio1_irq.try_recv() {
                interrupted = interrupt;
            }

            for robot_id in 0..3 {
                // Update data to sent to robot
                while let Ok(data) = radio_one_rxs[robot_id].try_recv() {
                    robot_data[robot_id] = data;
                }

                // Send data to robot
                robot_data[robot_id].clone().pack(&mut send_buffer).unwrap();
                radio1.stop_listening(&mut spi1, &mut delay1);
                radio1.open_writing_pipe(
                    ROBOT_RADIO_ADDRESSES[0][robot_id],
                    &mut spi1,
                    &mut delay1
                );
                radio1.set_payload_size(ControlMessage::len() as u8, &mut spi1, &mut delay1);
                radio1.write(&send_buffer, &mut spi1, &mut delay1);

                // Receive data from robot
                radio1.set_payload_size(RobotStatusMessage::len() as u8, &mut spi1, &mut delay1);
                radio1.start_listening(&mut spi1, &mut delay1);

                let wait_time = ref_clock.now() + Duration::from_micros(RECEIVE_DURATION_US);
                while ref_clock.now() < wait_time {
                    if radio1.available(&mut spi1, &mut delay1) {
                        radio1.read(&mut rx_buffer, &mut spi1, &mut delay1);
                        let message = RobotStatusMessage::unpack(&rx_buffer).unwrap();
                        match message.robot_id {
                            0 => {
                                let _ = radio_one_txs[0].0.send(message);
                                let _ = radio_one_txs[0].1.send(ref_clock.now());
                            },
                            1 => {
                                let _ = radio_one_txs[1].0.send(message);
                                let _ = radio_one_txs[1].1.send(ref_clock.now());
                            },
                            2 => {
                                let _ = radio_one_txs[2].0.send(message);
                                let _ = radio_one_txs[2].1.send(ref_clock.now());
                            },
                            _ => (),
                        }
                    }
                }
            }
        }
    });

    let radio2_irq = interrupt_rx.clone();
    let radio_two_rxs = [
        r3_ctrl_rx,
        r4_ctrl_rx,
        r5_ctrl_rx,
    ];
    let radio_two_txs = [
        (r3_status_tx, r3_time_tx),
        (r4_status_tx, r4_time_tx),
        (r5_status_tx, r5_time_tx)
    ];
    let radio_two = tokio::spawn(async move {
        let mut send_buffer = [0u8; CONTROL_MESSAGE_SIZE];
        let mut rx_buffer = [0u8; ROBOT_STATUS_SIZE];
        let mut interrupted = false;
        let mut robot_data = [
            ControlMessageBuilder::new()
                .robot_id(3)
                .build(),
            ControlMessageBuilder::new()
                .robot_id(4)
                .build(),
            ControlMessageBuilder::new()
                .robot_id(5)
                .build()
        ];
        let ref_clock = Clock::new();
        while !interrupted {
            if let Ok(interrupt) = radio2_irq.try_recv() {
                interrupted = interrupt;
            }

            for robot_id in 3..6 {
                // Update data to send to the robot
                while let Ok(data) = radio_two_rxs[robot_id].try_recv() {
                    robot_data[robot_id] = data;
                }

                // Send data to robot
                robot_data[robot_id].clone().pack(&mut send_buffer).unwrap();
                radio2.stop_listening(&mut spi2, &mut delay2);
                radio2.open_writing_pipe(
                    ROBOT_RADIO_ADDRESSES[0][robot_id],
                    &mut spi2,
                    &mut delay2
                );
                radio2.set_payload_size(ControlMessage::len() as u8, &mut spi2, &mut delay2);
                radio2.write(&send_buffer, &mut spi2, &mut delay2);

                // Receive data from robot
                radio2.set_payload_size(RobotStatusMessage::len() as u8, &mut spi2, &mut delay2);
                radio2.start_listening(&mut spi2, &mut delay2);

                let wait_time = ref_clock.now() + Duration::from_micros(RECEIVE_DURATION_US);
                while ref_clock.now() < wait_time {
                    if radio2.available(&mut spi2, &mut delay2) {
                        radio2.read(&mut rx_buffer, &mut spi2, &mut delay2);
                        let message = RobotStatusMessage::unpack(&rx_buffer).unwrap();
                        match message.robot_id {
                            3 => {
                                let _ = radio_two_txs[0].0.send(message);
                                let _ = radio_two_txs[0].1.send(ref_clock.now());
                            },
                            4 => {
                                let _ = radio_two_txs[1].0.send(message);
                                let _ = radio_two_txs[1].1.send(ref_clock.now());
                            },
                            5 => {
                                let _ = radio_two_txs[2].0.send(message);
                                let _ = radio_two_txs[2].1.send(ref_clock.now());
                            },
                            _ => (),
                        }
                    }
                }
            }            
        }
    });

    let _ = control_message_relay.await;
    let _ = robot_status_relay.await;
    let _ = alive_robots_relay.await;
    let _ = radio_one.await;
    let _ = radio_two.await;

    Ok(())
}