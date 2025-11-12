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

        port.write_all(&data[..])?;
        port.flush()?;
    }

    Ok(())
}
