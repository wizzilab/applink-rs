use num_enum::{IntoPrimitive, TryFromPrimitive};
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum CompanyID {
    Void = 0x00000000,
    WizziLab = 0x01BC50C7,
    OneSitu = 0x0A3EF31F,
    MykhailoBorysov = 0x0752386A,
    IoosBV = 0x0B564D5C,
    Kawantech = 0x13ED6E5F,
    Vestfold = 0x15060001,
    WizziLabDemo = 0x15070000,
    MunichRE = 0x16D62CE1,
    Rezohm = 0x1E65885F,
    EXO = 0x2911FAFC,
    MTNMobileIntelligenceLab = 0x29D022DB,
    ATMAConseil = 0x30E835BD,
    Master = 0x32F83780,
    BuraphaUniversity = 0x3430BA94,
    CAD42 = 0x349D34D7,
    SenseEletronica = 0x381874B2,
    HSR = 0x39EAD71B,
    Nornir = 0x3B774F4A,
    Freelancer = 0x4A916715,
    OBDO = 0x4B8E22ED,
    NolamEmbeddedSystems = 0x50EC4091,
    Master1 = 0x591302E6,
    MSSETelecomParis = 0x5935BD02,
    WiFindIt = 0x5A751604,
    SiiliSolutionsOyj = 0x5E50D2D3,
    ReachTechnology = 0x5F47ED8A,
    Hublex = 0x5F8FADA5,
    HEIA = 0x631C9326,
    DPControl = 0x6BC924CF,
    Qotto = 0x77FAA07A,
    KKRE = 0x7EABEEDE,
}

#[derive(Debug, Copy, Clone)]
pub enum DeviceType {
    Void,
    WizziLab(WizziLabDevice),
    OneSitu(OneSituDevice),
    //MykhailoBorysov(MykhailoBorysovDevice),
    //IoosBV(IoosBVDevice),
    Kawantech(KawantechDevice),
    //Vestfold(VestfoldDevice),
    //WizziLabDemo(WizziLabDemoDevice),
    //MunichRE(MunichREDevice),
    //Rezohm(RezohmDevice),
    //EXO(EXODevice),
    //MTNMobileIntelligenceLab(MTNMobileIntelligenceLabDevice),
    //ATMAConseil(ATMAConseilDevice),
    //Master(MasterDevice),
    //BuraphaUniversity(BuraphaUniversityDevice),
    //CAD42(CAD42Device),
    //SenseEletronica(SenseEletronicaDevice),
    //HSR(HSRDevice),
    //Nornir(NornirDevice),
    //Freelancer(FreelancerDevice),
    //OBDO(OBDODevice),
    //NolamEmbeddedSystems(NolamEmbeddedSystemsDevice),
    //Master1(Master1Device),
    //MSSETelecomParis(MSSETelecomParisDevice),
    WiFindIt(WiFindItDevice),
    //SiiliSolutionsOyj(SiiliSolutionsOyjDevice),
    ReachTechnology(ReachTechnologyDevice),
    //Hublex(HublexDevice),
    //HEIA(HEIADevice),
    //DPControl(DPControlDevice),
    //Qotto(QottoDevice),
    //KKRE(KKREDevice),
}

impl TryFrom<u64> for DeviceType {
    type Error = String;
    fn try_from(from: u64) -> Result<Self, Self::Error> {
        let company_id: u32 = (from >> 32) as u32;
        let device_id: u32 = (from & 0xFFFFFFFF) as u32;

        let company = CompanyID::try_from(company_id).map_err(|e| format!("{:?}", e))?;

        Ok(match company {
            CompanyID::Void => DeviceType::Void,
            CompanyID::WizziLab => DeviceType::WizziLab(
                WizziLabDevice::try_from(device_id).map_err(|e| format!("{:?}", e))?,
            ),
            CompanyID::OneSitu => DeviceType::OneSitu(
                OneSituDevice::try_from(device_id).map_err(|e| format!("{:?}", e))?,
            ),
            CompanyID::Kawantech => DeviceType::Kawantech(
                KawantechDevice::try_from(device_id).map_err(|e| format!("{:?}", e))?,
            ),
            CompanyID::WiFindIt => DeviceType::WiFindIt(
                WiFindItDevice::try_from(device_id).map_err(|e| format!("{:?}", e))?,
            ),
            CompanyID::ReachTechnology => DeviceType::ReachTechnology(
                ReachTechnologyDevice::try_from(device_id).map_err(|e| format!("{:?}", e))?,
            ),
            _ => return Err("Not implemented yet.".to_owned()),
        })
    }
}

impl DeviceType {
    // Returns the device app name
    // for searching for strbin files
    pub fn app(&self) -> Option<String> {
        match self {
            Self::WizziLab(d) => match d {
                WizziLabDevice::D7AMote | WizziLabDevice::D7AFileSystem => Some("wm".to_owned()),
                WizziLabDevice::GatewaySecondaryModem => Some("gw".to_owned()),
                WizziLabDevice::Wult => Some("wult".to_owned()),
                WizziLabDevice::WoltUWBTag => Some("wolt_uwb_tag".to_owned()),
                WizziLabDevice::WoltUWBAnchor => Some("wolt_uwb_anchor".to_owned()),
                WizziLabDevice::WoltMeter => Some("wolt_uwb_tag".to_owned()), // XXX No dedicated app yet
                WizziLabDevice::UguardController => Some("uguard_controller".to_owned()),
                WizziLabDevice::UguardPeripheral => Some("uguard_peripheral".to_owned()),
                WizziLabDevice::UguardTag => Some("uguard_tag".to_owned()),
                WizziLabDevice::UguardSpot => Some("uguard_spot".to_owned()),
                WizziLabDevice::AirConnect => Some("air_connect".to_owned()),
                WizziLabDevice::MotionConnect => Some("motion_connect".to_owned()),
                WizziLabDevice::WBeacon => Some("wbeacon".to_owned()),
                WizziLabDevice::Wisense2 => Some("ws".to_owned()),
                WizziLabDevice::Wisp => Some("wisp".to_owned()),
                WizziLabDevice::WispAir => Some("wispair".to_owned()),
                WizziLabDevice::Wiswitch => Some("wiswitch".to_owned()),
                WizziLabDevice::WispLight => Some("wisp_light".to_owned()),
                WizziLabDevice::AiforsiteAnchor => Some("aiforsite_anchor".to_owned()),
                WizziLabDevice::WP100 => Some("wp100".to_owned()),
                WizziLabDevice::LucyTrot => Some("lucy_trot".to_owned()),
                _ => None,
            },
            Self::WiFindIt(d) => match d {
                WiFindItDevice::WFITag => Some("wfi_tag".to_owned()),
            },
            Self::OneSitu(d) => match d {
                OneSituDevice::OS200 => Some("os200".to_owned()),
                OneSituDevice::OS300 => Some("os300".to_owned()),
                OneSituDevice::OS110 => Some("os110".to_owned()),
            },
            _ => None,
        }
    }
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum WizziLabDevice {
    GatewayModem = 0x00000000,
    Wisense = 0x00000001,
    Wisense2DemoLoRa = 0x00000002,
    RefHost = 0x00000003,
    Blinker = 0x00000004,
    IDI005 = 0x00000005,
    OTABootloader = 0x00000006,
    LoRaAlarm = 0x00000007,
    WMBlank = 0x00000008,
    D7A1xDemoSensors = 0x00000010,
    D7Aalarm = 0x00000011,
    D7Atestemitter = 0x00000012,
    D7A1xDemoSendFileData = 0x00000014,
    D7A1xDemoSendFileDataAndForget = 0x00000015,
    D7A1xDemoReceiveFileData = 0x00000016,
    D7ALoRaWAN = 0x00000017,
    D7A1xDemoBigFile = 0x00000018,
    D7A1xDemoCUP = 0x00000019,
    D7ADemoPicture = 0x0000001A,
    HelloWorld = 0x0000001B,
    ReachTechAnchor = 0x0000001C,
    RFLSimulation = 0x0000001D,
    D7ALocalisation = 0x0000001E,
    PacketsStats = 0x0000001F,
    WultDemo = 0x00000020,
    WgateProModem = 0x00000021,
    WoltUWB = 0x00000030,
    WoltD7 = 0x00000031,
    Wult = 0x00000032,
    USpaceDemo = 0x00000033,
    USpace = 0x00000034,
    USpaceCertif = 0x00000035,
    CrowdScan = 0x00000036,
    CertifFSK = 0x00000037,
    Certif = 0x00000038,
    Dash7Meter = 0x00000039,
    WBeacon = 0x0000003A,
    D7AFileSystem = 0x00001000,
    D7AMote = 0x00001001,
    RTUWBModem = 0x00001002,
    WMTrack = 0x00001003,
    KinderSafeDemo = 0x00005AFE,
    HostVoid = 0x0000701D,
    VestfoldBandDemo = 0x00700000,
    GatewayHost = 0x10000000,
    GatewaySecondaryModem = 0x10000001,
    GatewayHostHybrid = 0x10000002,
    VirtualGateway = 0x10000003,
    InfrastructureMonitorer = 0x10000004,
    GatewayHostVestfold = 0x11000000,
    GatewaySecondaryModemVestfold = 0x11000001,
    StubModem = 0xD7A02212,
    D7ATestReceiver = 0xD7A17E58,
    FRACTIV = 0xF4AC7170,
    Wisense2 = 0xFF000000,
    Wisense2Demo = 0xFF000001,
    Hostref = 0xFF000002,
    DisplayDemo = 0xFF000003,
    LoralarmDemo = 0xFF000004,
    Dash7LoRaDemo = 0xFF000006,
    WisenseMW = 0xFF000007,
    WisenseMDemo = 0xFF000008,
    Wisp = 0xFF000009,
    WispAir = 0xFF00000A,
    WispLink = 0xFF00000B,
    WispGaz = 0xFF00000C,
    Vestfold = 0xFF000010,
    WFITagDemo = 0xFF000011,
    WFIRemote = 0xFF000012,
    WISPIridium = 0xFF000013,
    IridiumTracker = 0xFF000014,
    Wiswitch = 0xFF000015,
    Wistof = 0xFF000016,
    AiforsiteAnchor = 0xFF000017,
    WISPDaherSlave = 0xFF000018,
    WispMate = 0xFF000019,
    WispEngine = 0xFF00001A,
    WispDaherMaster = 0xFF00001B,
    WoltUWBTag = 0xFF00001C,
    WoltUWBAnchor = 0xFF00001D,
    WPCOM = 0xFF00001E,
    UguardController = 0xFF00001F,
    UguardPeripheral = 0xFF000020,
    USpot = 0xFF000021,
    WoltMeter = 0xFF000022,
    WispLight = 0xFF000023,
    WispSH = 0xFF000024,
    UguardTag = 0xFF000026,
    WP100 = 0xFF000027,
    UguardSpot = 0xFF000028,
    WP300 = 0xFF000029,
    MotionConnect = 0xFF00002A,
    AirConnect = 0xFF00002B,
    Wistofdemo = 0xFF00002C,
    BLEBeaconEddystone = 0xFF00002D,
    BLEBeaconiBeacon = 0xFF00002E,
    StubCSMR = 0xFF002212,
    LucyTrot = 0xFF007307,
    NRF91FS = 0xFF025F91,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum OneSituDevice {
    OS200 = 0x00000200,
    OS300 = 0x00000300,
    OS110 = 0x00000400,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum KawantechDevice {
    Device = 0x00000000,
    Kara = 0x00000014,
    DeviceK2 = 0x00000017,
    DeviceK = 0x0000701D,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum WiFindItDevice {
    WFITag = 0x77F10000,
}

#[derive(Debug, Copy, Clone, Deserialize, Serialize, IntoPrimitive, TryFromPrimitive)]
#[repr(u32)]
pub enum ReachTechnologyDevice {
    WizziGatePyHost = 0x00000000,
    UWBAnchor = 0x0000001C,
}

/*

Ioos BV WIZZIGATE   0B564D5C00000000
Ioos BV IOOS Host   0B564D5C00000001
Ioos BV ENGIE-NODE  0B564D5C00000010
-   GateWay Test1   102715A500000000
-   Kara    102715A500000001
Vestfold    LIFE Aurora 1506000100000001
Vestfold    LIFE Iris Touch 1506000100000002
Vestfold    LIFE Transmitter family 1506000100000003
Vestfold    LIFE Junior 1506000100000004
Vestfold    LIFE Bed Vibrator   1506000100000005
Vestfold    LIFE AppSender  1506000100000006
Vestfold    LIFE Speech alert   1506000100000008
Vestfold    LIFE NORA FlexiWatch    1506000100000009
Vestfold    LIFE ARM/Ella   1506000100700000
WizziLab Demo   MYDEVICE    1507000000000000
WizziLab Demo   HelloBlink  1507000000000001
WizziLab Demo   D7A Mote Official   1507000000000002
WizziLab Demo   oss7    1507000000000003
WizziLab Demo   D7A_1x_plugfest 1507000000000004
WizziLab Demo   Device  1507000000000005
WizziLab Demo   DeviceTest  1507000000000006
WizziLab Demo   DeviceTest  1507000000000FFF
MunichRE    Device  16D62CE100000000
MunichRE    Nucleo_Sensor   16D62CE100000001
MunichRE    Wisense-M   16D62CE1FF000007
Rezohm  CAD42   1E65885F00000001
EXO S.A.    Device  2911FAFC00000000
EXO S.A.    Device  2911FAFC00000001
MTN Mobile Intelligence Lab Device 1    29D022DB00000001
ATMA Conseil    Send_big_file   30E835BD00000001
master  Device  32F8378000000000
Burapha University  Device  3430BA9400000000
CAD.42  MasterCAD.42    349D34D700000001
SENSE ELETRONICA LTDA   Device  381874B200000000
HSR GPS Tracker Node    39EAD71B00000000
Nornir  TechNorway_Beacon   3B774F4A00000001
Nornir  TechNorway_Sensor   3B774F4A00000002
Nornir  LB  3B774F4A00000009
Freelancer  Barrier Sensor  4A91671500000010
OBDO    Device1 4B8E22ED00000000
Nolam Embedded Systems  Passive Infrared Sensor 50EC409100009999
master1 Soil    591302E600000000
MSSE - Telecom Paris    Gateway_2   5935BD0200000000
Siili Solutions Oyj TinyTool    5E50D2D300000000
Hublex  Test_Gyropode   5F8FADA500000001
HEIA    test_device 631C932600000000
DPControl   Device  6BC924CF00000000
DPControl   Device_DPControl_demo_sensors   6BC924CF00000001
DPControl   Device_DPControl_receive_file_data  6BC924CF00000003
Qotto   QCOMv3  77FAA07A00000001
kkre    Device  7EABEEDE00000000

*/
