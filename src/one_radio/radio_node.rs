//!
//! The Radio Node is a single-radio solution to the sending and receiving of 
//! communication for the robots.
//! 

use std::time::SystemTime;
use std::sync::Arc;

use ncomm::node::Node;
use ncomm::publisher_subscriber::{Receive, Publish};
use ncomm::publisher_subscriber::local::{LocalPublisher, LocalSubscriber, MappedLocalSubscriber};
use ncomm::publisher_subscriber::packed_udp::{MappedPackedUdpSubscriber, PackedUdpPublisher};

use embedded_hal::blocking::{spi::{Transfer, Write}, delay::{DelayMs, DelayUs}};
use embedded_hal::digital::v2::OutputPin;

use robojackets_robocup_rtp::control_message::{ControlMessage, ControlMessageBuilder, CONTROL_MESSAGE_SIZE};
use robojackets_robocup_rtp::robot_status_message::RobotStatusMessage;
use robojackets_robocup_rtp::Team;
use robojackets_robocup_rtp::{BASE_STATION_ADDRESS, ROBOT_RADIO_ADDRESSES};

use rtic_nrf24l01::Radio;

use crate::publishers::nrf_pubsub::NrfPublisherSubscriber;
use crate::{BASE_AMPLIFICATION_LEVEL, CHANNEL};

pub struct RadioNode<
    'a,
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>,
    SPIE,
    GPIOE,
> {
    team: Team,
    num_robots: u8,
    control_message_subscriber: MappedPackedUdpSubscriber<ControlMessage, u8, 10>,
    radio_publisher_subscriber: NrfPublisherSubscriber<SPI, CSN, CE, DELAY, SPIE, GPIOE>,
    robot_status_publisher: PackedUdpPublisher<'a, RobotStatusMessage>,
    receive_message_publisher: LocalPublisher<u8>,
    alive_robots_intra_subscriber: Option<LocalSubscriber<u16>>,
}

impl<'a, SPI, CSN, CE, DELAY, SPIE, GPIOE> RadioNode<'a, SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>
{
    pub fn new(
        team: Team,
        num_robots: u8,
        ce: CE,
        csn: CSN,
        mut spi: SPI,
        mut delay: DELAY,
        control_message_bind_address: &'a str,
        robot_status_bind_address: &'a str,
        robot_status_send_address: &'a str,
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

        let control_message_subscriber = MappedPackedUdpSubscriber::new(
            control_message_bind_address,
            None,
            Arc::new(|message: &ControlMessage| { *message.robot_id })
        );
        let radio_publisher_subscriber = NrfPublisherSubscriber::new(radio, spi, delay);
        let robot_status_publisher = PackedUdpPublisher::new(
            robot_status_bind_address,
            vec![robot_status_send_address],
        );
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

    pub fn create_subscriber(&mut self) -> MappedLocalSubscriber<u8, u8> {
        self.receive_message_publisher.create_mapped_subscriber(Arc::new(|data| { *data }))
    }

    pub fn add_alive_robots_intra_publisher(&mut self, publisher: LocalSubscriber<u16>) {
        self.alive_robots_intra_subscriber = Some(publisher);
    }

    fn send_and_await_response(&mut self, control_message: ControlMessage, robot_id: u8) {
        // Send Control Message
        self.radio_publisher_subscriber.send(control_message);

        let start_instant = SystemTime::now();
        while SystemTime::now().duration_since(start_instant).unwrap().as_nanos() < 3_000 {
            self.radio_publisher_subscriber.update_data();
            if self.radio_publisher_subscriber.data.len() > 0 {
                for data in self.radio_publisher_subscriber.data.drain(..) {
                    self.robot_status_publisher.send(data);
                    self.receive_message_publisher.send(*data.robot_id);
                    if *data.robot_id == robot_id {
                        return;
                    }
                }
            }
        }
    }
}

impl<'a, SPI, CSN, CE, DELAY, SPIE, GPIOE> Node for RadioNode<'a, SPI, CSN, CE, DELAY, SPIE, GPIOE> where
    SPI: Transfer<u8, Error=SPIE> + Write<u8, Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    DELAY: DelayMs<u32> + DelayUs<u32>
{
    fn name(&self) -> String { String::from("CPU --> Base Station --> Radio --> Base Station --> CPU")}

    // Tweak this value, but I think sending a wave of commands every 50 milliseconds is not bad
    fn get_update_delay(&self) -> u128 { 50u128 }

    fn start(&mut self) { }

    fn update(&mut self) {
        self.control_message_subscriber.update_data();

        // For each robot, send them a control message and wait for a response
        for robot_id in 0..self.num_robots {
            if let Some(control_message) = self.control_message_subscriber.data.get(&robot_id) {
                self.send_and_await_response(*control_message, robot_id);
            } else if let Some(subscriber) = self.alive_robots_intra_subscriber.as_ref() {
                // The robot might be considered dead, but we should still check in with him.
                if let Some(alive_robots) = subscriber.data {
                    if alive_robots & (1 << robot_id) == 0 {
                        let blank_control_message = ControlMessageBuilder::new()
                            .team(self.team)
                            .build();

                        self.send_and_await_response(blank_control_message, robot_id);
                    }
                }
            }
        }
    }

    // TODO: Possibly Reset Radio Status on Shutdown (?)
    fn shutdown(&mut self) {}

    fn debug(&self) -> String { self.name() }
}
