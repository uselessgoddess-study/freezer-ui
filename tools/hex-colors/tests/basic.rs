use hex_colors::color_from_hex as hex;

#[test]
fn basic() {
    assert_eq!(hex!(0xDC143Cff), [0xDC, 0x14, 0x3C, 0xFF]);
    assert_eq!(hex!(0xDC143C00), [0xDC, 0x14, 0x3C, 0x00]);
    assert_eq!(hex!(0xDC143C), [0xDC, 0x14, 0x3C]);

    assert_eq!(hex!(0xDC143Cff), [220, 20, 60, 255]);
    assert_eq!(hex!(0xDC143C00), [220, 20, 60, 000]);
    assert_eq!(hex!(0xDC143C), [220, 20, 60]);
}
