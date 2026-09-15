#[cfg(test)]
mod tests {
    use crate::*;

    const VARINT_TESTS: &[(i32, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (2, &[0x02]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (255, &[0xff, 0x01]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (-1, &[0xff, 0xff, 0xff, 0xff, 0x0f]),
        (-2147483648, &[0x80, 0x80, 0x80, 0x80, 0x08]),
    ];

    const VARLONG_TESTS: &[(i64, &[u8])] = &[
        (0, &[0x00]),
        (1, &[0x01]),
        (2, &[0x02]),
        (127, &[0x7f]),
        (128, &[0x80, 0x01]),
        (255, &[0xff, 0x01]),
        (2147483647, &[0xff, 0xff, 0xff, 0xff, 0x07]),
        (
            9223372036854775807,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x7f],
        ),
        (
            -1,
            &[0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0x01],
        ),
        (
            -2147483648,
            &[0x80, 0x80, 0x80, 0x80, 0xF8, 0xFF, 0xFF, 0xFF, 0xFF, 0x01],
        ),
        (
            -9223372036854775808,
            &[0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x80, 0x01],
        ),
    ];

    #[test]
    fn test_var_i32_write() {
        for &(val, expected_bytes) in VARINT_TESTS {
            let mut buf = Buf::new();
            buf.write_var_i32(val);
            assert_eq!(
                &buf.buffer[..buf.get_writer_index() as usize],
                expected_bytes
            );
        }
    }

    #[test]
    fn test_var_i32_read() {
        for &(expected_val, bytes) in VARINT_TESTS {
            let mut buf = Buf::from_vec(bytes.to_vec());
            buf.set_writer_index(bytes.len() as u32);
            assert_eq!(buf.read_var_i32(), expected_val);
        }
    }

    #[test]
    fn test_var_i64_write() {
        for &(val, expected_bytes) in VARLONG_TESTS {
            let mut buf = Buf::new();
            buf.write_var_i64(val);
            assert_eq!(
                &buf.buffer[..buf.get_writer_index() as usize],
                expected_bytes
            );
        }
    }

    #[test]
    fn test_var_i64_read() {
        for &(expected_val, bytes) in VARLONG_TESTS {
            let mut buf = Buf::from_vec(bytes.to_vec());
            buf.set_writer_index(bytes.len() as u32);
            assert_eq!(buf.read_var_i64(), expected_val);
        }
    }

    #[test]
    #[should_panic]
    fn test_varint_error() {
        // VarInt too big (> 5 bytes with MSB set)
        let mut buf = Buf::from_vec(vec![0x80, 0x80, 0x80, 0x80, 0x80, 0x01]);
        buf.set_writer_index(6);
        buf.read_var_i32();
    }

    #[test]
    #[should_panic]
    fn test_varlong_error() {
        // Unexpected EOF (MSB indicates more bytes follow)
        let mut buf = Buf::from_vec(vec![0x80]);
        buf.set_writer_index(1);
        buf.read_var_i32();
    }
}
