# VE.Direct library for Rust

[![crates.io badge][crate_badge]][crate]
[![docs.rs badge][docs_badge]][docs]

Library to parse the Victron Energy "VE.Direct" protocol and map the data to useful structs with clear units.

Can be used in conjuction with the `serial` library to pull battery status information from devices like the BMV 700, or solar charging data from the Victron's various MPPT solar charge controllers.

[crate_badge]: https://img.shields.io/crates/v/vedirect
[crate]: https://crates.io/crates/vedirect
[docs]: https://docs.rs/vedirect/
[docs_badge]: https://docs.rs/vedirect/badge.svg

## Details

Developed using a VE.Direct to USB interface cable to a BMV 700. Should work identically with any other connection method to the device (such as the VE.Direct to serial adapters)

Cross compiled to use on a Raspberry Pi Zero W.

Based of the `VE.Direct-Protocol-3.27.pdf`.

Currently only implements the "Text-mode" (read only) interface,

> The VE.Direct interface includes two modes: Text-mode and the HEX-mode. The purpose of the Text-mode is to make retrieving information extremely simple. The product will periodically transmit all run-time fields. The HEX-mode allows not only to read data but also write data, for example, change settings.


## Status

Functional and reasonably well tested. Main limitation is that not all devices are handled, only the BMV battery monitor and solar charge controllers
