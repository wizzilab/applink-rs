use crate::uid::Uid;
use serde::{Deserialize, Serialize};
use wizzi_common::json;

pub mod raw {
    use super::*;

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub struct Meta {
        pub uid: String,
        pub guid: String,
        pub gmuid: String,
        pub lb: u8,
        pub fid: u8,
        pub fname: String,
        pub device_type: String,
        pub site_id: u16,
        pub lqual: u8,
        pub offset: u32,
        pub roaming: bool,
        pub ct: String,
        pub freq: f64,
        pub status: u32,
        pub s_status: u8,
        pub a_status: u8,
        pub timestamp: i64,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub struct KnownReport {
        pub meta: Meta,
        pub msg: serde_json::Value,
    }

    #[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
    pub struct RawReportMsg {
        pub offset: u32,
        pub payload: String,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    pub struct RawReport {
        pub meta: Meta,
        pub rmsg: RawReportMsg,
    }

    #[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
    #[serde(untagged)]
    pub enum Report {
        Known(KnownReport),
        Raw(Box<RawReport>),
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum SecurityStatus {
    BelowExpectations = 1,
    MatchingExpectations = 2,
    AboveExpectations = 3,
    TodoAskBen = 4,
}

impl TryFrom<u8> for SecurityStatus {
    type Error = ();

    fn try_from(n: u8) -> Result<Self, ()> {
        Ok(match n {
            1 => Self::BelowExpectations,
            2 => Self::MatchingExpectations,
            3 => Self::AboveExpectations,
            4 => Self::TodoAskBen,
            _ => return Err(()),
        })
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum AcceptationStatus {
    Accepted = 0,
    AcceptableRepeat = 1,
    AcceptableReplay = 2,
    AcceptableOutOfSeq = 3,
    RejectedSecurityLevel = 4,
    RejectedBadNlss = 5,
    RejectedIllegal = 6,
}

impl TryFrom<u8> for AcceptationStatus {
    type Error = ();

    fn try_from(n: u8) -> Result<Self, ()> {
        Ok(match n {
            0 => Self::Accepted,
            1 => Self::AcceptableRepeat,
            2 => Self::AcceptableReplay,
            3 => Self::AcceptableOutOfSeq,
            4 => Self::RejectedSecurityLevel,
            5 => Self::RejectedBadNlss,
            6 => Self::RejectedIllegal,
            _ => return Err(()),
        })
    }
}

#[repr(u8)]
#[derive(Debug, Copy, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum Lqual {
    L0 = 0,
    L1 = 1,
    L2 = 2,
    L3 = 3,
    L4 = 4,
    L5 = 5,
}

impl TryFrom<u8> for Lqual {
    type Error = ();

    fn try_from(n: u8) -> Result<Self, ()> {
        Ok(match n {
            0 => Self::L0,
            1 => Self::L1,
            2 => Self::L2,
            3 => Self::L3,
            4 => Self::L4,
            5 => Self::L5,
            _ => return Err(()),
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Meta {
    pub uid: Uid,
    pub guid: Uid,
    pub gmuid: Uid,
    pub lb: u8,
    pub fid: u8,
    pub fname: String,
    pub device_type: u64,
    pub site_id: u16,
    pub lqual: Lqual,
    pub offset: u32,
    pub roaming: bool,
    pub ct: String,
    pub freq: f64,
    pub status: u32,
    pub s_status: SecurityStatus,
    pub a_status: AcceptationStatus,
    pub timestamp: i64,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum MetaParseError {
    BadDeviceType(std::num::ParseIntError),
    BadLqual(u8),
    BadSecurityStatus(u8),
    BadAcceptationStatus(u8),
}

impl TryFrom<raw::Meta> for Meta {
    type Error = MetaParseError;
    fn try_from(meta: raw::Meta) -> Result<Self, Self::Error> {
        let raw::Meta {
            uid,
            guid,
            gmuid,
            lb,
            fid,
            fname,
            device_type,
            site_id,
            lqual,
            offset,
            roaming,
            ct,
            freq,
            status,
            s_status,
            a_status,
            timestamp,
        } = meta;
        Ok(Meta {
            uid: uid.into(),
            guid: guid.into(),
            gmuid: gmuid.into(),
            lb,
            fid,
            fname,
            device_type: u64::from_str_radix(&device_type, 16)
                .map_err(MetaParseError::BadDeviceType)?
                .swap_bytes(),
            site_id,
            lqual: lqual
                .try_into()
                .map_err(|_| MetaParseError::BadLqual(lqual))?,
            offset,
            roaming,
            ct,
            freq,
            status,
            s_status: s_status
                .try_into()
                .map_err(|_| MetaParseError::BadSecurityStatus(s_status))?,
            a_status: a_status
                .try_into()
                .map_err(|_| MetaParseError::BadAcceptationStatus(a_status))?,
            timestamp,
        })
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub struct RawReportMsg {
    pub offset: u32,
    pub payload: Box<[u8]>,
}

#[derive(Debug, Clone, Hash, Eq, PartialEq, Deserialize, Serialize)]
pub enum RawReportMsgParseError {
    NonHexPayload(String),
}

impl TryFrom<raw::RawReportMsg> for RawReportMsg {
    type Error = RawReportMsgParseError;
    fn try_from(report: raw::RawReportMsg) -> Result<Self, Self::Error> {
        Ok(RawReportMsg {
            offset: report.offset,
            payload: hex::decode(&report.payload)
                .map_err(|_| RawReportMsgParseError::NonHexPayload(report.payload))?
                .into_boxed_slice(),
        })
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Deserialize, Serialize)]
pub enum ReportMsg {
    Known(serde_json::Value),
    Raw(RawReportMsg),
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct Report {
    pub meta: Meta,
    pub msg: ReportMsg,
}

#[derive(Debug, Clone)]
pub enum ReportParseError {
    Json(json::DecodingError),
    Meta(MetaParseError),
    RawMsg(RawReportMsgParseError),
}

impl TryFrom<raw::Report> for Report {
    type Error = ReportParseError;
    fn try_from(report: raw::Report) -> Result<Self, Self::Error> {
        Ok(match report {
            raw::Report::Known(report) => Self {
                meta: report.meta.try_into().map_err(Self::Error::Meta)?,
                msg: ReportMsg::Known(report.msg),
            },
            raw::Report::Raw(report) => Self {
                meta: report.meta.try_into().map_err(Self::Error::Meta)?,
                msg: ReportMsg::Raw(report.rmsg.try_into().map_err(Self::Error::RawMsg)?),
            },
        })
    }
}

#[derive(Debug, Clone, PartialEq, Deserialize, Serialize)]
pub struct RawReport {
    pub meta: Meta,
    pub rmsg: RawReportMsg,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum RawReportParseError {
    Meta(MetaParseError),
    Rmsg(RawReportMsgParseError),
}

impl TryFrom<raw::RawReport> for RawReport {
    type Error = RawReportParseError;
    fn try_from(report: raw::RawReport) -> Result<Self, Self::Error> {
        Ok(Self {
            meta: report.meta.try_into().map_err(Self::Error::Meta)?,
            rmsg: report.rmsg.try_into().map_err(Self::Error::Rmsg)?,
        })
    }
}

pub fn parse(data: &str) -> Result<Report, ReportParseError> {
    let raw_report: raw::Report = json::from_str(data).map_err(ReportParseError::Json)?;
    raw_report.try_into()
}
