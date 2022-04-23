use std::{collections::HashMap, marker::PhantomData};

use crate::{data, VEError};

#[derive(Debug)]
pub struct VEField {
    pub label: String,
    pub value: Vec<u8>,
}

pub struct Parser<'a, D: data::VEDirectData, E: Events<D>> {
    first_parse: bool,
    parse_buf: Vec<u8>,
    fields: HashMap<String, Vec<u8>>,
    listener: &'a mut E,
    phanton: PhantomData<(&'a E, D)>,
}

pub trait Events<D: data::VEDirectData> {
    fn on_complete_block(&mut self, _block: D) {}
    fn on_missing_field(&mut self, _label: String) {}
    fn on_mapping_error(&mut self, _error: VEError) {}
    fn on_parse_error(&mut self, _error: VEError, _parse_buf: &Vec<u8>) {}
}

const CR: u8 = 13;
const LF: u8 = 10;
const TAB: u8 = 9;
const COLON: u8 = 58;
const A: u8 = 65;

impl<'a, E: Events<D>, D: data::VEDirectData> Parser<'a, D, E> {
    pub fn new(listener: &'a mut E) -> Self {
        Parser {
            first_parse: true,
            parse_buf: Vec::new(),
            fields: HashMap::new(),
            listener,
            phanton: PhantomData,
        }
    }

    fn parse_field(data: &[u8], read_pos: usize) -> Result<(VEField, usize), VEError> {
        if read_pos + 1 >= data.len() {
            return Err(VEError::NeedMoreData);
        }

        let mut cp = read_pos;

        if data[cp] == CR && data[cp + 1] == LF {
            cp = cp + 2;
            match data[cp..].iter().position(|&c| c == TAB) {
                Some(pos) => {
                    let label = String::from_utf8((&data[cp..(cp + pos)]).to_vec())
                        .map_err(|e| VEError::Parse(
                            format!("label string was invalid UTF-8: {}", e)))?;

                    cp = cp + pos + 1; // +1 to skip TAB
                    let endpos_res = data[cp..].iter().position(|&c| c == CR);
                    match endpos_res {
                        Some(endpos) => {
                            let value = &data[cp..(cp + endpos)];
                            Ok((
                                VEField {
                                    label,
                                    value: value.to_vec(),
                                },
                                cp + endpos,
                            ))
                        }
                        None => {
                            if label == "Checksum" {
                                let value = &data[cp..];
                                Ok((
                                    VEField {
                                        label,
                                        value: value.to_vec(),
                                    },
                                    data.len(),
                                ))
                            } else {
                                Err(VEError::NeedMoreData)
                            }
                        }
                    }
                }
                None => Err(VEError::NeedMoreData),
            }
        } else {
            Err(VEError::Parse("Illegal field start".to_string()))
        }
    }

    pub fn feed(&mut self, data: &[u8]) -> Result<(), VEError> {
        if self.first_parse {
            // skip to first field start as we might have started somewhere in the middle
            match data.iter().position(|&c| c == CR) {
                Some(pos) => self.parse_buf.extend_from_slice(&data[pos..]),
                None => return Err(VEError::NeedMoreData),
            }
            self.first_parse = false;
        } else {
            self.parse_buf.extend(data);
        }

        let mut cp = 0;
        loop {
            // skip hex mode messages, those can periodically occur
            while cp + 1 < self.parse_buf.len()
                && self.parse_buf[cp] == COLON
                && self.parse_buf[cp + 1] == A
            {
                match self.parse_buf[cp..].iter().position(|&c| c == LF) {
                    Some(pos) => {
                        if cp + pos + 1 < self.parse_buf.len() {
                            cp = cp + pos + 1;
                        } else {
                            self.parse_buf.clear();
                            return Ok(());
                        }
                    }
                    None => return Ok(()),
                }
            }

            match Parser::<D, E>::parse_field(&self.parse_buf, cp) {
                Ok((field, read_pos)) => {
                    cp = read_pos;
                    if &field.label == "Checksum" {
                        match D::fill(&self.fields) {
                            Ok(mapped) => {
                                self.listener.on_complete_block(mapped);
                                // block_complete(mapped);
                                self.fields.clear();
                            }
                            Err(VEError::MissingField(label)) => {
                                // we didn't get all needed fields to map
                                // reset and hope for more in the next block
                                self.listener.on_missing_field(label);
                                self.fields.clear(); // reset fields
                                self.parse_buf.drain(0..cp);
                                cp = 0;
                            }
                            Err(e) => self.listener.on_mapping_error(e),
                        }
                    } else {
                        self.fields.insert(field.label, field.value);
                    }
                }
                Err(VEError::NeedMoreData) => {
                    let clear_range = if cp > self.parse_buf.len() {
                        self.parse_buf.len()
                    } else {
                        cp
                    };
                    self.parse_buf.drain(0..clear_range);
                    break;
                }
                Err(e) => {
                    self.listener.on_parse_error(e, &self.parse_buf);
                    self.parse_buf.clear();
                    self.fields.clear(); // reset fields
                    self.first_parse = true;
                    break;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct CollectorBmv700 {
        data: Vec<data::Bmv700>,
    }

    impl Events<data::Bmv700> for CollectorBmv700 {
        fn on_complete_block(&mut self, block: data::Bmv700) {
            self.data.push(block);
            println!("Block complete");
        }

        fn on_parse_error(&mut self, _error: VEError, _parse_buf: &Vec<u8>) {
            println!("parse error");
        }
    }

    #[test]
    fn test_partial_stream() {
        let data = "\r\nH18\t2415\r\nChecksum\t\u{4}\r\nPID\t0xA381\r\nV\t12282\r\nVS\t29\r\nI\t-2288\r\nP\t-28\r\nCE\t-74900\r\nSOC\t916\r\nTTG\t10350\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\t~\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876218\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199744\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{3}\r\nPID\t0xA381\r\n".as_bytes();
        let mut collector = CollectorBmv700 { data: vec![] };

        let mut parser = Parser::new(&mut collector);
        parser.feed(data).unwrap();

        // Should have some data remaining
        assert!(parser.parse_buf.len() > 0);
        assert_eq!(parser.parse_buf.len(), 2);
        // Got one block valid data
        assert_eq!(collector.data.len(), 1);
    }

    #[test]
    fn test_parse_field() {
        let data = "\r\nPID\t0xA053\r\nFW\t159\r\nChecksum\t?".as_bytes();

        let (field, read_pos) =
            Parser::<data::Bmv700, CollectorBmv700>::parse_field(data, 0).unwrap();
        assert_eq!(field.label, "PID");
        assert_eq!(field.value, "0xA053".as_bytes());
        assert_eq!(read_pos, 12);

        let (field, read_pos) =
            Parser::<data::Bmv700, CollectorBmv700>::parse_field(data, 12).unwrap();
        assert_eq!(field.label, "FW");
        assert_eq!(field.value, "159".as_bytes());
        assert_eq!(read_pos, 20);

        assert_eq!(
            Parser::<data::Bmv700, CollectorBmv700>::parse_field(data, 19)
                .err()
                .unwrap()
                .to_string(),
            "error parsing data: Illegal field start".to_string()
        );

        let (field, read_pos) =
            Parser::<data::Bmv700, CollectorBmv700>::parse_field(data, 20).unwrap();
        assert_eq!(field.label, "Checksum");
        assert_eq!(read_pos, 32);

        let data = "\r\nFW\t159".as_bytes();
        assert_eq!(
            Parser::<data::Bmv700, CollectorBmv700>::parse_field(data, 0)
                .err()
                .unwrap()
                .to_string(),
            "Need more data to parse successfully".to_string()
        );
    }

    struct CollectorMPPT {
        data: Vec<data::MPPT>,
    }

    impl Events<data::MPPT> for CollectorMPPT {
        fn on_complete_block(&mut self, block: data::MPPT) {
            self.data.push(block);
        }

        fn on_parse_error(&mut self, _error: VEError, _parse_buf: &Vec<u8>) {
            println!("parse error");
        }
    }

    #[test]
    fn test_mppt_stream() {
        let data = "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY2KR\r\nV\t12540\r\nI\t40\r\nVPV\t18540\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nOR\t0x00000000\r\nERR\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHSDS\t16\r\nChecksum\t?".as_bytes();

        let mut collector = CollectorMPPT { data: vec![] };
        let mut parser = Parser::new(&mut collector);
        parser.feed(data).unwrap();
        assert_eq!(collector.data.len(), 1);
        let fields = &collector.data[0];
        assert_eq!(fields.channel1_voltage, 12.54);
        assert_eq!(fields.battery_current, 0.04);
        assert_eq!(fields.panel_voltage, 18.54);
        assert_eq!(fields.panel_power, 5);
        assert_eq!(fields.load_current, 0.3);
        assert_eq!(fields.load_output_state, true);
        assert_eq!(fields.yield_total, 144);
        assert_eq!(fields.yield_today, 1);
        assert_eq!(fields.yield_yesterday, 4);
        assert_eq!(fields.max_power_today, 6);
        assert_eq!(fields.max_power_yesterday, 14);
        assert_eq!(fields.day_sequence, 16);
        assert_eq!(fields.firmware, 159);
        assert_eq!(
            fields.tracker_mode,
            crate::data::TrackerOperationMode::MPPTrackerActive
        );
    }

    #[test]
    fn test_mppt_stream_partial() {
        let datas = vec![
        "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY",
        "2KR\r\nV\t12540\r\nI\t40\r\nVPV\t18540\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nO",
        "R\t0x00000000\r\nERR\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHS",
        "DS\t16\r\nChecksum\t?",
        "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY2KR\r\nV\t12",
        "540\r\nI\t110\r\nVPV\t17660\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nOR\t0x00000000\r\nERR",
        "\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHSDS\t16\r\nChecksum\t?",
        ];

        let mut collector = CollectorMPPT { data: vec![] };
        let mut parser = Parser::new(&mut collector);
        for data in datas {
            parser.feed(data.as_bytes()).unwrap();
        }
        assert_eq!(collector.data.len(), 2);
        let fields = &collector.data[0];
        assert_eq!(fields.channel1_voltage, 12.54);
        assert_eq!(fields.battery_current, 0.04);
        assert_eq!(fields.panel_voltage, 18.54);
        assert_eq!(fields.panel_power, 5);
        assert_eq!(fields.load_current, 0.3);
        assert_eq!(fields.load_output_state, true);
        assert_eq!(fields.yield_total, 144);
        assert_eq!(fields.yield_today, 1);
        assert_eq!(fields.yield_yesterday, 4);
        assert_eq!(fields.max_power_today, 6);
        assert_eq!(fields.max_power_yesterday, 14);
        assert_eq!(fields.day_sequence, 16);
        assert_eq!(fields.firmware, 159);
        assert_eq!(
            fields.tracker_mode,
            crate::data::TrackerOperationMode::MPPTrackerActive
        );

        let fields = &collector.data[1];
        assert_eq!(fields.channel1_voltage, 12.54);
        assert_eq!(fields.battery_current, 0.11);
        assert_eq!(fields.panel_voltage, 17.66);
        assert_eq!(fields.panel_power, 5);
        assert_eq!(fields.load_current, 0.3);
        assert_eq!(fields.load_output_state, true);
        assert_eq!(fields.yield_total, 144);
        assert_eq!(fields.yield_today, 1);
        assert_eq!(fields.yield_yesterday, 4);
        assert_eq!(fields.max_power_today, 6);
        assert_eq!(fields.max_power_yesterday, 14);
        assert_eq!(fields.day_sequence, 16);
        assert_eq!(fields.firmware, 159);
        assert_eq!(
            fields.tracker_mode,
            crate::data::TrackerOperationMode::MPPTrackerActive
        );
    }

    #[test]
    fn test_incomplete_block_reset() {
        let datas = vec![
        "2540\r\nI\t40\r\nVPV\t18540\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nO",
        "R\t0x00000000\r\nERR\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHS",
        "DS\t16\r\nChecksum\t?",
        "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY2KR\r\nV\t12",
        "540\r\nI\t110\r\nVPV\t17660\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nOR\t0x00000000\r\nERR",
        "\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHSDS\t16\r\nChecksum\t?",
    ];

        let mut collector = CollectorMPPT { data: vec![] };
        let mut parser = Parser::new(&mut collector);
        for data in datas {
            parser.feed(data.as_bytes()).unwrap();
        }
        assert_eq!(collector.data.len(), 1);

        let fields = &collector.data[0];
        assert_eq!(fields.channel1_voltage, 12.54);
        assert_eq!(fields.battery_current, 0.11);
        assert_eq!(fields.panel_voltage, 17.66);
        assert_eq!(fields.panel_power, 5);
        assert_eq!(fields.load_current, 0.3);
        assert_eq!(fields.load_output_state, true);
        assert_eq!(fields.yield_total, 144);
        assert_eq!(fields.yield_today, 1);
        assert_eq!(fields.yield_yesterday, 4);
        assert_eq!(fields.max_power_today, 6);
        assert_eq!(fields.max_power_yesterday, 14);
        assert_eq!(fields.day_sequence, 16);
        assert_eq!(fields.firmware, 159);
        assert_eq!(
            fields.tracker_mode,
            crate::data::TrackerOperationMode::MPPTrackerActive
        );
    }

    #[test]
    fn test_mppt_stream_hex_messages() {
        let datas = vec![
        "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY",
        "2KR\r\nV\t12540\r\nI\t40\r\nVPV\t18540\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nO",
        "R\t0x00000000\r\nERR\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHS",
        "DS\t16\r\nChecksum\t?",
        ":A4F1000010000000000AD000000AD000000E508AE05139D04",
        "FFFFFFFFFFFFFFFFFFFFFFFFFF4A\n",
        ":A5010000002000000040000002405C60400000000002E01000000000E0000000A00BA071300D7\n",
        "\r\nPID\t0xA053\r\nFW\t159\r\nSER#\tHQ2132QY2KR\r\nV\t12",
        "540\r\nI\t110\r\nVPV\t17660\r\nPPV\t5\r\nCS\t3\r\nMPPT\t2\r\nOR\t0x00000000\r\nERR",
        "\t0\r\nLOAD\tON\r\nIL\t300\r\nH19\t144\r\nH20\t1\r\nH21\t6\r\nH22\t4\r\nH23\t14\r\nHSDS\t16\r\nChecksum\t?",
    ];

        let mut collector = CollectorMPPT { data: vec![] };
        let mut parser = Parser::new(&mut collector);
        for data in datas {
            parser.feed(data.as_bytes()).unwrap();
        }
        assert_eq!(collector.data.len(), 2);
        let fields = &collector.data[0];
        assert_eq!(fields.channel1_voltage, 12.54);
        assert_eq!(fields.battery_current, 0.04);
        assert_eq!(fields.panel_voltage, 18.54);
        assert_eq!(fields.panel_power, 5);
        assert_eq!(fields.load_current, 0.3);
        assert_eq!(fields.load_output_state, true);
        assert_eq!(fields.yield_total, 144);
        assert_eq!(fields.yield_today, 1);
        assert_eq!(fields.yield_yesterday, 4);
        assert_eq!(fields.max_power_today, 6);
        assert_eq!(fields.max_power_yesterday, 14);
        assert_eq!(fields.day_sequence, 16);
        assert_eq!(fields.firmware, 159);
        assert_eq!(
            fields.tracker_mode,
            crate::data::TrackerOperationMode::MPPTrackerActive
        );

        let fields = &collector.data[1];
        assert_eq!(fields.channel1_voltage, 12.54);
        assert_eq!(fields.battery_current, 0.11);
        assert_eq!(fields.panel_voltage, 17.66);
        assert_eq!(fields.panel_power, 5);
        assert_eq!(fields.load_current, 0.3);
        assert_eq!(fields.load_output_state, true);
        assert_eq!(fields.yield_total, 144);
        assert_eq!(fields.yield_today, 1);
        assert_eq!(fields.yield_yesterday, 4);
        assert_eq!(fields.max_power_today, 6);
        assert_eq!(fields.max_power_yesterday, 14);
        assert_eq!(fields.day_sequence, 16);
        assert_eq!(fields.firmware, 159);
        assert_eq!(
            fields.tracker_mode,
            crate::data::TrackerOperationMode::MPPTrackerActive
        );
    }
}
