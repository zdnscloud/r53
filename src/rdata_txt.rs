use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata_field::{name_field_from_iter, u16_field_from_iter};
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TXT {
    pub data: Vec<Vec<u8>>,
}

impl TXT {
    pub fn from_wire(buf: &mut InputBuffer, len: u16) -> Result<Self> {
        let mut read_len = 0;
        let mut data = Vec::new();
        while read_len < len {
            let sl = buf.read_u8()?;
            let bytes = buf.read_bytes(sl as usize)?;
            read_len += (sl + 1) as u16;
            data.push(bytes.to_vec());
        }
        Ok(TXT { data })
    }

    pub fn from_string<'a>(rdata_str: &mut impl Iterator<Item = &'a str>) -> Result<Self> {
        Ok(TXT { data: Vec::new() })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        for data in &self.data {
            render.write_u8(data.len() as u8);
            render.write_bytes(data.as_slice());
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        for data in &self.data {
            buf.write_u8(data.len() as u8);
            buf.write_bytes(data.as_slice());
        }
    }

    pub fn to_string(&self) -> String {
        "".to_string()
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_txt_to_wire() {}
}
