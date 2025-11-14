use clap::Parser;

#[derive(Debug, Parser)]
struct Cli {
    port: String,

    #[clap(long)]
    baud: u32,
    #[clap(long)]
    packet_size: usize,
    #[clap(long)]
    packets: usize,
    #[clap(long)]
    flush_each_packet: bool,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let mut port = serialport::new(cli.port, cli.baud).open()?;

    for packet_number in 0..cli.packets {
        let number = format!("{packet_number:020}");
        println!("{number}");

        let number_bytes = number.into_bytes();
        let payload_len = cli.packet_size - number_bytes.len();
        let data: Vec<u8> = number_bytes
            .into_iter()
            .chain(std::iter::repeat_n(b'a', payload_len))
            .collect();

        assert_eq!(data.len(), cli.packet_size);

        let mut timeouts = 0usize;

        loop {
            match port.write_all(&data[..]) {
                Ok(()) => break,
                Err(e) if matches!(e.kind(), std::io::ErrorKind::TimedOut) => {
                    timeouts += 1;
                    continue
                }
                e => e?,
            }
        }

        if timeouts > 0 {
            println!("timeouts: {timeouts}");
        }

        if cli.flush_each_packet {
            port.flush()?;
        }
    }

    Ok(())
}
