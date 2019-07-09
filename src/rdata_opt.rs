use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::rdata_field::hex_field_from_iter;
use crate::util::hex::to_hex;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct OPT {
    pub data: Vec<u8>,
}

impl OPT {
    pub fn from_wire(buf: &mut InputBuffer, len: u16) -> Result<Self> {
        buf.read_bytes(len as usize).map(|data| OPT {
            data: data.to_vec(),
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_bytes(self.data.as_slice());
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_bytes(self.data.as_slice());
    }

    pub fn to_string(&self) -> String {
        to_hex(&self.data)
    }

    pub fn from_string<'a>(rdata_str: &mut impl Iterator<Item = &'a str>) -> Result<Self> {
        match hex_field_from_iter("data", rdata_str) {
            Err(e) => Err(DNSError::InvalidRdataString("A", e).into()),
            Ok(data) => Ok(OPT { data }),
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_opt_to_wire() {}
}
