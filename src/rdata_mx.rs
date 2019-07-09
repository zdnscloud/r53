use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata_field::{name_field_from_iter, u16_field_from_iter};
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct MX {
    pub preference: u16,
    pub name: Name,
}

impl MX {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        let preference = buf.read_u16()?;
        let name = Name::from_wire(buf)?;
        Ok(MX { preference, name })
    }

    pub fn from_string<'a>(rdata_str: &mut impl Iterator<Item = &'a str>) -> Result<Self> {
        match u16_field_from_iter("preference", rdata_str) {
            Err(e) => Err(DNSError::InvalidRdataString("MX", e).into()),
            Ok(preference) => match name_field_from_iter("name", rdata_str) {
                Err(e) => Err(DNSError::InvalidRdataString("MX", e).into()),
                Ok(name) => Ok(MX { preference, name }),
            },
        }
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.preference);
        render.write_name(&self.name, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.preference);
        self.name.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        [self.preference.to_string(), self.name.to_string()].join(" ")
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_mx_to_wire() {}
}
