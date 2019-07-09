use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata_field::name_field_from_iter;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NS {
    pub name: Name,
}

impl NS {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        Name::from_wire(buf).map(|name| NS { name })
    }

    pub fn from_string<'a>(rdata_str: &mut impl Iterator<Item = &'a str>) -> Result<Self> {
        match name_field_from_iter("name", rdata_str) {
            Err(e) => Err(DNSError::InvalidRdataString("NS", e).into()),
            Ok(name) => Ok(NS { name }),
        }
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_name(&self.name, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.name.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        self.name.to_string()
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::util::hex::from_hex;

    #[test]
    fn test_ns_to_wire() {
        let raw = from_hex("0474657374076578616d706c6503636f6d00").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let ns = NS::from_wire(&mut buf, raw.len() as u16).unwrap();
        assert_eq!(&ns.name, &Name::new("test.example.com").unwrap());

        let mut render = MessageRender::new();
        ns.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
        assert_eq!(ns.to_string(), "test.example.com.");
    }
}
