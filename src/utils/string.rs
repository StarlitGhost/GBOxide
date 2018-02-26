use std::str;

pub fn str_from_u8_null_utf8(utf8_src: &[u8]) -> Result<&str, str::Utf8Error> {
    let null_range_end = utf8_src.iter()
        .position(|&c| c == b'\0')
        .unwrap_or(utf8_src.len());
    str::from_utf8(&utf8_src[0..null_range_end])
}
