use failure::Fail;

#[derive(Debug, Fail)]
pub enum DNSError {
    #[fail(display = "wire data is incomplete")]
    InCompleteWire,

    #[fail(display = "name is too long")]
    TooLongName,

    #[fail(display = "label is too long")]
    TooLongLabel,

    #[fail(display = "decimal format isn't valid")]
    InvalidDecimalFormat,

    #[fail(display = "none terminate label")]
    NoneTerminateLabel,

    #[fail(display = "period is duplicate")]
    DuplicatePeriod,

    #[fail(display = "unknown rr type {}", _0)]
    UnknownRRType(u16),

    #[fail(display = "invalid label character")]
    InvalidLabelCharacter,

    #[fail(display = "compress format isn't valid")]
    BadCompressPointer,

    #[fail(display = "name isn't complete")]
    InCompleteName,

    #[fail(display = "length of rdata isn't correct")]
    RdataLenIsNotCorrect,

    #[fail(display = "invalid ipv4 address")]
    InvalidIPv4Address,

    #[fail(display = "no question is provided")]
    ShortOfQuestion,

    #[fail(display = "label index is invalid")]
    InvalidLabelIndex,
}
