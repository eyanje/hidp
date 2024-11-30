/// Bluetooth HID protocol (HIDP) as defined in section 3 of the Bluetooth HID specification. 
use std::io::{self, Read};
use std::iter;
use std::slice;

pub type MessageType = u8;
pub type Parameter = u8;

pub mod message_type {
    use super::MessageType;

    pub const HANDSHAKE: MessageType = 0x0;
    pub const HID_CONTROL: MessageType = 0x1;

    pub const GET_REPORT: MessageType = 0x4;
    pub const SET_REPORT: MessageType = 0x5;
    pub const GET_PROTOCOL: MessageType = 0x6;
    pub const SET_PROTOCOL: MessageType = 0x7;

    pub const DATA: MessageType = 0xA;
}

pub mod handshake {
    use super::MessageType;

    pub const SUCCESSFUL: MessageType = 0x0;
    pub const NOT_READY: MessageType = 0x1;
    pub const ERR_INVAILD_REPORT_ID: MessageType = 0x2;
    pub const ERR_UNSUPPORTED_REQUEST: MessageType = 0x3;
    pub const ERR_INVALID_PARAMETER: MessageType = 0x4;
    pub const ERR_UNKNOWN: MessageType = 0xE;
    pub const ERR_FATAL: MessageType = 0xF;
}

pub mod protocol {
    use super::MessageType;

    pub const BOOT: MessageType = 0x0;
    pub const REPORT: MessageType = 0x1;
}

pub struct Header(MessageType, Parameter);

impl Header {
    /// Construct a Header from a message type and parameter.
    pub const fn new(message_type: MessageType, parameter: Parameter) -> Self {
        Header(message_type, parameter)
    }

    /// Returns the message type specified in this header.
    pub const fn message_type(&self) -> MessageType {
        self.0
    }

    /// Returns the parameter contained in this header.
    pub const fn parameter(&self) -> Parameter {
        self.1
    }
}

impl From<u8> for Header {
    /// Construct a header by splitting a single byte.
    fn from(v: u8) -> Self {
        Self((v >> 4) & 0xF, v & 0xF)
    }
}

impl From<Header> for u8 {
    /// Convert a Header to its byte representation.
    fn from(v: Header) -> Self {
        v.message_type() << 4 | v.parameter()
    }
}

/// HID protocol messages. Deprecated messages are unsupported.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Message {
    Handshake(Parameter),
    HidControl(Parameter),
    GetReport(Parameter, Box<[u8]>),
    SetReport(Parameter, Box<[u8]>),
    GetProtocol(Parameter),
    SetProtocol(Parameter),
    Data(Parameter, Box<[u8]>),
}

impl Message {
    /// Construct a new Message for a report other than an input, output, or feature report.
    pub fn new_data_other(data: Box<[u8]>) -> Self{
        Self::Data(0, data)
    }
    /// Construct a new Message for an input report.
    pub fn new_data_input(data: Box<[u8]>) -> Self{
        Self::Data(1, data)
    }
    /// Construct a new Message for an output report.
    pub fn new_data_output(data: Box<[u8]>) -> Self{
        Self::Data(2, data)
    }
    /// Construct a new Message for a feature report.
    pub fn new_data_feature(data: Box<[u8]>) -> Self{
        Self::Data(3, data)
    }


    pub fn parameter(&self) -> Parameter {
        match self {
            Self::Handshake(parameter) | Self::HidControl(parameter) | Self::GetProtocol(parameter)
                | Self::SetProtocol(parameter) | Self::GetReport(parameter, _) |
                Self::SetReport(parameter, _) | Self::Data(parameter, _) => *parameter,
        }
    }

    /// Message type code
    pub fn message_type(&self) -> MessageType {
        match self {
            Message::Handshake(..) => message_type::HANDSHAKE,
            Message::HidControl(..) => message_type::HID_CONTROL,
            Message::GetReport(..) => message_type::GET_REPORT,
            Message::SetReport(..) => message_type::SET_REPORT,
            Message::GetProtocol(..) => message_type::GET_PROTOCOL,
            Message::SetProtocol(..) => message_type::SET_PROTOCOL,
            Message::Data(..) => message_type::DATA,
        }
    }

    pub fn header(&self) -> Header {
        Header::new(self.message_type(), self.parameter())
    }

    /// Return this message's data, if it exists.
    pub fn data<'a>(&'a self) -> Option<&'a [u8]> {
        match self {
            Self::GetReport(_, data) | Self::SetReport(_, data) | Self::Data(_, data) =>
                Some(data),
            _ => None,
        }
    }

    pub fn read_from(mut data: &[u8]) -> io::Result<Self> {
        let mut header_byte = 0u8;
        data.read_exact(slice::from_mut(&mut header_byte))?;
        let header = Header::from(header_byte);

        Ok(match header.message_type() {
            message_type::HANDSHAKE => Message::Handshake(header.parameter()),
            message_type::HID_CONTROL => Message::HidControl(header.parameter()),
            message_type::GET_REPORT => Message::GetReport(header.parameter(), Box::from(data)),
            message_type::SET_REPORT => Message::SetReport(header.parameter(), Box::from(data)),
            message_type::GET_PROTOCOL => Message::GetProtocol(header.parameter()),
            message_type::SET_PROTOCOL => Message::SetProtocol(header.parameter()),
            message_type::DATA => Message::Data(header.parameter(), Box::from(data)),
            _ => {
                return Err(io::Error::new(io::ErrorKind::InvalidData, "invalid message type encountered"));
            },
        })
    }

    pub fn as_bytes(&self) -> Box<[u8]> {
        let parameter_iter = iter::once(self.header().into());
        if let Some(data) = self.data() {
            parameter_iter.chain(data.into_iter().copied()).collect()
        } else {
            parameter_iter.collect()
        }
    }
}
