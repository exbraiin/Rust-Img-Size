macro_rules! match_signature {
    ($bytes:expr, $src:expr) => {{
        $bytes[..$src.len()] == $src
    }};
    ($bytes:expr, $src:expr, $dst:expr) => {{
        let end = $bytes.len() - $dst.len();
        $bytes[..$src.len()] == $src && $bytes[end..] == $dst
    }};
}

pub fn get_img_size(bytes: &[u8]) -> (u32, u32) {
    if png_check(bytes) {
        return png_size(bytes);
    }
    if bmp_check(bytes) {
        return bmp_size(bytes);
    }
    if gif_check(bytes) {
        return gif_size(bytes);
    }
    if jpg_check(bytes) {
        return jpg_size(bytes);
    }
    if webp_check(bytes) {
        return webp_size(bytes);
    }
    (0, 0)
}

pub fn get_img_type(bytes: &[u8]) -> &str {
    if png_check(bytes) {
        return "PNG";
    }
    if bmp_check(bytes) {
        return "BMP";
    }
    if gif_check(bytes) {
        return "GIF";
    }
    if jpg_check(bytes) {
        return "JPG";
    }
    if webp_check(bytes) {
        return "WEBP";
    }
    ""
}

fn png_check(bytes: &[u8]) -> bool {
    const SRC: [u8; 8] = [0x89, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    const DST: [u8; 8] = [0x49, 0x45, 0x4E, 0x44, 0xAE, 0x42, 0x60, 0x82];
    match_signature!(bytes, SRC, DST)
}

fn png_size(bytes: &[u8]) -> (u32, u32) {
    let w = read_u32_be(bytes, 0x10);
    let h = read_u32_be(bytes, 0x14);
    (w, h)
}

fn bmp_check(bytes: &[u8]) -> bool {
    const SRC: [u8; 2] = [0x42, 0x4D];
    match_signature!(bytes, SRC)
}

fn bmp_size(bytes: &[u8]) -> (u32, u32) {
    let w = read_u32_le(bytes, 0x12);
    let h = read_u32_le(bytes, 0x16);
    (w, h)
}

fn gif_check(bytes: &[u8]) -> bool {
    const SRC89A: [u8; 6] = [0x47, 0x49, 0x46, 0x38, 0x37, 0x61];
    const SRC87A: [u8; 6] = [0x47, 0x49, 0x46, 0x38, 0x39, 0x61];
    const DST: [u8; 1] = [0x3B];
    match_signature!(bytes, SRC87A, DST) || match_signature!(bytes, SRC89A, DST)
}

fn gif_size(bytes: &[u8]) -> (u32, u32) {
    let w = read_u16_le(bytes, 6) as u32;
    let h = read_u16_le(bytes, 8) as u32;
    (w, h)
}

fn jpg_check(bytes: &[u8]) -> bool {
    const SRC: [u8; 2] = [0xFF, 0xD8];
    match_signature!(bytes, SRC)
}

fn jpg_size(bytes: &[u8]) -> (u32, u32) {
    let mut ptr = 2;
    let mut orientation = 1;
    loop {
        let b = read_u8_be(bytes, ptr);
        if b != 0xFF {
            break;
        }

        let typ = read_u8_be(bytes, ptr + 1);
        let len = read_u16_be(bytes, ptr + 2) as u32 + 2;

        if typ == 0xE1 {
            let data = &bytes[ptr..(ptr + len as usize)];
            let or = get_jpg_orientation(data);
            orientation = or.or_else(|| Some(orientation)).unwrap();
        }

        if typ == 0xC0 || typ == 0xC2 {
            let w = read_u16_be(bytes, ptr + 7) as u32;
            let h = read_u16_be(bytes, ptr + 5) as u32;
            return if orientation == 6 || orientation == 8 {
                (h, w)
            } else {
                (w, h)
            };
        }
        ptr += len as usize;
    }
    (0, 0)
}

fn webp_check(bytes: &[u8]) -> bool {
    const SRC: [u8; 4] = [0x52, 0x49, 0x46, 0x46];
    const DST: [u8; 4] = [0x57, 0x45, 0x42, 0x50];
    match_signature!(bytes[..12], SRC, DST)
}

fn webp_size(bytes: &[u8]) -> (u32, u32) {
    const VP8X: [u8; 4] = [0x56, 0x50, 0x38, 0x58];
    let is_extended_format = bytes[12..16] == VP8X;
    if is_extended_format {
        let w = (read_u16_le(bytes, 0x18) + 1) as u32;
        let h = (read_u16_le(bytes, 0x1b) + 1) as u32;
        return (w, h);
    }

    const VP8L: [u8; 4] = [0x56, 0x50, 0x38, 0x4C];
    let is_lossless_format = bytes[12..16] == VP8L;
    if is_lossless_format {
        let buffer = &bytes[0x15..0x19].to_vec();
        let value = read_u32_le(buffer, 0);
        let w = (value & 0x3FFF) + 1;
        let h = ((value >> 14) & 0x3FFF) + 1;
        return (w, h);
    }

    let w = read_u16_le(bytes, 0x1a) as u32;
    let h = read_u16_le(bytes, 0x1c) as u32;
    (w, h)
}

fn get_jpg_orientation(bytes: &[u8]) -> Option<u16> {
    if bytes.len() < 14 {
        return None;
    }

    const EXIF: [u8; 6] = [0x45, 0x78, 0x69, 0x66, 0x00, 0x00];
    if bytes[4..10] != EXIF {
        return None;
    }

    let little = bytes[10] == 0x49;
    let read_u16 = if little { read_u16_le } else { read_u16_be };

    let count = read_u16(bytes, 18);
    let mut idx = 20;
    for _ in 0..count {
        let t = read_u16(bytes, idx);
        if t == 0x0112 {
            return Some(read_u16(bytes, idx + 8));
        }
        idx += 0xC;
    }

    None
}

fn read_u8_be(bytes: &[u8], i: usize) -> u8 {
    u8::from_be_bytes([bytes[i]])
}

fn read_u16_be(bytes: &[u8], i: usize) -> u16 {
    u16::from_be_bytes([bytes[i], bytes[i + 1]])
}

fn read_u16_le(bytes: &[u8], i: usize) -> u16 {
    u16::from_le_bytes([bytes[i], bytes[i + 1]])
}

fn read_u32_be(bytes: &[u8], i: usize) -> u32 {
    u32::from_be_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]])
}

fn read_u32_le(bytes: &[u8], i: usize) -> u32 {
    u32::from_le_bytes([bytes[i], bytes[i + 1], bytes[i + 2], bytes[i + 3]])
}
