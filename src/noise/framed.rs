use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::io::{Error, ErrorKind};
use std::mem::{swap, MaybeUninit};
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf};
use tokio::net::TcpStream;

/// noise messages can be max 2¹⁶ bytes
const MAX_FRAME_CONTENT_SIZE: usize = 2usize.pow(16);

/// MAX_FRAME_CONTENT_SIZE + (1..=3) frame bytes
const MAX_FRAME_SIZE: usize = MAX_FRAME_CONTENT_SIZE + 3;

/// Taken from tokio
macro_rules! ready {
    ($e:expr $(,)?) => {
        match $e {
            std::task::Poll::Ready(t) => t,
            std::task::Poll::Pending => return std::task::Poll::Pending,
        }
    };
}

pub struct Frame16TcpStream {
    stream: TcpStream,
    read_buffer: Option<BytesMut>,
    write_buffer: Option<(usize, Bytes)>,
}

impl Frame16TcpStream {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            read_buffer: Some(BytesMut::new()),
            write_buffer: None,
        }
    }
}

impl AsyncRead for Frame16TcpStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        rbuf: &mut ReadBuf<'_>,
    ) -> Poll<std::io::Result<()>> {
        let mut buf = [0u8; MAX_FRAME_SIZE];
        let mut rb = ReadBuf::new(&mut buf);

        let Self {
            stream,
            read_buffer,
            ..
        } = self.get_mut();

        let (explen, len) = {
            let read_buffer = read_buffer.as_mut().unwrap();

            loop {
                let pin = Pin::new(&mut *stream);

                let explen = match extract_len(read_buffer.chunk()) {
                    Ok(0) => {
                        ready!(pin.poll_read(cx, &mut rb))?;
                        read_buffer.put_slice(rb.filled());
                        continue;
                    }
                    Ok(len) => len,
                    Err(err) => return Poll::Ready(Err(err)),
                };

                let len = match prost::decode_length_delimiter(read_buffer.chunk()) {
                    Ok(len) => len,
                    Err(_) => return Poll::Ready(Err(Error::from(ErrorKind::InvalidData))),
                };

                if read_buffer.len() < len + explen {
                    ready!(pin.poll_read(cx, &mut rb))?;
                    read_buffer.put_slice(rb.filled());
                    continue;
                }

                break (explen, len);
            }
        };
        let buf = read_buffer.take().unwrap();

        let mut buf = buf.freeze();
        buf.advance(explen);
        rbuf.put_slice(&buf.chunk()[..len]);
        buf.advance(len);

        *read_buffer = Some(BytesMut::from(buf.chunk()));

        Poll::Ready(Ok(()))
    }
}

impl AsyncWrite for Frame16TcpStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, Error>> {
        let Self {
            stream,
            write_buffer,
            ..
        } = self.get_mut();

        loop {
            let swap = if let Some((_, buf)) = write_buffer {
                loop {
                    let pin = Pin::new(&mut *stream);

                    let read = ready!(pin.poll_write(cx, buf.chunk()))?;
                    buf.advance(read);

                    if buf.is_empty() {
                        break;
                    }
                }
                true
            } else {
                false
            };
            if swap {
                return Poll::Ready(Ok(write_buffer.take().unwrap().0));
            }

            if buf.len() > MAX_FRAME_CONTENT_SIZE {
                return Poll::Ready(Err(Error::from(ErrorKind::InvalidData)));
            }

            let mut bmut = BytesMut::with_capacity(buf.len() + 3);
            prost::encode_length_delimiter(buf.len(), &mut bmut).unwrap();
            bmut.put_slice(buf);
            *write_buffer = Some((buf.len(), bmut.freeze()));
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let pin = Pin::new(&mut self.stream);
        pin.poll_flush(cx)
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), Error>> {
        let pin = Pin::new(&mut self.stream);
        pin.poll_shutdown(cx)
    }
}

fn extract_len(slice: &[u8]) -> std::io::Result<usize> {
    let len = slice.len().min(3);
    for (b, index) in (&slice[..len]).iter().zip(1..) {
        if *b < 0x80 {
            return Ok(index);
        }
    }
    match len >= 3 {
        true => Err(Error::from(ErrorKind::InvalidData)),
        false => Ok(0),
    }
}