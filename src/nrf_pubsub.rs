//!
//! nRF24L01+ Radio Publisher and Receiver
//! 

use std::{
    collections::HashMap, fmt::Debug, marker::{PhantomData, Send}, time::Duration
};

use robojackets_robocup_rtp::{control_message::Mode, BASE_STATION_ADDRESSES};
use rppal::hal::Delay;

use ncomm::prelude::*;
use ncomm::utils::packing::{Packable, PackingError};

use rtic_nrf24l01::error::RadioError;
use rtic_nrf24l01::Radio;

use embedded_hal::{
    digital::OutputPin,
    spi::SpiDevice,
};

use quanta::Clock;

use robojackets_robocup_rtp::{
    ControlMessage, RobotStatusMessage, Team, ROBOT_RADIO_ADDRESSES,
    imu_test_message::ImuTestMessage,
    radio_benchmarks::{RadioSendBenchmarkMessage, RadioReceiveBenchmarkMessage},
    kicker_testing::KickerTestingMessage,
    kicker_program_message::KickerProgramMessage,
    control_test_message::ControlTestMessage,
};

use crate::{BASE_AMPLIFICATION_LEVEL, CHANNEL};

/// A packet published by the NrfPublisherSubscriber
pub struct Packet<T: Packable> {
    /// The id of the robot the packet should go to
    pub robot_id: u8,
    /// The data that should be sent to the robot
    pub data: T,
}

#[derive(Debug)]
pub enum IncomingMessage {
    RobotStatus(RobotStatusMessage),
    ImuTest(ImuTestMessage),
    RadioSend(RadioSendBenchmarkMessage),
    RadioReceive(RadioReceiveBenchmarkMessage),
    KickerTesting(KickerTestingMessage),
    KickerProgram(KickerProgramMessage),
    ControlTest(ControlTestMessage),
}

impl Packable for IncomingMessage {
    fn len() -> usize {
        0
    }

    fn pack(self, _buffer: &mut [u8]) -> Result<(), PackingError> {
        Ok(())
    }

    fn unpack(data: &[u8]) -> Result<Self, PackingError> {
        let message = RobotStatusMessage::unpack(data)?;
        Ok(IncomingMessage::RobotStatus(message))
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Errors from sending data via the NRF publisher / subscriber
pub enum NrfSendError {
    /// The data was not acknowledged by the receiving robot
    Timeout,
}

pub struct NrfPublisherSubscriber<
    SPI: SpiDevice<Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    GPIOE: Debug,
    SPIE: Debug,
> {
    /// The id of the team the base station will be sending to
    team: usize,
    /// The number of retries when attempting to transmit a piece of data
    retries: u8,
    /// The delay (in us) between successive retransmit attempts
    retry_delay: u64,
    /// The delay (in us) to block for while waiting for incoming data
    receive_block_time: u64,

    /// The current data mode of the radio
    data_mode: Mode,
    /// The most recent piece of data from a robot
    pub robot_data: HashMap<u8, Result<IncomingMessage, NrfSendError>>,
    /// The robot id of the most recent robot data was sent to
    current_robot_id: u8,

    radio: Radio<CE, CSN, SPI, GPIOE, SPIE>,
    spi: SPI,
    delay: Delay,
    clock: Clock,
    phantom: PhantomData<ControlMessage>,
}

impl<SPI, CSN, CE, GPIOE, SPIE> NrfPublisherSubscriber<SPI, CSN, CE, GPIOE, SPIE> where 
    SPI: SpiDevice<Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    GPIOE: Debug,
    SPIE: Debug,
{
    pub fn new(
        team: Team,
        mut spi: SPI,
        csn: CSN,
        ce: CE,
    ) -> Result<Self, RadioError> {
        let mut radio = Radio::new(ce, csn);

        // Setup Radio
        let mut delay = Delay::new();
        radio.begin(&mut spi, &mut delay)?;
        radio.set_pa_level(BASE_AMPLIFICATION_LEVEL, &mut spi, &mut delay);
        radio.set_payload_size(RobotStatusMessage::len() as u8, &mut spi, &mut delay);
        radio.set_channel(CHANNEL, &mut spi, &mut delay);
        radio.open_reading_pipe(1, ROBOT_RADIO_ADDRESSES[(team == Team::Yellow) as usize][0], &mut spi, &mut delay);
        radio.open_writing_pipe(BASE_STATION_ADDRESSES[(team == Team::Yellow) as usize], &mut spi, &mut delay);
        radio.stop_listening(&mut spi, &mut delay);

        Ok(Self {
            team: (team == Team::Yellow) as usize,
            retries: 3,
            retry_delay: 1_000,
            receive_block_time: 5_000,

            data_mode: Mode::Default,
            robot_data: HashMap::new(),
            current_robot_id: 0,

            radio,
            spi,
            delay,
            clock: Clock::new(),
            phantom: PhantomData,
        })
    }
}

impl<SPI, CSN, CE, GPIOE, SPIE> Publisher for NrfPublisherSubscriber<SPI, CSN, CE, GPIOE, SPIE> where 
    SPI: SpiDevice<Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    GPIOE: Debug,
    SPIE: Debug,
{
    type Data = Packet<ControlMessage>;
    type Error = NrfSendError;

    fn publish(&mut self, data: Self::Data) -> Result<(), Self::Error> {
        // Pack the data to send
        let mut buffer = vec![0u8; ControlMessage::len()];
        data.data.pack(&mut buffer).unwrap();
        
        // Send the data
        self.radio.stop_listening(&mut self.spi, &mut self.delay);
        self.radio.open_writing_pipe(ROBOT_RADIO_ADDRESSES[self.team][data.robot_id as usize], &mut self.spi, &mut self.delay);
        self.radio.set_payload_size(ControlMessage::len() as u8, &mut self.spi, &mut self.delay);
        for _ in 0..self.retries {
            if self.radio.write(&buffer, &mut self.spi, &mut self.delay) {
                println!("Packet Received");
                break;
            }
            let next_time = self.clock.now() + Duration::from_micros(self.retry_delay);
            while self.clock.now() < next_time {}
        }
        self.radio.start_listening(&mut self.spi, &mut self.delay);
        self.robot_data.insert(data.robot_id, Err(NrfSendError::Timeout));
        self.current_robot_id = data.robot_id;

        // Prime to receive the robot's response
        match data.data.mode {
            Mode::Default => self.radio.set_payload_size(RobotStatusMessage::len() as u8, &mut self.spi, &mut self.delay),
            Mode::ImuTest => self.radio.set_payload_size(ImuTestMessage::len() as u8, &mut self.spi, &mut self.delay),
            Mode::FpgaTest => self.radio.set_payload_size(ControlTestMessage::len() as u8, &mut self.spi, &mut self.delay),
            Mode::KickerTest => self.radio.set_payload_size(KickerTestingMessage::len() as u8, &mut self.spi, &mut self.delay),
            Mode::ProgramKickOnBreakbeam
            | Mode::ProgramKicker => self.radio.set_payload_size(KickerProgramMessage::len() as u8, &mut self.spi, &mut self.delay),
            Mode::ReceiveBenchmark => self.radio.set_payload_size(RadioReceiveBenchmarkMessage::len() as u8, &mut self.spi, &mut self.delay),
            Mode::SendBenchmark => self.radio.set_payload_size(RadioSendBenchmarkMessage::len() as u8, &mut self.spi, &mut self.delay),
        }
        self.data_mode = data.data.mode;
        Ok(())
    }
}

impl<SPI, CSN, CE, GPIOE, SPIE> Subscriber for NrfPublisherSubscriber<SPI, CSN, CE, GPIOE, SPIE> where 
    SPI: SpiDevice<Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    GPIOE: Debug,
    SPIE: Debug,
{
    type Target = Result<IncomingMessage, NrfSendError>;

    fn get(&mut self) -> &Self::Target {
        let wait_time = self.clock.now() + Duration::from_micros(self.receive_block_time);
        while self.clock.now() < wait_time {
            if self.radio.available(&mut self.spi, &mut self.delay) {
                // Create a buffer
                let mut buffer = match self.data_mode {
                    Mode::Default => vec![0u8; RobotStatusMessage::len()],
                    Mode::FpgaTest => vec![0u8; ControlTestMessage::len()],
                    Mode::ImuTest => vec![0u8; ImuTestMessage::len()],
                    Mode::KickerTest => vec![0u8; KickerTestingMessage::len()],
                    Mode::ProgramKickOnBreakbeam
                    | Mode::ProgramKicker => vec![0u8; KickerProgramMessage::len()],
                    Mode::ReceiveBenchmark => vec![0u8; RadioReceiveBenchmarkMessage::len()],
                    Mode::SendBenchmark => vec![0u8; RadioSendBenchmarkMessage::len()],
                };
                
                // Receive the incoming data
                self.radio.read(&mut buffer, &mut self.spi, &mut self.delay);
    
                // Decode the incoming data
                let incoming_message = match self.data_mode {
                    Mode::Default => IncomingMessage::RobotStatus(RobotStatusMessage::unpack(&buffer).unwrap()),
                    Mode::FpgaTest => IncomingMessage::ControlTest(ControlTestMessage::unpack(&buffer).unwrap()),
                    Mode::ImuTest => IncomingMessage::ImuTest(ImuTestMessage::unpack(&buffer).unwrap()),
                    Mode::KickerTest => IncomingMessage::KickerTesting(KickerTestingMessage::unpack(&buffer).unwrap()),
                    Mode::ProgramKickOnBreakbeam
                    | Mode::ProgramKicker => IncomingMessage::KickerProgram(KickerProgramMessage::unpack(&buffer).unwrap()),
                    Mode::ReceiveBenchmark => IncomingMessage::RadioReceive(RadioReceiveBenchmarkMessage::unpack(&buffer).unwrap()),
                    Mode::SendBenchmark => IncomingMessage::RadioSend(RadioSendBenchmarkMessage::unpack(&buffer).unwrap()),
                };
    
                // Put the new data into the hashmap
                println!("Incoming Message: {:?}", incoming_message);
                self.robot_data.insert(self.current_robot_id, Ok(incoming_message));
                break;
            }
        }

        self.robot_data.get(&self.current_robot_id).unwrap()
    }
}

unsafe impl<SPI, CSN, CE, GPIOE, SPIE> Send for NrfPublisherSubscriber<SPI, CSN, CE, GPIOE, SPIE> where 
    SPI: SpiDevice<Error=SPIE>,
    CSN: OutputPin<Error=GPIOE>,
    CE: OutputPin<Error=GPIOE>,
    GPIOE: Debug,
    SPIE: Debug {}