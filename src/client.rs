use anyhow::Result;
use mio::{net::UdpSocket, Events, Interest, Poll, Token};
use std::{io, time::Duration};

fn init_log() -> Result<()> {
  // default RUST_LOG=info
  std::env::set_var(
    "RUST_LOG",
    std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into()),
  );
  Ok(env_logger::try_init()?)
}

fn main() -> Result<()> {
  init_log()?;

  let mut socket = UdpSocket::bind("127.0.0.1:0".parse()?)?;
  socket.connect("127.0.0.1:9000".parse()?)?;

  let mut poll = Poll::new()?;
  let mut events = Events::with_capacity(128);

  const SOCKET: Token = Token(0);
  poll
    .registry()
    .register(&mut socket, SOCKET, Interest::READABLE | Interest::WRITABLE)?;

  let mut read_buffer = vec![0u8; 65536];
  loop {
    poll.poll(&mut events, Some(Duration::from_millis(100)))?;
    for event in events.iter() {
      match event.token() {
        SOCKET if event.is_writable() => {
          let msg = b"Hello from the other side";
          socket.send(msg)?;
          log::info!("Sent '{}'", std::str::from_utf8(msg)?);
        }
        SOCKET if event.is_readable() => loop {
          match socket.recv(&mut read_buffer[..]) {
            Ok(len) => {
              let msg = std::str::from_utf8(&read_buffer[0..len])?;
              log::info!("Received '{}'", msg);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
              break;
            }
            Err(e) => {
              return Err(e.into());
            }
          }
        },
        _ => unreachable!(),
      }
    }
  }
}
