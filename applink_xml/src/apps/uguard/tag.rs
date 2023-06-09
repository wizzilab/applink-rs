use crate::*;
use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::Deserialize;

#[derive(Debug, Clone)]
pub struct LogEntryTag {
    /// VID
    pub vid: u16,
    /// Distance in dcm
    pub distance: u8,
}

#[derive(Debug, Clone)]
pub struct LogEntryDistance {
    /// Timestamp
    pub ts: u32,
    /// VID
    pub controller_vid: u16,
    /// Distance in dcm
    pub distance: u8,
    /// Tags
    pub tag: [LogEntryTag; 4],
}

#[derive(Debug, Clone)]
pub struct LogEntryOff {
    /// Timestamp
    pub ts: u32,
}

#[derive(Debug, Clone)]
pub struct LogEntryBoot {
    /// Timestamp
    pub ts: u32,
    /// Last Reset Cause
    pub reset_cause: char,
    /// Number of asserts
    pub assert_count: u16,
    /// ID of last assert
    pub last_assert: u32,
    /// 1st argument of last assert
    pub last_assert_arg: u32,
}

#[derive(Debug, Clone)]
pub struct LogEntryBattery {
    /// Timestamp
    pub ts: u32,
    /// Battery voltage in mV
    pub vbat: u16,
}

#[derive(Debug, Clone)]
pub struct LogEntryState {
    /// Timestamp
    pub ts: u32,
    pub state: TagState,
    pub active: bool,
    pub cup: bool,
    pub facedown: bool,
    pub stable: bool,
    pub configured: bool,
    pub running: bool,
    pub plugged: bool,
    pub battery_critical: bool,
}

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive)]
#[repr(u8)]
pub enum TagState {
    Ok,
    Error,
    Stopped,
}

#[derive(Debug, Copy, Clone, IntoPrimitive, TryFromPrimitive, Deserialize)]
#[repr(u8)]
pub enum ActionEnum {
    Boot,
    DriverOn,
    DriverOff,
    WarningOn,
    WarningOff,
    AlarmOn,
    AlarmOff,
    BatteryPlugged,
    BatteryUnplugged,
    BatteryCritical,
    StateStart,
    StateStop,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "serde_json::Value")]
pub enum TagLogAction {
    Boot(LogEntryBoot),
    DriverOn(LogEntryDistance),
    DriverOff(LogEntryOff),
    WarningOn(LogEntryDistance),
    WarningOff(LogEntryOff),
    AlarmOn(LogEntryDistance),
    AlarmOff(LogEntryOff),
    BatteryPlugged(LogEntryBattery),
    BatteryUnplugged(LogEntryBattery),
    BatteryCritical(LogEntryBattery),
    StateStart(LogEntryState),
    StateStop(LogEntryState),
}

type ActionData = [u8; 20];

//{"uguard_tag_log_remaining"=>0, "uguard_tag_log_0_ts"=>1686146781, "uguard_tag_log_0_action"=>6, "uguard_tag_log_0_data"=>"FFFF00000000000000000000000000", "uguard_tag_log_1_ts"=>1686146781, "uguard_tag_log_1_action"=>4, "uguard_tag_log_1_data"=>"FFFF00000000000000000000000000", "uguard_tag_log_2_ts"=>1686146794, "uguard_tag_log_2_action"=>8, "uguard_tag_log_2_data"=>"E30F00000000000000000000000000", "uguard_tag_log_3_ts"=>1686146794, "uguard_tag_log_3_action"=>10, "uguard_tag_log_3_data"=>"000100000101010000000000000000", "uguard_tag_log_4_ts"=>1686146795, "uguard_tag_log_4_action"=>5, "uguard_tag_log_4_data"=>"1A00071C000A000000000000000000", "uguard_tag_log_5_ts"=>1686146795, "uguard_tag_log_5_action"=>3, "uguard_tag_log_5_data"=>"1A00071C000A000000000000000000", "uguard_tag_log_6_ts"=>1686147160, "uguard_tag_log_6_action"=>7, "uguard_tag_log_6_data"=>"E80F00000000000000000000000000", "uguard_tag_log_7_ts"=>1686147160, "uguard_tag_log_7_action"=>11, "uguard_tag_log_7_data"=>"020100000101000100000000000000"}
#[derive(Debug, Clone, Deserialize)]
pub struct TagLog {
    #[serde(rename = "uguard_tag_log_remaining")]
    pub remaining: u16,
    #[serde(rename = "uguard_tag_log_0")]
    pub entry_0: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_1")]
    pub entry_1: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_2")]
    pub entry_2: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_3")]
    pub entry_3: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_4")]
    pub entry_4: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_5")]
    pub entry_5: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_6")]
    pub entry_6: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_7")]
    pub entry_7: Option<TagLogAction>,
    #[serde(rename = "uguard_tag_log_8")]
    pub entry_8: Option<TagLogAction>,
}

impl_xml!(TagLog, 201, "uguard_tag_log");

impl TryFrom<serde_json::Value> for TagLogAction {
    type Error = XMLError;
    fn try_from(from: serde_json::Value) -> Result<Self, Self::Error> {
        Ok(match from {
            serde_json::Value::Object(o) => {
                let mut raw: ActionData = [0; 20];
                let s = o
                    .get("hex")
                    .ok_or(XMLError::ParseError((file!().to_owned(), line!())))?;
                let s = s
                    .as_str()
                    .ok_or(XMLError::ParseError((file!().to_owned(), line!())))?;

                hex::decode_to_slice(s, &mut raw)
                    .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

                let action = ActionEnum::try_from(raw[4])
                    .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

                match action {
                    ActionEnum::Boot => TagLogAction::Boot(raw.try_into()?),
                    ActionEnum::DriverOn => TagLogAction::DriverOn(raw.try_into()?),
                    ActionEnum::DriverOff => TagLogAction::DriverOff(raw.try_into()?),
                    ActionEnum::WarningOn => TagLogAction::WarningOn(raw.try_into()?),
                    ActionEnum::WarningOff => TagLogAction::WarningOff(raw.try_into()?),
                    ActionEnum::AlarmOn => TagLogAction::AlarmOn(raw.try_into()?),
                    ActionEnum::AlarmOff => TagLogAction::AlarmOff(raw.try_into()?),
                    ActionEnum::BatteryPlugged => TagLogAction::BatteryPlugged(raw.try_into()?),
                    ActionEnum::BatteryUnplugged => TagLogAction::BatteryUnplugged(raw.try_into()?),
                    ActionEnum::BatteryCritical => TagLogAction::BatteryCritical(raw.try_into()?),
                    ActionEnum::StateStart => TagLogAction::StateStart(raw.try_into()?),
                    ActionEnum::StateStop => TagLogAction::StateStop(raw.try_into()?),
                }
            }
            _ => return Err(XMLError::ParseError((file!().to_owned(), line!()))),
        })
    }
}

impl TryFrom<ActionData> for LogEntryDistance {
    type Error = XMLError;
    fn try_from(from: ActionData) -> Result<Self, Self::Error> {
        let action = ActionEnum::try_from(from[4])
            .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

        match action {
            ActionEnum::AlarmOn | ActionEnum::DriverOn | ActionEnum::WarningOn => {
                let ts = u32::from_le_bytes(
                    from[0..=3]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                let controller_vid: u16 = u16::from_le_bytes(
                    from[5..=6]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                let distance: u8 = u8::from_le(from[7]);

                let tag =
                    [
                        LogEntryTag {
                            vid: u16::from_le_bytes(from[8..=9].try_into().map_err(|_| {
                                XMLError::ParseError((file!().to_owned(), line!()))
                            })?),
                            distance: u8::from_le(from[10]),
                        },
                        LogEntryTag {
                            vid: u16::from_le_bytes(from[11..=12].try_into().map_err(|_| {
                                XMLError::ParseError((file!().to_owned(), line!()))
                            })?),
                            distance: u8::from_le(from[13]),
                        },
                        LogEntryTag {
                            vid: u16::from_le_bytes(from[14..=15].try_into().map_err(|_| {
                                XMLError::ParseError((file!().to_owned(), line!()))
                            })?),
                            distance: u8::from_le(from[16]),
                        },
                        LogEntryTag {
                            vid: u16::from_le_bytes(from[17..=18].try_into().map_err(|_| {
                                XMLError::ParseError((file!().to_owned(), line!()))
                            })?),
                            distance: u8::from_le(from[19]),
                        },
                    ];

                Ok(LogEntryDistance {
                    ts,
                    controller_vid,
                    distance,
                    tag,
                })
            }
            _ => Err(XMLError::ParseError((file!().to_owned(), line!()))),
        }
    }
}

impl TryFrom<ActionData> for LogEntryOff {
    type Error = XMLError;
    fn try_from(from: ActionData) -> Result<Self, Self::Error> {
        let action = ActionEnum::try_from(from[4])
            .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

        match action {
            ActionEnum::AlarmOff | ActionEnum::DriverOff | ActionEnum::WarningOff => {
                let ts = u32::from_le_bytes(
                    from[0..=3]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );
                Ok(LogEntryOff { ts })
            }
            _ => Err(XMLError::ParseError((file!().to_owned(), line!()))),
        }
    }
}

impl TryFrom<ActionData> for LogEntryBoot {
    type Error = XMLError;
    fn try_from(from: ActionData) -> Result<Self, Self::Error> {
        let action = ActionEnum::try_from(from[4])
            .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

        match action {
            ActionEnum::Boot => {
                let ts = u32::from_le_bytes(
                    from[0..=3]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                let reset_cause = char::from_u32(from[5] as u32)
                    .ok_or(XMLError::ParseError((file!().to_owned(), line!())))?;
                let assert_count = u16::from_le_bytes(
                    from[6..=7]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );
                let last_assert = u32::from_le_bytes(
                    from[8..=11]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );
                let last_assert_arg = u32::from_le_bytes(
                    from[12..=15]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                Ok(LogEntryBoot {
                    ts,
                    reset_cause,
                    assert_count,
                    last_assert,
                    last_assert_arg,
                })
            }
            _ => Err(XMLError::ParseError((file!().to_owned(), line!()))),
        }
    }
}

impl TryFrom<ActionData> for LogEntryBattery {
    type Error = XMLError;
    fn try_from(from: ActionData) -> Result<Self, Self::Error> {
        let action = ActionEnum::try_from(from[4])
            .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

        match action {
            ActionEnum::BatteryPlugged
            | ActionEnum::BatteryUnplugged
            | ActionEnum::BatteryCritical => {
                let ts = u32::from_le_bytes(
                    from[0..=3]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                let vbat: u16 = u16::from_le_bytes(
                    from[5..=6]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                Ok(LogEntryBattery { ts, vbat })
            }
            _ => Err(XMLError::ParseError((file!().to_owned(), line!()))),
        }
    }
}

impl TryFrom<ActionData> for LogEntryState {
    type Error = XMLError;
    fn try_from(from: ActionData) -> Result<Self, Self::Error> {
        let action = ActionEnum::try_from(from[4])
            .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;

        match action {
            ActionEnum::StateStop | ActionEnum::StateStart => {
                let ts = u32::from_le_bytes(
                    from[0..=3]
                        .try_into()
                        .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?,
                );

                let state: TagState = from[5]
                    .try_into()
                    .map_err(|_| XMLError::ParseError((file!().to_owned(), line!())))?;
                let active: bool = from[6] != 0u8;
                let cup: bool = from[7] != 0u8;
                let facedown: bool = from[8] != 0u8;
                let stable: bool = from[9] != 0u8;
                let configured: bool = from[10] != 0u8;
                let running: bool = from[11] != 0u8;
                let plugged: bool = from[12] != 0u8;
                let battery_critical: bool = from[13] != 0u8;

                Ok(LogEntryState {
                    ts,
                    state,
                    active,
                    cup,
                    facedown,
                    stable,
                    configured,
                    running,
                    plugged,
                    battery_critical,
                })
            }
            _ => Err(XMLError::ParseError((file!().to_owned(), line!()))),
        }
    }
}
