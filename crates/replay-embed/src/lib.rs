#![no_std]
#![forbid(unsafe_code)]

use replay_core::artifact::Header0;

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct EmbedBuffer {
    pub header: Header0,
}
