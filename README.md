# Rust Img Size

This is my first connection with rust, an reimplementation of a previous lib I wrote in dart.

- `get_img_size(bytes: &[u8]) -> (u32, u32)` 
    - Returns the image size based on given byte array.
    - (width, height)

- `get_img_type(bytes: &[u8]) -> &str` 
    - Returns the image type based on the given byte array.
    - PNG, BMP, GIF, JPG, WEBP