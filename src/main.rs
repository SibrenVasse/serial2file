use clap::Parser;
use std::path::Path;
use std::time::Duration;

use serialport::SerialPort;

use tokio::fs::{File, OpenOptions};
use tokio::io::{self, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::broadcast;
use tokio::sync::broadcast::{Receiver, Sender};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Serial port to open
    #[clap(long, short, value_name = "PATH", default_value = "/dev/ttyUSB0")]
    serial: String,

    /// Serial port baud rate
    #[clap(short, long, value_parser = clap::value_parser!(u32).range(1..), default_value_t = 115_200)]
    baud_rate: u32,

    /// File to save p1 messages to
    #[clap(long, short, value_name = "PATH", required = true)]
    output: String,

    /// TCP port
    #[clap(short, long, value_parser = clap::value_parser!(u16).range(1024..), default_value_t = 1080)]
    port: u16,
}

#[tokio::main]
async fn main() -> io::Result<()> {
    let args: Args = Args::parse();
    let serial_path = Path::new(&args.serial);
    let output_path = Path::new(&args.output);

    if !serial_path.exists() {
        eprintln!("Serial port \"{:?}\" does not exist.", serial_path);
        ::std::process::exit(1);
    }

    // Setup serial port
    let serial_port = serialport::new(&args.serial, args.baud_rate)
        .timeout(Duration::from_millis(10))
        .open()
        .expect(format!("Failed to open \"{}", args.serial).as_str());

    // Setup output file
    let output_file: File;
    if !output_path.exists() {
        output_file = File::create(&args.output).await?;
    } else {
        output_file = OpenOptions::new().append(true).open(&args.output).await?;
    }

    // Setup communication channel
    let (channel_tx, channel_rx) = broadcast::channel::<Vec<u8>>(32);

    tokio::spawn(async move {
        listen_network(args.port, channel_rx).await.expect("Network error");
    });

    io_thread(serial_port, output_file, channel_tx).await?;

    Ok(())
}

async fn listen_network(port: u16, channel_rx: Receiver<Vec<u8>>) -> io::Result<()> {
    let tcp_listener = TcpListener::bind(format!("{}:{}", "127.0.0.1", port))
        .await
        .expect(format!("Unable to listen on 127.0.0.1:{}", port).as_str());

    loop {
        let (mut socket, _) = tcp_listener.accept().await?;
        let mut rx = channel_rx.resubscribe();

        tokio::spawn(async move {
            loop {
                let message = rx.recv().await.unwrap();
                match socket.write_all(&message).await {
                    Ok(_) => {}
                    Err(_) => { return; }
                }
            }
        });
    }
}

async fn io_thread(
    mut serial_port: Box<dyn SerialPort>,
    mut output_file: File,
    channel_tx: Sender<Vec<u8>>,
) -> io::Result<()> {
    let mut serial_buf: Vec<u8> = vec![0; 1024];
    loop {
        match serial_port.read(serial_buf.as_mut_slice()) {
            Ok(t) => {
                channel_tx.send(serial_buf[..t].to_vec()).expect("Unable to send message over IPC");
                output_file.write_all(&serial_buf[..t]).await?;
            }
            Err(ref e) if e.kind() == io::ErrorKind::TimedOut => (),
            Err(e) => {
                panic!("{:?}", e);
            }
        }
    }
}
