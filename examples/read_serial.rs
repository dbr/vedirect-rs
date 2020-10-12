// serial = "0.4"

extern crate serial;
use serial::SerialPort;

pub fn record(port: &mut dyn SerialPort) -> anyhow::Result<()> {
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud19200)?;
        settings.set_char_size(serial::Bits8);
        Ok(())
    })?;
    port.set_timeout(std::time::Duration::from_secs(2))?;
    let mut buf: [u8; 1024] = [0; 1024];
    loop {
        let r = port.read(&mut buf)?;
        let (p, _remainder) = vedirect::parse(&buf)?;
        println!("Got data: {:#?}", &p);
        let mapped = vedirect::map_fields_Bmv700(&p)?;
        println!("Mapped data {:#?}", &mapped);
        std::thread::sleep(std::time::Duration::from_millis(100));
    }
}

fn main() -> anyhow::Result<()> {
    let mut port = serial::open("/dev/ttyUSB1").expect("Failed to open serial port");
    record(&mut port)?;
    Ok(())
}
