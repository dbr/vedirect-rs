use criterion::{black_box, criterion_group, criterion_main, Criterion};
use vedirect::*;

pub fn bench_checksum(c: &mut Criterion) {
    let data: Vec<u8> = (0..u8::MAX).collect();
    c.bench_function("checksum 0..255", |b| {
        b.iter(|| checksum::calculate(black_box(&data)))
    });
}

pub fn bench_parse_and_map(c: &mut Criterion) {
    let frame = "\r\nPID\t0xA042\r\nFW\t150\r\nSER#\tHQ1328A1B2C\r\nV\t0\r\nI\t0\r\nVPV\t0\r\nPPV\t0\r\nCS\t0\r\nMPPT\t0\r\nOR\t0x00000001\r\nERR\t0\r\nH19\t0\r\nH20\t0\r\nH21\t0\r\nH22\t0\r\nH23\t0\r\nHSDS\t0\r\nChecksum\t".as_bytes();
    let frame = &checksum::append(frame, 68);

    c.bench_function("Parse & Map fields", |b| {
        b.iter(|| {
            let (fields, checksum, _remainder) =
                vedirect::parser::parse(black_box(&frame)).unwrap();
            let _device = MpptFrame::map_fields(&fields, checksum).unwrap();
        })
    });
}

criterion_group!(benches, bench_checksum, bench_parse_and_map);
criterion_main!(benches);
