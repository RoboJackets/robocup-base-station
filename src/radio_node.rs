//!
//! The Radio Node is a single-radio solution to the sending and receiving of 
//! communication for the robots.
//! 

use super::NodeIdentifier;

use std::time::SystemTime;

use ncomm::prelude::*;
use ncomm::pubsubs::{local::{LocalPublisher, LocalSubscriber, LocalMappedSubscriber}, udp::{UdpMappedSubscriber, UdpPublisher}};

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;

use robojackets_robocup_rtp::control_message::{ControlMessage, ControlMessageBuilder, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::robot_status_message::RobotStatusMessage;
use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::{BASE_STATION_ADDRESS, ROBOT_RADIO_ADDRESSES};

use rtic_nrf24l01::Radio;

use crate::nrf_pubsub::NrfPublisherSubscriber;
use crate::{BASE_AMPLIFICATION_LEVEL, CHANNEL};

pub fn id_from_message(message: &ControlMessage) -> u8 {
    message.robot_id
}

pub fn subscriber_map(data: &Option<u8>) -> u8 {
    data.unwrap_or_default()
}

pub struct RadioNode<
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SPIE,
    GPIOE,
> {
    team: Team,
    num_robots: u8,
    control_message_subscriber: UdpMappedSubscriber<ControlMessage, u8, fn(&ControlMessage) -> u8>,
    radio_publisher_subscriber: NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE>,
    robot_status_publisher: UdpPublisher<RobotStatusMessage>,
    receive_message_publisher: LocalPublisher<u8>,
    alive_robots_intra_subscriber: Option<LocalSubscriber<u16>>,
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE> RadioNode<SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
{
    pub fn new(
        team: Team,
        num_robots: u8,
        ce: CE,
        csn: CSN,
        mut spi: SPI,
        mut delay: DELAY,
        control_message_bind_address: String,
        robot_status_bind_address: String,
        robot_status_send_address: String,
    ) -> Self {
        let mut radio = Radio::new(ce, csn);
        if radio.begin(&mut spi, &mut delay).is_err() {
            panic!("Unable to Initialize the radio");
        }
        radio.set_pa_level(BASE_AMPLIFICATION_LEVEL, &mut spi, &mut delay);
        radio.set_payload_size(CONTROL_MESSAGE_SIZE as u8, &mut spi, &mut delay);
        radio.set_channel(CHANNEL, &mut spi, &mut delay);
        radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[0], &mut spi, &mut delay);
        radio.open_reading_pipe(1, BASE_STATION_ADDRESS, &mut spi, &mut delay);
        radio.start_listening(&mut spi, &mut delay);
        delay.delay_ms(1_000);
        radio.stop_listening(&mut spi, &mut delay);

        let control_message_subscriber = UdpMappedSubscriber::new(
            control_message_bind_address.parse().unwrap(),
            id_from_message as fn(&ControlMessage) -> u8,
        ).unwrap();
        let radio_publisher_subscriber = NrfPublisherSubscriber::new(radio, spi, delay);
        let robot_status_publisher = UdpPublisher::new(
            robot_status_bind_address.parse().unwrap(),
            vec![robot_status_send_address.parse().unwrap()],
        ).unwrap();
        let receive_message_publisher = LocalPublisher::new();

        Self {
            team: team,
            num_robots,
            control_message_subscriber,
            radio_publisher_subscriber,
            robot_status_publisher,
            receive_message_publisher,
            alive_robots_intra_subscriber: None,
        }
    }

    pub fn create_subscriber(&mut self) -> LocalMappedSubscriber<u8, u8, fn(&Option<u8>) -> u8> {
        self.receive_message_publisher.subscribe_mapped(subscriber_map as fn(&Option<u8>) -> u8)
    }

    pub fn add_alive_robots_intra_publisher(&mut self, publisher: LocalSubscriber<u16>) {
        self.alive_robots_intra_subscriber = Some(publisher);
    }

    fn send_and_await_response(&mut self, control_message: ControlMessage, robot_id: u8) {
        // Send Control Message
        let _ = self.radio_publisher_subscriber.publish(control_message);

        let start_instant = SystemTime::now();
        while SystemTime::now().duration_since(start_instant).unwrap().as_millis() < 3 {
            for data in self.radio_publisher_subscriber.get() {
                self.robot_status_publisher.publish(*data).unwrap();
                self.receive_message_publisher.publish(robot_id).unwrap();
            }
        }
    }
}

impl<SPI, CSN, CE, DELAY, SPIE, GPIOE> Node<NodeIdentifier> for RadioNode<SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
{
    fn get_id(&self) -> NodeIdentifier {
        NodeIdentifier::Radio1
    }

    fn get_update_delay_us(&self) -> u128 {
        100
    }

    fn start(&mut self) { }

    fn update(&mut self) {
        // For each robot, send them a control message and wait for a response
        for robot_id in 0..self.num_robots {
            let control_message = match self.control_message_subscriber.get().get(&robot_id) {
                Some(control_message) => *control_message,
                None => ControlMessageBuilder::new().team(self.team).build(),
            };
            self.send_and_await_response(control_message, robot_id);
        }
    }

    // TODO: Possibly Reset Radio Status on Shutdown (?)
    fn shutdown(&mut self) {}
}
