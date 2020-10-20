// serial = "0.4"

extern crate serial;
use std::cmp;

use serial::SerialPort;
use vedirect::Map;

pub fn record(port: &mut dyn SerialPort) -> anyhow::Result<()> {
    port.reconfigure(&|settings| {
        settings.set_baud_rate(serial::Baud19200)?;
        settings.set_char_size(serial::Bits8);
        Ok(())
    })?;
    port.set_timeout(std::time::Duration::from_secs(2))?;
    let mut counter = 0u8;
    loop {
        let mut buf: [u8; 1024] = [0; 1024];
        counter += 1;
        println!("counter: {}", counter);

        let _ = port.read(&mut buf)?;
        let data: Vec<u8> = buf.iter().filter(|x| **x > 0).map(|&x| x).collect();
        // let data: Vec<u8> = buf.iter().map(|&x| x).collect();

        let message = unsafe {
            let length = data.len();
            std::str::from_utf8_unchecked(&data[0..cmp::min(150, length)])
        };
        // let message = std::str::from_utf8_lossy(&data);

        println!("Serial data:\n{:?}", message);

        match vedirect::parse(&buf) {
            Ok(result) => {
                let (p, _remainder) = result;
                println!("Got data: {:#?}", &p);
                // let mapped = vedirect::Bmv700::map_fields(&p)?;
                let mapped = vedirect::Mppt::map_fields(&p)?;
                println!("Mapped data {:#?}", &mapped);
            }
            _ => println!("Parsing failed for BMV-700"),
        }

        println!();
        std::thread::sleep(std::time::Duration::from_millis(1000));
    }
}

fn main() -> anyhow::Result<()> {
    let port = "/dev/tty.SLAB_USBtoUART";
    // let port = "/dev/ttyUSB1";

    let mut port = serial::open(port).expect("Failed to open serial port");
    record(&mut port)?;
    Ok(())
}
