use applink_client::mqtt::{Client, Conf, Unsolicited};
use applink_codec::report::{AcceptationStatus, Meta, Report, ReportMsg};
use applink_codec::wizzi_macro::Uid;
use applink_xml::apps::common::WmSys;
use applink_xml::modem::v6_3::*;
use applink_xml::*;
use chrono::prelude::*;
use clap::Parser;
use colored::Colorize;
use std::collections::HashMap;
use std::path::Path;
use wizzicom::strbin::StrBin;
use wizzicom::trace::dprint;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
pub struct Params {
    /// Company ID without the '0x'. Displayed as 'Uid' on your company page. (Default:
    /// $COMPANY_ID)
    #[arg(short, long)]
    pub company: Option<String>,
    /// Applink ID. Displayed as 'User' on your company page. (Default: $APPLINK_ID)
    #[arg(short, long)]
    pub username: Option<String>,
    /// Applink Key. Displayed as 'Key' on your company page. (Default: $APPLINK_KEY)
    #[arg(short, long)]
    pub password: Option<String>,
}

#[derive(Debug)]
pub struct Device {
    uid: Uid,
    last_report: Report,
    dtype: Option<DeviceType>,
    modem_rev: Option<ModemRevision>,
    host_rev: Option<HostRevision>,
}

impl Device {
    pub fn new(r: Report) -> Self {
        Self {
            uid: r.meta.uid.clone(),
            last_report: r,
            dtype: None,
            modem_rev: None,
            host_rev: None,
        }
    }
}

pub fn log(d: &Device, s: String) {
    // Date
    let naive = NaiveDateTime::from_timestamp_opt(d.last_report.meta.timestamp, 0).unwrap();
    let datetime: DateTime<Local> = Local.from_utc_datetime(&naive);
    let ts = datetime.format("%Y-%m-%d %H:%M:%S").to_string();
    let dtype = d.dtype.map_or("Unknown".to_owned(), |v| format!("{:?}", v));

    println!("{} - {} {:<24}: {}", ts, d.uid, dtype, s);
}

fn bad_report(d: &Device, e: serde_json::Error, msg: &serde_json::Value) {
    log(
        d,
        format!("Bad report format: {:#?}\n{:#?}", e, msg)
            .bright_red()
            .to_string(),
    );
}

#[tokio::main]
async fn main() {
    //let mut builder = env_logger::Builder::new();
    //builder.filter_level(log::LevelFilter::Debug);
    //builder.init();

    let mut g_device = HashMap::<Uid, Device>::new();

    let params = Params::parse();

    // Try to get default credentials
    let company =
        params.company.unwrap_or(std::env::var("COMPANY_ID").expect(
            "Environment variable COMPANY_ID not defined or option --company not specified.",
        ));

    let username =
        params
            .username
            .unwrap_or(std::env::var("APPLINK_ID").expect(
                "Environment variable APPLINK_ID not defined or option --username not specified.",
            ));

    let password = params
        .password
        .unwrap_or(std::env::var("APPLINK_KEY").expect(
            "Environment variable APPLINK_KEY not defined or option --password not specified.",
        ));

    println!("Start mqtt client");
    let client_id = format!("{}:4", username);
    let mut options = rumqttc::MqttOptions::new(client_id, "roger.wizzilab.com", 8883);
    options.set_credentials(username, password);
    options.set_transport(rumqttc::Transport::tls_with_default_config());
    let conf = Conf {
        mqtt_options: options,
        subscription_topics: vec![(format!("/applink/{}/#", company), rumqttc::QoS::AtMostOnce)],
    };
    let mut client = Client::new(conf, company, 1).await.unwrap();

    let mut rx = client.unsolicited().await;
    while let Some(msg) = rx.recv().await {
        if let Unsolicited::Report(r) = msg {
            let device = g_device
                .entry(r.meta.uid.clone())
                .or_insert(Device::new(r.clone()));

            device.last_report = r.clone();

            handle_report(device);
        }
    }
}

fn handle_report(device: &mut Device) {
    match device.last_report.meta.a_status {
        AcceptationStatus::Accepted => {
            //println!("{:#?}", r);
            handle_device(device);
        }
        AcceptationStatus::AcceptableRepeat
        | AcceptationStatus::AcceptableReplay
        | AcceptationStatus::AcceptableOutOfSeq => {}
        _ => {
            log(
                device,
                format!("Report rejected: {:?}", device.last_report.meta.a_status)
                    .bright_red()
                    .to_string(),
            );
        }
    }
}

fn handle_device(device: &mut Device) {
    let Report {
        meta: Meta {
            fid, device_type, ..
        },
        ..
    } = device.last_report;

    let fname = device.last_report.meta.fname.clone();

    device.dtype = match DeviceType::try_from(u64::from_be(device_type)) {
        Ok(d) => Some(d),
        _ => {
            log(
                device,
                format!("Unknown device 0x{:016X}", u64::from_be(device_type))
                    .yellow()
                    .to_string(),
            );
            None
        }
    };

    let file = (&fid, fname.as_str());

    let msg = match device.last_report.msg.clone() {
        ReportMsg::Known(m) => m,
        _ => {
            log(
                device,
                format!(
                    "Unknown report {} {} from {:?}",
                    file.0, file.1, device.dtype
                )
                .yellow()
                .to_string(),
            );

            return;
        }
    };

    if ModemRevision::file() == file {
        handle_modem_rev(device, &msg);
    } else if HostRevision::file() == file {
        handle_host_rev(device, &msg);
    } else if WmDebug::file() == file {
        handle_modem_boot(device, &msg);
    } else if WmSys::file() == file {
        handle_host_boot(device, &msg);
    }
}

fn handle_modem_rev(device: &mut Device, msg: &serde_json::Value) {
    match serde_json::from_value::<ModemRevision>(msg.clone()) {
        Ok(rev) => {
            device.modem_rev = Some(rev);
        }
        Err(e) => bad_report(device, e, msg),
    };
}

fn handle_host_rev(device: &mut Device, msg: &serde_json::Value) {
    match serde_json::from_value::<HostRevision>(msg.clone()) {
        Ok(rev) => {
            device.host_rev = Some(rev);
        }
        Err(e) => bad_report(device, e, msg),
    };
}

fn handle_modem_boot(device: &mut Device, msg: &serde_json::Value) {
    match serde_json::from_value::<WmDebug>(msg.clone()) {
        Ok(wm_debug) => {
            log(device, format!("{:?}", wm_debug).yellow().to_string());

            if wm_debug.boot_cause == 'A' {
                let boot_info = BootInfo::Modem(BootInfoModem {
                    msg: wm_debug,
                    rev: device.modem_rev.clone(),
                    typ: match &device.modem_rev {
                        Some(r) => DeviceType::try_from(r.dtype).ok(),
                        None => None,
                    },
                });

                let assert = match decode_assert(&boot_info) {
                    Ok(a) => a.bright_green().to_string(),
                    Err(e) => e.bright_red().to_string(),
                };

                log(device, assert);
            }
        }
        Err(e) => bad_report(device, e, msg),
    }
}

fn handle_host_boot(device: &mut Device, msg: &serde_json::Value) {
    match serde_json::from_value::<WmSys>(msg.clone()) {
        Ok(sys_status) => {
            log(device, format!("{:?}", sys_status).yellow().to_string());

            if sys_status.boot_cause == 'A' {
                let boot_info = BootInfo::Host(BootInfoHost {
                    msg: sys_status,
                    rev: device.host_rev.clone(),
                    typ: device.dtype,
                });

                let assert = match decode_assert(&boot_info) {
                    Ok(a) => a.bright_green().to_string(),
                    Err(e) => e.bright_red().to_string(),
                };

                log(device, assert);
            }
        }
        Err(e) => bad_report(device, e, msg),
    }
}

#[derive(Debug, Clone)]
enum BootInfo {
    Modem(BootInfoModem),
    Host(BootInfoHost),
}

#[derive(Debug, Clone)]
struct BootInfoModem {
    msg: WmDebug,
    rev: Option<ModemRevision>,
    typ: Option<DeviceType>,
}

#[derive(Debug, Clone)]
struct BootInfoHost {
    msg: WmSys,
    rev: Option<HostRevision>,
    typ: Option<DeviceType>,
}

fn decode_assert(boot: &BootInfo) -> Result<String, String> {
    let cloudstation = std::env::var("WIZZIVAULT_ROOT").map_err(|e| format!("{:?}", e))?;

    let (assert, param, dtype) = match boot {
        BootInfo::Modem(info) => (
            info.msg.last_assert & 0xFFFFFF,
            info.msg.last_assert_arg,
            info.typ,
        ),
        BootInfo::Host(info) => (
            info.msg.last_assert & 0xFFFFFF,
            info.msg.last_assert_arg,
            info.typ,
        ),
    };

    let app = dtype
        .ok_or("Unknown device type.".to_owned())?
        .app()
        .ok_or("No app for this type.".to_owned())?;

    let app_folder = format!("{cloudstation}/Releases/Firmware/{app}");

    let ver = match boot {
        BootInfo::Modem(BootInfoModem { rev: Some(rev), .. }) => Some(format!(
            "{}.{}.{}-{:08x}",
            rev.fwmaj, rev.fwmin, rev.fwp, rev.fwh
        )),
        BootInfo::Host(BootInfoHost { rev: Some(rev), .. }) => Some(format!(
            "{}.{}.{}-{:08x}",
            rev.fwmaj, rev.fwmin, rev.fwp, rev.fwh
        )),
        _ => None,
    }
    .ok_or("No revision")?;

    let strbin_name = format!("strbin_{app}_v{ver}.bin");

    let strbin_file = format!("{app_folder}/{app}_v{ver}/{strbin_name}");
    let strbin_file = Path::new(&strbin_file);

    let strbin_file_rc = format!("{app_folder}/rc/{app}_v{ver}/{strbin_name}");
    let strbin_file_rc = Path::new(&strbin_file_rc);

    let file = match (strbin_file.exists(), strbin_file_rc.exists()) {
        (true, _) => strbin_file,
        (_, true) => strbin_file_rc,
        _ => {
            return Err(format!("No strbin file found {:?}", strbin_file));
        }
    };

    let strbin = StrBin::default()
        .load(file)
        .map_err(|e| format!("{:?}", e))?;

    let args = Vec::from([param as i32]);

    dprint(assert, args, &strbin).or_else(|e| Ok(format!("{:?}", e)))
}
