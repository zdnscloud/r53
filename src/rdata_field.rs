use crate::name::Name;
use crate::util::hex::from_hex;
use std::fmt::Display;
use std::net::{Ipv4Addr, Ipv6Addr};
use std::str::FromStr;

pub fn name_field_from_iter<'a>(
    field_name: &str,
    rdata_str: &mut impl Iterator<Item = &'a str>,
) -> Result<Name, String> {
    match rdata_str.next() {
        None => Err(format!("{} is missing", field_name)),
        Some(field_str) => parse_name(field_name, field_str),
    }
}

pub fn hex_field_from_iter<'a>(
    field_name: &str,
    rdata_str: &mut impl Iterator<Item = &'a str>,
) -> Result<Vec<u8>, String> {
    match rdata_str.next() {
        None => Err(format!("{} is missing", field_name)),
        Some(field_str) => parse_hex(field_name, field_str),
    }
}

pub fn u16_field_from_iter<'a>(
    field_name: &str,
    rdata_str: &mut impl Iterator<Item = &'a str>,
) -> Result<u16, String> {
    match rdata_str.next() {
        None => Err(format!("{} is missing", field_name)),
        Some(field_str) => parse_str::<u16>(field_name, field_str),
    }
}

pub fn u32_field_from_iter<'a>(
    field_name: &str,
    rdata_str: &mut impl Iterator<Item = &'a str>,
) -> Result<u32, String> {
    match rdata_str.next() {
        None => Err(format!("{} is missing", field_name)),
        Some(field_str) => parse_str::<u32>(field_name, field_str),
    }
}

pub fn ipv4_field_from_iter<'a>(
    field_name: &str,
    rdata_str: &mut impl Iterator<Item = &'a str>,
) -> Result<Ipv4Addr, String> {
    match rdata_str.next() {
        None => Err(format!("{} is missing", field_name)),
        Some(field_str) => parse_str::<Ipv4Addr>(field_name, field_str),
    }
}

pub fn ipv6_field_from_iter<'a>(
    field_name: &str,
    rdata_str: &mut impl Iterator<Item = &'a str>,
) -> Result<Ipv6Addr, String> {
    match rdata_str.next() {
        None => Err(format!("{} is missing", field_name)),
        Some(field_str) => parse_str::<Ipv6Addr>(field_name, field_str),
    }
}

fn parse_str<T>(field_name: &str, field_str: &str) -> Result<T, String>
where
    T: FromStr,
    <T as std::str::FromStr>::Err: Display,
{
    match field_str.parse::<T>() {
        Err(e) => Err(format!("{} is not valid:{}", field_name, e)),
        Ok(v) => Ok(v),
    }
}

fn parse_name(field_name: &str, field_str: &str) -> Result<Name, String> {
    match Name::new(field_str) {
        Err(e) => Err(format!("{} is not valid:{}", field_name, e)),
        Ok(name) => Ok(name),
    }
}

fn parse_hex(field_name: &str, field_str: &str) -> Result<Vec<u8>, String> {
    match from_hex(field_str) {
        None => Err(format!("{} is not valid hex", field_name)),
        Some(bytes) => Ok(bytes),
    }
}
