use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata_field::name_field_from_iter;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PTR {
    pub name: Name,
}

impl PTR {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        Name::from_wire(buf).map(|name| PTR { name })
    }

    pub fn from_string<'a>(rdata_str: &mut impl Iterator<Item = &'a str>) -> Result<Self> {
        match name_field_from_iter("name", rdata_str) {
            Err(e) => Err(DNSError::InvalidRdataString("PTR", e).into()),
            Ok(name) => Ok(PTR { name }),
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
    #[test]
    fn test_ptr_to_wire() {}
}
