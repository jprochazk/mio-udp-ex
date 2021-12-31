use anyhow::Result;
use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};
use std::time::Duration;

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

    let mut socket = UdpSocket::bind("127.0.0.1:0".parse().unwrap())?;
    socket.connect("127.0.0.1:9000".parse().unwrap())?;

    let mut poll = Poll::new()?;

    const SOCKET: Token = Token(0);
    poll.registry()
        .register(&mut socket, SOCKET, Interest::READABLE)?;

    let send_one = || -> Result<()> {
        let msg = b"Hello from the other side";
        let len = socket.send(&msg[..])?;
        log::info!(
            "Sent '{}' ({} bytes)",
            std::str::from_utf8(msg).unwrap(),
            len
        );
        Ok(())
    };

    let mut read_buffer = vec![0u8; 65536];
    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        for event in events.iter() {
            match dbg!(event).token() {
                // ready to read data
                SOCKET => {
                    if event.is_readable() {
                        let len = socket.recv(&mut read_buffer[..])?;
                        let msg = std::str::from_utf8(&read_buffer[0..len])?;
                        log::info!("Received '{}'", msg);
                    }
                }
                _ => unreachable!(),
            }
        }

        send_one()?;
    }
}
