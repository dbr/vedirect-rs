use crate::ve_error::VeError;

#[derive(Debug)]
pub struct Field<'a> {
    pub key: &'a str,
    pub value: &'a str,
}

#[derive(Debug)]
pub struct Checksum {
    value: u8,
}

/// Parse binary protocol using nom
fn rawparse(data: &[u8]) -> nom::IResult<&[u8], (Vec<Field>, Checksum)> {
    use nom::bytes::streaming::tag;
    use nom::bytes::streaming::take_until;
    use nom::character::streaming::anychar;
    use nom::character::streaming::char;
    use nom::combinator::map;
    use nom::combinator::not;

    use nom::multi::many1;
    use nom::sequence::pair;
    use nom::sequence::preceded;
    use nom::sequence::separated_pair;
    use nom::IResult;

    // Label which is not "Checksum"
    fn field_label(input: &[u8]) -> IResult<&[u8], &str> {
        map(
            preceded(not(tag("Checksum")), take_until("\t")),
            |s: &[u8]| std::str::from_utf8(s).expect("label"),
        )(input)
    }
    fn line(input: &[u8]) -> IResult<&[u8], Field> {
        // Each field starts with newline, then <field-label> <tab> <field-value>
        let parsed = pair(
            // Newline∏
            tag("\r\n"),
            // Field, tab, value
            separated_pair(field_label, char('\t'), take_until("\r\n")),
        );
        // Map data
        let f = map(parsed, |(_nl, d)| Field {
            key: (d.0),
            value: std::str::from_utf8(d.1).expect("invalid string"),
        })(input);
        f
    }
    fn checksum(input: &[u8]) -> IResult<&[u8], Checksum> {
        // "Checksum" <tab> <checksum byte>
        let parsed = pair(
            tag("\r\n"),
            separated_pair(tag("Checksum"), char('\t'), anychar),
        );
        map(parsed, |(_nl, d)| Checksum { value: d.1 as u8 })(input)
    }
    fn chunk(input: &[u8]) -> IResult<&[u8], (Vec<Field>, Checksum)> {
        pair(many1(line), checksum)(input)
    }
    chunk(data)
}

pub fn parse(data: &[u8]) -> Result<(Vec<Field>, &[u8]), VeError> {
    let (data, remainder) = match rawparse(data) {
        Err(nom::Err::Error(e)) => Err(VeError::Parse(format!(
            "Parse error: {:?} - remaining data: {:?}",
            e.1,
            std::str::from_utf8(e.0),
        ))),
        Err(nom::Err::Incomplete(_needed)) => Err(VeError::NeedMoreData),
        Err(e) => Err(VeError::Parse(format!(
            "Unknown error while parsing: {}",
            e
        ))),
        Ok((remainder, data)) => Ok((data, remainder)),
    }?;
    let (fields, _checksum) = data;
    Ok((fields, &remainder))
}

#[cfg(test)]
mod tests_parser {
    use super::*;

    #[test]
    fn test_parse_line() {
        let data = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\t4".as_bytes();
        let (data, _remaining) = parse(data).unwrap();
        assert_eq!(data.len(), 2);
        assert_eq!(data[0].key, "field1");
        assert_eq!(data[0].value, "value1");
        assert_eq!(data[1].key, "field2");
        assert_eq!(data[1].value, "value2");
    }

    #[test]
    fn test_parse_serial() {
        let data = "\r\nSER#\tABC123\r\nChecksum\t4".as_bytes();
        let (data, _remaining) = parse(data).unwrap();
        assert_eq!(data.len(), 1);
        assert_eq!(data[0].key, "SER#");
        assert_eq!(data[0].value, "ABC123");
    }

    #[test]
    #[ignore = "Since Victron decided to throw which chars as field names (ie SER#), we need to revise the non-sense unforntunately"]
    fn test_parse_nonsense() {
        let data = "\r\n!!!!\t\tvalue1\r\nChecksum\t42".as_bytes();
        assert!(parse(data).is_err());
    }

    #[test]
    fn test_partial_stream() {
        let mut data = "\r\nH18\t2415\r\nChecksum\t\u{4}\r\nPID\t0xA381\r\nV\t12282\r\nVS\t29\r\nI\t-2288\r\nP\t-28\r\nCE\t-74900\r\nSOC\t916\r\nTTG\t10350\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\t~\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876218\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199744\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{3}\r\nPID\t0xA381\r\n".as_bytes();
        let mut alldata: Vec<Vec<Field>> = vec![];
        while data.len() > 0 {
            let res = parse(&data);
            match res {
                Ok((parsed, remainder)) => {
                    alldata.push(parsed);
                    data = remainder;
                }
                Err(VeError::NeedMoreData) => {
                    break;
                }
                Err(e) => {
                    panic!(e);
                }
            };
        }

        // Got three blocks of data
        assert_eq!(alldata.len(), 3);
        // Should have some data remaining
        assert!(data.len() > 0);
        assert_eq!(data.len(), 14);
    }
}
