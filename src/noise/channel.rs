use crate::error::{ChannelError, StreamError};
use bytes::Bytes;
use flume::{Receiver, Sender};
use prost::Message;
use std::collections::VecDeque;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct NoiseChannel {
    id: ChannelId,
    sender: Sender<Control>,
    receiver: Receiver<Bytes>,
    stream: Arc<Mutex<Option<super::InnerNoiseStream>>>,
}

#[derive(Eq, PartialEq, Copy, Clone)]
pub struct ChannelId(pub u8);

pub(crate) struct InnerNoiseChannel {
    pub sender: Sender<Bytes>,
    pub receiver: Receiver<Control>,
}

pub enum Control {
    Message(Bytes),
    Failure(ChannelError, FailureResolution),
    Eof,
}

pub enum FailureResolution {
    Ignore,
    CloseChannel,
    CloseConnection,
}

pub(super) struct InternalNoiseChannel {
    sender: Sender<Bytes>,
    receiver: Receiver<Bytes>,
}

impl NoiseChannel {
    pub(super) fn new_pair(id: ChannelId) -> (Self, InnerNoiseChannel) {
        let (p1s, p1r) = flume::unbounded();
        let (p2s, p2r) = flume::unbounded();
        (
            Self {
                sender: p1s,
                receiver: p2r,
                id,
                stream: todo!(),
            },
            InnerNoiseChannel {
                sender: p2s,
                receiver: p1r,
            },
        )
    }

    pub async fn recv<M: Message>(&mut self) -> Result<Option<M>, StreamError> {
        todo!()
    }

    pub async fn send<M: Message>(&mut self, m: &M) -> Result<(), StreamError> {
        todo!()
    }

    pub async fn new_channel(&mut self) -> ! {
        todo!()
    }
}

impl Drop for NoiseChannel {
    fn drop(&mut self) {
        let _ = self.sender.send(Control::Eof);
    }
}
