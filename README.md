# VE.Direct library for Rust

Library to parse the Victron Energy "VE.Direct" protocol and map the data to useful structs with clear units.

Can be used in conjuction with the `serial` library to pull battery status information from devices like the BMV 700, or solar charging data from the Victron's various MPPT solar charge controllers.

## Details

Developed using a VE.Direct to USB interface cable to a BMV 700, cross compiled for the connected Raspberry Pi Zero W.

Based of the `VE.Direct-Protocol-3.27.pdf`.

Currently only implements the "Text-mode" (read only) interface,

> The VE.Direct interface includes two modes: Text-mode and the HEX-mode. The purpose of the Text-mode is to make retrieving information extremely simple. The product will periodically transmit all run-time fields. The HEX-mode allows not only to read data but also write data, for example, change settings.
