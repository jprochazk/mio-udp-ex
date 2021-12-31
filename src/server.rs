use anyhow::Result;
use mio::net::UdpSocket;
use mio::{Events, Interest, Poll, Token};
use std::{
    io::{self, Cursor, Write},
    net::SocketAddr,
    sync::mpsc::channel,
    time::Duration,
};

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

    let addr = "127.0.0.1:9000";
    let mut socket = UdpSocket::bind(addr.parse().unwrap())?;
    log::info!("Server listening on {}", addr);

    let mut poll = Poll::new()?;

    const SOCKET: Token = Token(0);
    poll.registry()
        .register(&mut socket, SOCKET, Interest::WRITABLE | Interest::READABLE)?;

    let (send, recv) = channel::<(SocketAddr, String)>();

    let mut write_buffer = vec![0u8; 65536];
    let mut read_buffer = vec![0u8; 65536];
    log::info!("Scratch space {}", read_buffer.len());

    let mut events = Events::with_capacity(128);
    loop {
        poll.poll(&mut events, Some(Duration::from_millis(100)))?;
        for event in events.iter() {
            match event.token() {
                // ready to send
                SOCKET if event.is_writable() => {
                    if let Ok((addr, msg)) = recv.try_recv() {
                        let mut writer = Cursor::new(&mut write_buffer[..]);
                        writer.write_all(msg.as_bytes())?;
                        let len = writer.position() as usize;
                        drop(writer);

                        socket.send_to(&write_buffer[0..len], addr)?;
                        log::info!("Echoed '{}'", msg);
                    }
                }
                SOCKET if event.is_readable() => loop {
                    match socket.recv_from(read_buffer.as_mut_slice()) {
                        Ok((len, addr)) => {
                            let msg = std::str::from_utf8(&read_buffer[0..len])?.to_string();
                            log::info!("Received '{}'", msg);
                            send.send((addr, msg))?;
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
