use crate::VEError;

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
    use nom::character::streaming::alphanumeric1;
    use nom::character::streaming::anychar;
    use nom::character::streaming::char;
    use nom::combinator::map;
    use nom::combinator::not;
    use nom::multi::many1;
    use nom::sequence::pair;
    use nom::sequence::preceded;
    use nom::sequence::separated_pair;
    use nom::IResult;

    /// Label which is not "Checksum"
    fn field_label(input: &[u8]) -> IResult<&[u8], &str> {
        map(
            preceded(not(tag("Checksum")), alphanumeric1),
            |s: &[u8]| std::str::from_utf8(s).expect("label"),
        )(input)
    }
    fn line(input: &[u8]) -> IResult<&[u8], Field> {
        // Then <field-label> <tab> <field-value>
        let parsed = pair(
            // Field, tab, value
            separated_pair(field_label, char('\t'), take_until("\r\n")),
            // Newline
            tag("\r\n"));
        // Map data
        let f = map(parsed, |(d, _nl)| Field {
            key: (d.0),
            value: std::str::from_utf8(d.1).expect("invalid string"),
        })(input);
        f
    }
    fn checksum(input: &[u8]) -> IResult<&[u8], Checksum> {
        // "Checksum" <tab> <checksum byte>
        let parsed = pair(
            separated_pair(tag("Checksum"), char('\t'), anychar),
            tag("\r\n")
        );
        map(parsed, |(d, _nl)| Checksum { value: d.1 as u8 })(input)
    }
    fn chunk(input: &[u8]) -> IResult<&[u8], (Vec<Field>, Checksum)> {
        pair(many1(line), checksum)(input)
    }
    chunk(data)
}

pub fn parse(data: &[u8]) -> Result<(Vec<Field>, &[u8]), VEError> {
    let (data, remainder) = match rawparse(data) {
        Err(nom::Err::Error(e)) => Err(VEError::Parse(format!(
            "Parse error: {:?} - remaining data: {:?}",
            e.1,
            std::str::from_utf8(e.0),
        ))),
        Err(e) => Err(VEError::Parse(format!("Unknown error while parsing: {}", e))),
        Ok((remainder, data)) => Ok((data, remainder)),
    }?;
    let (fields, checksum) = data;
    println!("Checksum: {:?}\nFields: {:#?}", checksum, fields);
    Ok((fields, &remainder))
}

#[test]
fn test_parse_line() {
    let data = "\r\nfield1\tvalue1\r\nfield2\tvalue2\r\nChecksum\t4".as_bytes();
    println!("{:?}", data);
    let (data, remaining) = parse(data).unwrap();
    assert_eq!(data.len(), 2);
    assert_eq!(data[0].key, "field1");
    assert_eq!(data[0].value, "value1");
    assert_eq!(data[1].key, "field2");
    assert_eq!(data[1].value, "value2");
}

#[test]
fn test_parse_nonsense() {
    let data= "\r\n!!!!\t\tvalue1\r\nChecksum\t42".as_bytes();
    assert!(parse(data).is_err());
}

#[test]
fn test_partial_stream() {
    let data = "\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876215\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199737\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{4}\r\nPID\t0xA381\r\nV\t12288\r\nVS\t31\r\nI\t-1579\r\nP\t-19\r\nCE\t-74897\r\nSOC\t916\r\nTTG\t10313\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\to\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876215\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199738\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{3}\r\nPID\t0xA381\r\nV\t12289\r\nVS\t29\r\nI\t-1590\r\nP\t-20\r\nCE\t-74897\r\nSOC\t916\r\nTTG\t10345\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\tq\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876215\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199739\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{2}\r\nPID\t0xA381\r\nV\t12290\r\nVS\t29\r\nI\t-1588\r\nP\t-20\r\nCE\t-74898\r\nSOC\t916\r\nTTG\t10375\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\tn\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876216\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199740\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\t\r\nPID\t0xA381\r\nV\t12294\r\nVS\t28\r\nI\t-1428\r\nP\t-18\r\nCE\t-74898\r\nSOC\t916\r\nTTG\t10412\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\ts\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876216\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199741\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{8}\r\nPID\t0xA381\r\nV\t12279\r\nVS\t29\r\nI\t-2569\r\nP\t-32\r\nCE\t-74899\r\nSOC\t916\r\nTTG\t10458\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\ta\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876217\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199742\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{6}\r\nPID\t0xA381\r\nV\t12283\r\nVS\t30\r\nI\t-2389\r\nP\t-29\r\nCE\t-74899\r\nSOC\t916\r\nTTG\t10346\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\tl\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876218\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199743\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{4}\r\nPID\t0xA381\r\nV\t12282\r\nVS\t29\r\nI\t-2288\r\nP\t-28\r\nCE\t-74900\r\nSOC\t916\r\nTTG\t10350\r\nAlarm\tOFF\r\nRelay\tOFF\r\nAR\t0\r\nBMV\t712 Smart\r\nFW\t0403\r\nChecksum\t~\r\nH1\t-76138\r\nH2\t-76138\r\nH3\t0\r\nH4\t0\r\nH5\t0\r\nH6\t-1876218\r\nH7\t12171\r\nH8\t20418\r\nH9\t1199744\r\nH10\t0\r\nH11\t0\r\nH12\t0\r\nH15\t20\r\nH16\t21033\r\nH17\t2404\r\nH18\t2415\r\nChecksum\t\u{3}\r\nPID\t0xA381\r\n".as_bytes();
    let (parsed, remainder) = parse(&data).unwrap();
    assert!(false);
}
