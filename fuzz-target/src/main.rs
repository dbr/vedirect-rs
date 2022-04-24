#[macro_use]
extern crate afl;
extern crate vedirect;

use vedirect::{Events, VEError};

struct Listener;

impl Events<vedirect::Bmv700> for Listener {
    fn on_complete_block(&mut self, block: vedirect::Bmv700) {
        println!("Mapped data {:#?}", &block);
    }

    fn on_parse_error(&mut self, error: VEError, _parse_buf: &Vec<u8>) {
        println!("Parse error {:#?}", &error);
    }
}

fn main() {
    fuzz!(|data: &[u8]| {
        let mut listener = Listener {};
        let mut parser = vedirect::Parser::new(&mut listener);
        let _ = parser.feed(data);
    });
}
