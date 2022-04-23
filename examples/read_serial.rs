// serialport = "4.1"

use serialport::SerialPort;
use vedirect::{Events, VEError};

struct Listener;

impl Events<vedirect::Bmv700> for Listener {
    fn on_complete_block(&mut self, block: vedirect::Bmv700) {
        println!("Mapped data {:#?}", &block);
    }

    fn on_parse_error(&mut self, _error: VEError, _parse_buf: &Vec<u8>) {}
}

pub fn record(mut port: Box<dyn SerialPort>) -> anyhow::Result<()> {
    let mut buf: Vec<u8> = vec![0; 1024];
    let mut listener = Listener{};
    let mut parser = vedirect::Parser::new(& mut listener);
    loop {
        let r = port.read(buf.as_mut_slice())?;
        parser.feed(&buf[..r]).unwrap();
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn main() -> anyhow::Result<()> {
    let port = serialport::new("/dev/ttyUSB1", 19_200)
        .data_bits(serialport::DataBits::Eight)
        .timeout(core::time::Duration::from_secs(2))
        .open()
        .expect("Failed to open vedirect serial port");
    record(port)?;
    Ok(())
}
