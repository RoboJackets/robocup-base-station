//!
//! Test program to tell a specific robot to move in a straight line
//! 

use clap::Parser;

use std::{error::Error, thread, time::Duration};

use robojackets_robocup_rtp::{control_message::ControlMessageBuilder, Team};

use robocup_base_station::{nrf_pubsub::{IncomingMessage, NrfPublisherSubscriber, NrfSendError, Packet}, RADIO_ONE_CE, RADIO_ONE_CSN};

use ncomm::prelude::*;

use rppal::{
    gpio::Gpio,
    spi::{self, Spi, SimpleHalSpiDevice},
};

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// The id of the robot to send commands to
    #[arg(short, long, default_value_t = 0)]
    pub robot_id: u8,

    /// Is the robot on the blue team
    #[arg(short, long, default_value_t = true)]
    pub blue: bool,

    /// Is the robot on the yellow team
    #[arg(short, long, default_value_t = false)]
    pub yellow: bool,

    /// Should the robot be moving forwards
    #[arg(short, long, default_value_t = true)]
    pub forward: bool,

    /// Should the robot be moving to the right
    #[arg(short, long, default_value_t = false)]
    pub right: bool,

    /// Should the robot be moving backwards
    #[arg(short, long, default_value_t = false)]
    pub backward: bool,

    /// Should the robot be moving left
    #[arg(short, long, default_value_t = false)]
    pub left: bool,

    /// Should the robot be spinning clockwise
    #[arg(short, long, default_value_t = false)]
    pub clockwise: bool,

    /// Should the robot be spinning counter-clockwise
    #[arg(short, long, default_value_t = false)]
    pub counter_clockwise: bool,

    /// The velocity the robot should move at
    #[arg(short, long, default_value_t = 1.0)]
    pub velocity: f32,
}

fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    let team = if args.yellow {
        Team::Yellow
    } else {
        Team::Blue
    };

    let mut builder = ControlMessageBuilder::new()
        .robot_id(args.robot_id)
        .team(team);

    if args.right {
        builder = builder.body_x(args.velocity);
    } else if args.backward {
        builder = builder.body_y(-args.velocity);
    } else if args.left {
        builder = builder.body_x(-args.velocity);
    } else if args.clockwise {
        builder = builder.body_w(args.velocity);
    } else if args.counter_clockwise {
        builder = builder.body_w(-args.velocity);
    } else {
        builder = builder.body_y(args.velocity);
    }

    let command = builder.build();

    let gpio = Gpio::new().unwrap();
    let spi = SimpleHalSpiDevice::new(Spi::new(spi::Bus::Spi0, spi::SlaveSelect::Ss0, 5_000_000, rppal::spi::Mode::Mode0).unwrap());
    let csn = gpio.get(RADIO_ONE_CSN).unwrap().into_output();
    let ce = gpio.get(RADIO_ONE_CE).unwrap().into_output();

    let mut radio_pubsub = NrfPublisherSubscriber::new(team, spi, csn, ce).unwrap();

    loop {
        match radio_pubsub.publish(Packet { robot_id: args.robot_id, data: command.clone() }) {
            Ok(_) => println!("Robot {} Received the Message", args.robot_id),
            Err(err) => match err {
                NrfSendError::Timeout => println!("Message Timed Out"),
            }
        }

        for _ in 0..5 {
            match radio_pubsub.get() {
                Ok(message) => {
                    match message {
                        IncomingMessage::RobotStatus(status) => {
                            println!("Robot Status Received");
                            println!("{:?}", status);
                            break;
                        },
                        _ => println!("Received: {:?}", message),
                    }
                },
                Err(err) => match err {
                    NrfSendError::Timeout => println!("Receiving Timed Out"),
                }
            }
        }

        thread::sleep(Duration::from_millis(100));
    }
}