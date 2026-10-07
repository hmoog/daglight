#![forbid(unsafe_code)]
//! The protocol's parameters: the block rate, the finality horizon and the handshake, set by the
//! genesis and inherited by every block from its selected parent.

mod protocol_parameters;

pub use protocol_parameters::ProtocolParameters;
