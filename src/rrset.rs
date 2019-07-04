use crate::message_render::MessageRender;
use crate::name::Name;
use crate::rdata::RData;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt::Write;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RRTtl(pub u32);

impl RRTtl {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        buf.read_u32().map(RRTtl)
    }

    pub fn rend(self, render: &mut MessageRender) {
        render.write_u32(self.0);
    }

    pub fn to_wire(self, buf: &mut OutputBuffer) {
        buf.write_u32(self.0);
    }

    fn to_string(self) -> String {
        self.0.to_string()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct RRset {
    pub name: Name,
    pub typ: RRType,
    pub class: RRClass,
    pub ttl: RRTtl,
    pub rdatas: Vec<RData>,
}

impl RRset {
    pub fn from_wire(buf: &mut InputBuffer) -> Result<Self> {
        let name = Name::from_wire(buf)?;
        let typ = RRType::from_wire(buf)?;
        let class = RRClass::from_wire(buf)?;
        let ttl = RRTtl::from_wire(buf)?;
        let rdlen = buf.read_u16()?;
        let mut rdatas = Vec::with_capacity(1);
        if rdlen > 0 {
            let rdata = RData::from_wire(typ, buf, rdlen)?;
            rdatas.push(rdata);
        }
        Ok(RRset {
            name,
            typ,
            class,
            ttl,
            rdatas,
        })
    }

    pub fn rend(&self, render: &mut MessageRender) {
        if self.rdatas.is_empty() {
            self.name.rend(render);
            self.typ.rend(render);
            self.class.rend(render);
            self.ttl.rend(render);
            render.write_u16(0)
        } else {
            self.rdatas.iter().for_each(|rdata| {
                self.name.rend(render);
                self.typ.rend(render);
                self.class.rend(render);
                self.ttl.rend(render);
                let pos = render.len();
                render.skip(2);
                rdata.rend(render);
                let rdlen = render.len() - pos - 2;
                render.write_u16_at(rdlen as u16, pos);
            })
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        if self.rdatas.is_empty() {
            self.name.to_wire(buf);
            self.typ.to_wire(buf);
            self.class.to_wire(buf);
            self.ttl.to_wire(buf);
            buf.write_u16(0)
        } else {
            self.rdatas.iter().for_each(|rdata| {
                self.name.to_wire(buf);
                self.typ.to_wire(buf);
                self.class.to_wire(buf);
                self.ttl.to_wire(buf);
                let pos = buf.len();
                buf.skip(2);
                rdata.to_wire(buf);
                let rdlen = buf.len() - pos - 2;
                buf.write_u16_at(rdlen as u16, pos);
            })
        }
    }

    pub fn to_string(&self) -> String {
        let mut rrset_str = String::new();
        self.rdatas.iter().for_each(|rdata| {
            writeln!(&mut rrset_str, "{}\t{}", self.header(), rdata.to_string()).unwrap();
        });
        rrset_str
    }

    fn header(&self) -> String {
        [
            self.name.to_string(),
            self.ttl.to_string(),
            self.class.to_string(),
            self.typ.to_string(),
        ]
        .join("\t")
    }

    pub fn rr_count(&self) -> usize {
        self.rdatas.len()
    }

    pub fn is_same_rrset(&self, other: &RRset) -> bool {
        self.typ == other.typ && self.name.eq(&other.name)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::rdata_a::A;
    use crate::util::hex::from_hex;

    #[test]
    fn test_rrset_to_wire() {
        let raw =
            from_hex("0474657374076578616d706c6503636f6d000001000100000e100004c0000201").unwrap();
        let mut buf = InputBuffer::new(raw.as_slice());
        let rrset = RRset::from_wire(&mut buf).unwrap();
        let desired_rrset = RRset {
            name: Name::new("test.example.com.").unwrap(),
            typ: RRType::A,
            class: RRClass::IN,
            ttl: RRTtl(3600),
            rdatas: [RData::A(A::from_string("192.0.2.1").unwrap())].to_vec(),
        };
        assert_eq!(rrset, desired_rrset);

        let mut render = MessageRender::new();
        desired_rrset.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
