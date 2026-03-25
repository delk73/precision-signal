use dpw4::SignalFrameHeader;
use std::mem::{align_of, size_of};

fn main() {
    println!("Size: {}", size_of::<SignalFrameHeader>());
    println!("Align: {}", align_of::<SignalFrameHeader>());

    let h = SignalFrameHeader::new(123, 48000);
    let bytes = h.to_bytes();
    println!("Bytes: {:02x?}", bytes);
}
