use err_derive::Error;

use bytes::Bytes;
use prost::{DecodeError, EncodeError};
use snow::Error as NoiseError;
use std::io::Error as IoError;

#[derive(Debug, Error)]
pub enum StreamError {
    #[error(display = "IO error occurred")]
    IoError(#[error(source)] IoError),
    #[error(display = "failed to encrypt/decrypt data")]
    NoiseError(#[error(source)] NoiseError),
    #[error(display = "failed to decode packet")]
    DecodeError(#[error(source)] DecodeError, Bytes),
    #[error(display = "failed to encode packet")]
    EncodeError(#[error(source)] EncodeError),
    #[error(display = "channel specific error occurred")]
    ChannelError(#[error(source)] ChannelError),
    #[error(display = "stream is already closed")]
    AlreadyClosed,
    #[error(display = "invalid packet received")]
    InvalidPacket,
}

#[derive(Debug, Error)]
pub enum ChannelError {
    #[error(display = "failed to decode packet")]
    DecodeError(#[error(source)] DecodeError, Bytes),
    #[error(display = "failed to encode packet")]
    EncodeError(#[error(source)] EncodeError),
    #[error(display = "invalid packet received")]
    InvalidPacket,
}
