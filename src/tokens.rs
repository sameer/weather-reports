use uom::si::f64::{Angle, Length, Pressure, TemperatureInterval, Velocity};

macro_rules! enum_with_str_repr {
    (
        $(#[$enum_attr:meta])*
        $ident: ident {
            $(
                $(#[$variant_attr:meta])*
                $variant: ident => $val: literal $(| $alt: literal)*,
            )*
    }) => {
        #[derive(Clone, Copy, PartialEq, Eq, Debug)]
        $(#[$enum_attr])*
        pub enum $ident {
            $(
                $(#[$variant_attr])*
                $variant,
            )*
        }

        impl Into<&'static str> for $ident {
            fn into(self) -> &'static str {
                use $ident::*;
                match self {
                    $(
                        $variant => $val,
                    )*
                }
            }
        }

        impl<'input> std::convert::TryFrom<&'input str> for $ident {
            type Error = ();

            fn try_from(val: &'input str) -> Result<Self, Self::Error> {
                use $ident::*;
                match val {
                    $(
                        $val $(| $alt)* => Ok($variant),
                    )*
                    _ => Err(())
                }
            }
        }
    };
}

enum_with_str_repr! {
    ObservationType {
        Auto => "AUTO",
        Correction => "COR",
        CorrectionA => "CCA",
        CorrectionB => "CCB",
        Delayed => "RTD",
    }
}

enum_with_str_repr! {
    CloudCoverage {
        NoCloud => "SKC",
        NilCloud => "NCD",
        Clear => "CLR",
        NoSignificantCloud => "NSC",
        Few => "FEW" | "FW",
        Scattered => "SCT" | "SC",
        Broken => "BKN",
        Overcast => "OVC",
        VerticalVisibility => "VV",
    }
}

enum_with_str_repr! {
    Intensity {
        Light => "-",
        Moderate => "",
        Heavy => "+",
    }
}

enum_with_str_repr! {
    Descriptor {
        Shallow => "MI",
        Partial => "PR",
        Patches => "BC",
        LowDrifting => "DR",
        Blowing => "BL",
        Showers => "SH",
        Thunderstorm => "TS",
        Freezing => "FZ",
    }
}

enum_with_str_repr! {
    Precipitation {
        Rain => "RA",
        Drizzle => "DZ",
        Snow => "SN",
        SnowGrains => "SG",
        IceCrystals => "IC",
        IcePellets => "PL",
        Hail => "GR",
        Graupel => "GS",
        Unknown => "UP",
    }
}

enum_with_str_repr! {
    Obscuration {
        Fog => "FG",
        Mist => "BR",
        Haze => "HZ",
        VolcanicAsh => "VA",
        WidespreadDust => "DU",
        Smoke => "FU",
        Sand => "SA",
        Spray => "PY",
    }
}

enum_with_str_repr! {
    Other {
        Squall => "SQ",
        SandWhirls => "PO",
        Duststorm => "DS",
        Sandstorm => "SS",
        FunnelCloud => "FC",
    }
}

enum_with_str_repr! {
    CloudType {
        Cumulonimbus => "CB",
        ToweringCumulus => "TCU",
        Cumulus => "CU",
        Cirrus => "CI",
        Altocumulus => "AC",
        Stratus => "ST",
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct ZuluDateTime {
    pub day_of_month: u8,
    pub hour: u8,
    pub minute: u8,
    /// Some stations omit the Z. It is unclear whether this means
    /// that the timestamp is not zulu, or if it is incorrect implementation.
    pub is_zulu: bool,
}

impl ZuluDateTime {
    #[cfg(feature = "chrono_helpers")]
    pub fn as_datetime(&self, year: i32, month: u32) -> chrono::DateTime<chrono_tz::Tz> {
        chrono::TimeZone::ymd(&chrono_tz::Greenwich, year, month, self.day_of_month as u32).and_hms(
            self.hour as u32,
            self.minute as u32,
            0,
        )
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wind {
    /// A lack of direction indicates variable
    pub direction: Option<Angle>,
    pub speed: Velocity,
    pub peak_gust: Option<Velocity>,
    pub variance: Option<(Angle, Angle)>,
}

impl Wind {
    pub fn is_calm(&self) -> bool {
        self.speed.value < f64::EPSILON
            && self
                .direction
                .map(|angle| angle.value < f64::EPSILON)
                .unwrap_or(false)
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RunwayVisibility<'input> {
    pub designator: &'input str,
    pub visibility: VisibilityType,
    pub trend: Option<VisibilityTrend>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum VisibilityType {
    Varying {
        lower: RawRunwayVisibility,
        upper: RawRunwayVisibility,
    },
    Fixed(RawRunwayVisibility),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RawRunwayVisibility {
    pub value: Length,
    pub out_of_range: Option<OutOfRange>,
}

enum_with_str_repr! {
    VisibilityTrend {
        Up => "U",
        Down => "D",
        NoChange => "N",
    }
}

enum_with_str_repr! {
    OutOfRange {
        Above => "P",
        Below => "M",
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RunwayReport<'input> {
    pub designator: &'input str,
    pub report_info: RunwayReportInfo,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum RunwayReportInfo {
    Cleared { friction: Option<f64> },
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Weather {
    pub intensity: Intensity,
    pub vicinity: bool,
    pub descriptor: Option<Descriptor>,
    /// Condition should always be present, unless this is a VCTS or VCSH
    pub condition: Option<Condition>,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Condition {
    Precipitation(Vec<Precipitation>),
    Obscuration(Obscuration),
    Other(Other),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CloudCover {
    pub coverage: CloudCoverage,
    /// The absence of a base indicates it is below station level or an inability to assess
    pub base: Option<Length>,
    pub cloud_type: Option<CloudType>,
}

/// If negative, these are rounded up to the more positive whole degree
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Temperatures {
    pub air: TemperatureInterval,
    /// Some stations don't report this
    pub dewpoint: Option<TemperatureInterval>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AccumulatedRainfall {
    /// In the last 10 minutes
    pub recent: Length,
    /// Since 0900 (presumably local time)
    pub past: Length,
}

enum_with_str_repr! {
    /// [Color states](https://en.wikipedia.org/wiki/Colour_state) quickly provide information about visibility and cloud height
    ///
    /// Often used by military fields.
    ColorState {
        BluePlus => "BLU+",
        Blue => "BLU",
        White => "WHT",
        Green => "GRN",
        YellowOne => "YLO1" | "YLO",
        YellowTwo => "YLO2",
        Amber => "AMB",
        Red => "RED",
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Visibility {
    pub prevailing: Length,
    pub minimum_directional: Option<DirectionalVisibility>,
    pub maximum_directional: Option<DirectionalVisibility>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DirectionalVisibility {
    pub direction: CompassDirection,
    pub distance: Length,
}

enum_with_str_repr! {
    /// The eight [compass points](https://en.wikipedia.org/wiki/Points_of_the_compass#8-wind_compass_rose)
    CompassDirection {
        NorthEast => "NE",
        NorthWest => "NW",
        North => "N",
        SouthEast => "SE",
        SouthWest => "SW",
        South => "S",
        East => "E",
        West => "W",
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct SeaConditions {
    /// Seawater temperature
    pub temperature: Option<TemperatureInterval>,
    /// On a unit-less scale of 0 = lowest to 9 = highest
    pub wave_intensity: Option<u8>,
}

#[derive(Clone, PartialEq, Debug)]
pub enum Trend {
    /// No significant change in weather expected for the next 2 hours
    NoSignificantChange,
    Becoming(TrendReport),
    Temporarily(TrendReport),
}

#[derive(Clone, PartialEq, Debug)]
pub struct TrendReport {
    pub time: Option<TrendTime>,
    pub wind: Option<Wind>,
    pub visibility: Option<Visibility>,
    pub weather: Vec<Weather>,
    pub cloud_cover: Vec<CloudCover>,
    pub color_state: Option<ColorState>,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TrendTime {
    pub time_type: TrendTimeType,
    pub time: u16,
}

enum_with_str_repr! {
    TrendTimeType {
        At => "AT",
        From => "FM",
        Until => "TL",
    }
}

#[derive(Clone, PartialEq, Debug)]
pub struct MetarReport<'input> {
    pub identifier: &'input str,
    pub observation_time: ZuluDateTime,
    pub observation_type: Option<ObservationType>,
    pub wind: Option<Wind>,
    pub visibility: Option<Visibility>,
    pub runway_visibility: Vec<RunwayVisibility<'input>>,
    pub runway_reports: Vec<RunwayReport<'input>>,
    pub weather: Vec<Weather>,
    pub cloud_cover: Vec<CloudCover>,
    /// Indicative of OK ceiling and visibility
    ///
    /// While in the international standard, some countries do not use this. Notably, Canada
    pub cavok: bool,
    pub temperatures: Option<Temperatures>,
    pub pressure: Option<Pressure>,
    pub accumulated_rainfall: Option<AccumulatedRainfall>,
    /// `BLACK` in a METAR report indicates the airfield is closed
    pub is_closed: bool,
    pub color_state: Option<ColorState>,
    /// Some stations will report the next color state
    pub next_color_state: Option<ColorState>,
    pub recent_weather: Vec<Weather>,
    pub sea_conditions: Option<SeaConditions>,
    pub trends: Vec<Trend>,
    pub remark: Option<&'input str>,
    /// Some automated METAR reports indicate if the system needs maintenance
    ///
    /// This may indicate unreliable measurements.
    pub maintenance_needed: bool,
}
