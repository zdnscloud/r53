use crate::error::DNSError;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata_field::{name_field_from_iter, u16_field_from_iter};
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct NAPTR {
    pub order: u16,
    pub preference: u16,
    pub flags: u16,
    pub services: u16,
    pub replacement: Name,
}

impl NAPTR {
    pub fn from_wire(buf: &mut InputBuffer, _len: u16) -> Result<Self> {
        let order = buf.read_u16()?;
        let preference = buf.read_u16()?;
        let flags = buf.read_u16()?;
        let services = buf.read_u16()?;
        let replacement = Name::from_wire(buf)?;
        Ok(NAPTR {
            order,
            preference,
            flags,
            services,
            replacement,
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        render.write_u16(self.order);
        render.write_u16(self.preference);
        render.write_u16(self.flags);
        render.write_u16(self.services);
        render.write_name(&self.replacement, true);
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        buf.write_u16(self.order);
        buf.write_u16(self.preference);
        buf.write_u16(self.flags);
        buf.write_u16(self.services);
        self.replacement.to_wire(buf);
    }

    pub fn to_string(&self) -> String {
        [
            self.order.to_string(),
            self.preference.to_string(),
            self.flags.to_string(),
            self.services.to_string(),
            self.replacement.to_string(),
        ]
        .join(" ")
    }

    pub fn from_string<'a>(rdata_str: &mut impl Iterator<Item = &'a str>) -> Result<Self> {
        match u16_field_from_iter("order", rdata_str) {
            Err(e) => Err(DNSError::InvalidRdataString("NAPTR", e).into()),
            Ok(order) => match u16_field_from_iter("preference", rdata_str) {
                Err(e) => Err(DNSError::InvalidRdataString("NAPTR", e).into()),
                Ok(preference) => match u16_field_from_iter("flags", rdata_str) {
                    Err(e) => Err(DNSError::InvalidRdataString("NAPTR", e).into()),
                    Ok(flags) => match u16_field_from_iter("services", rdata_str) {
                        Err(e) => Err(DNSError::InvalidRdataString("NAPTR", e).into()),
                        Ok(services) => match name_field_from_iter("replacement", rdata_str) {
                            Err(e) => Err(DNSError::InvalidRdataString("NAPTR", e).into()),
                            Ok(replacement) => Ok(NAPTR {
                                order,
                                preference,
                                flags,
                                services,
                                replacement,
                            }),
                        },
                    },
                },
            },
        }
    }
}

#[cfg(test)]
mod test {
    #[test]
    fn test_naptr_to_wire() {}
}
