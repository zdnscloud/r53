use crate::edns::Edns;
use crate::error::DNSError;
use crate::header::Header;
use crate::header_flag::HeaderFlag;
use crate::message_render::MessageRender;
use crate::name::Name;
use crate::question::Question;
use crate::rr_class::RRClass;
use crate::rr_type::RRType;
use crate::rrset::RRset;
use crate::util::{InputBuffer, OutputBuffer};
use failure::Result;
use std::fmt::Write;

#[derive(Copy, Clone)]
pub enum SectionType {
    Answer = 0,
    Auth = 1,
    Additional = 2,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Section(pub Option<Vec<RRset>>);

impl Section {
    fn rr_count(&self) -> usize {
        self.0.as_ref().map_or(0, |rrsets| {
            rrsets
                .iter()
                .fold(0, |count, ref rrset| count + rrset.rr_count())
        })
    }

    pub fn from_wire(buf: &mut InputBuffer, rr_count: u16) -> Result<Self> {
        if rr_count == 0 {
            return Ok(Section(None));
        }

        let mut rrsets = Vec::with_capacity(rr_count as usize);
        let mut last_rrset = RRset::from_wire(buf)?;
        for _ in 1..rr_count {
            let mut rrset = RRset::from_wire(buf)?;
            if rrset.is_same_rrset(&last_rrset) {
                last_rrset.rdatas.push(rrset.rdatas.remove(0));
            } else {
                rrsets.push(last_rrset);
                last_rrset = rrset;
            }
        }
        rrsets.push(last_rrset);
        Ok(Section(Some(rrsets)))
    }

    pub fn rend(&self, render: &mut MessageRender) {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets.iter().for_each(|rrset| rrset.rend(render));
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        if let Some(rrsets) = self.0.as_ref() {
            rrsets.iter().for_each(|rrset| rrset.to_wire(buf));
        }
    }

    pub fn to_string(&self) -> String {
        self.0.as_ref().map_or(String::new(), |rrsets| {
            rrsets.iter().fold(String::new(), |mut out, ref rrset| {
                write!(out, "{}", rrset.to_string()).unwrap();
                out
            })
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Message {
    pub header: Header,
    pub question: Question,
    pub sections: [Section; 3],
    pub edns: Option<Edns>,
}

impl Message {
    pub fn with_query(name: Name, qtype: RRType) -> Self {
        let mut header: Header = Default::default();
        header.set_flag(HeaderFlag::RecursionDesired, true);
        Message {
            header,
            question: Question {
                name,
                typ: qtype,
                class: RRClass::IN,
            },
            sections: [Section(None), Section(None), Section(None)],
            edns: None,
        }
    }

    pub fn from_wire(raw: &[u8]) -> Result<Self> {
        let buf = &mut InputBuffer::new(raw);
        let header = Header::from_wire(buf)?;
        if header.qd_count != 1 {
            return Err(DNSError::ShortOfQuestion.into());
        }

        let question = Question::from_wire(buf)?;
        let answer = Section::from_wire(buf, header.an_count)?;
        let auth = Section::from_wire(buf, header.ns_count)?;
        let mut additional = Section::from_wire(buf, header.ar_count)?;

        let mut edns = None;
        if header.ar_count > 0 {
            let rrsets = additional.0.as_mut().unwrap();
            if rrsets[rrsets.len() - 1].typ == RRType::OPT {
                edns = Some(Edns::from_rrset(&rrsets.pop().unwrap()));
            }
        }

        Ok(Message {
            header,
            question,
            sections: [answer, auth, additional],
            edns,
        })
    }

    pub fn recalculate_header(&mut self) {
        self.header.qd_count = 1;
        self.header.an_count = self.sections[0].rr_count() as u16;
        self.header.ns_count = self.sections[1].rr_count() as u16;
        self.header.ar_count = self.sections[2].rr_count() as u16;
        self.header.ar_count += self.edns.as_ref().map_or(0, |edns| edns.rr_count() as u16);
    }

    pub fn rend(&self, render: &mut MessageRender) {
        self.header.rend(render);
        self.question.rend(render);
        self.sections
            .iter()
            .for_each(|section| section.rend(render));
        if let Some(edns) = self.edns.as_ref() {
            edns.rend(render)
        }
    }

    pub fn to_wire(&self, buf: &mut OutputBuffer) {
        self.header.to_wire(buf);
        self.question.to_wire(buf);
        self.sections
            .iter()
            .for_each(|section| section.to_wire(buf));
        if let Some(edns) = self.edns.as_ref() {
            edns.to_wire(buf)
        }
    }

    pub fn to_string(&self) -> String {
        let mut message_str = String::new();
        write!(message_str, "{}", self.header.to_string()).unwrap();
        if let Some(edns) = self.edns.as_ref() {
            write!(message_str, ";; OPT PSEUDOSECTION:\n{}", edns.to_string()).unwrap();
        }

        write!(
            message_str,
            ";; QUESTION SECTION:\n{}\n",
            self.question.to_string()
        )
        .unwrap();

        if self.header.an_count > 0 {
            write!(
                message_str,
                "\n;; ANSWER SECTION:\n{}",
                self.sections[0].to_string()
            )
            .unwrap();
        }

        if self.header.ns_count > 0 {
            write!(
                message_str,
                "\n;; AUTHORITY SECTION:\n{}",
                self.sections[1].to_string()
            )
            .unwrap();
        }

        if self.header.ar_count > 0 {
            write!(
                message_str,
                "\n;; ADDITIONAL SECTION:\n{}",
                self.sections[2].to_string()
            )
            .unwrap();
        }
        message_str
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::header_flag::HeaderFlag;
    use crate::message_builder::MessageBuilder;
    use crate::name::Name;
    use crate::opcode::Opcode;
    use crate::rcode::Rcode;
    use crate::rdata::RData;
    use crate::rdata_a::A;
    use crate::rdata_ns::NS;
    use crate::rr_class::RRClass;
    use crate::rr_type::RRType;
    use crate::rrset::RRTtl;
    use crate::util::hex::from_hex;

    fn build_desired_message() -> Message {
        let mut msg = Message::with_query(Name::new("test.example.com.").unwrap(), RRType::A);
        {
            let mut builder = MessageBuilder::new(&mut msg);
            builder
                .id(1200)
                .opcode(Opcode::Query)
                .rcode(Rcode::NoError)
                .set_flag(HeaderFlag::QueryRespone)
                .set_flag(HeaderFlag::AuthAnswer)
                .set_flag(HeaderFlag::RecursionDesired)
                .add_answer(RRset {
                    name: Name::new("test.example.com.").unwrap(),
                    typ: RRType::A,
                    class: RRClass::IN,
                    ttl: RRTtl(3600),
                    rdatas: [
                        RData::A(A::from_string("192.0.2.2").unwrap()),
                        RData::A(A::from_string("192.0.2.1").unwrap()),
                    ]
                    .to_vec(),
                })
                .add_auth(RRset {
                    name: Name::new("example.com.").unwrap(),
                    typ: RRType::NS,
                    class: RRClass::IN,
                    ttl: RRTtl(3600),
                    rdatas: [RData::NS(Box::new(
                        NS::from_string("ns1.example.com.").unwrap(),
                    ))]
                    .to_vec(),
                })
                .add_additional(RRset {
                    name: Name::new("ns1.example.com.").unwrap(),
                    typ: RRType::A,
                    class: RRClass::IN,
                    ttl: RRTtl(3600),
                    rdatas: [RData::A(A::from_string("2.2.2.2").unwrap())].to_vec(),
                })
                .edns(Edns {
                    versoin: 0,
                    extened_rcode: 0,
                    udp_size: 4096,
                    dnssec_aware: false,
                    options: None,
                })
                .done();
        }
        msg
    }

    #[test]
    fn test_message_to_wire() {
        let raw =
            from_hex("04b0850000010002000100020474657374076578616d706c6503636f6d0000010001c00c0001000100000e10000
                     4c0000202c00c0001000100000e100004c0000201c0110002000100000e100006036e7331c011c04e0001000100000e100004020202020000
                     291000000000000000").unwrap();
        let message = Message::from_wire(raw.as_slice()).unwrap();
        let desired_message = build_desired_message();
        assert_eq!(message, desired_message);

        let mut render = MessageRender::new();
        desired_message.rend(&mut render);
        assert_eq!(raw.as_slice(), render.data());
    }
}
