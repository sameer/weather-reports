
use uom::si::f64::{Angle, Length, Pressure, ThermodynamicTemperature, Velocity};

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

        impl From<$ident> for &'static str {
            fn from(slf: $ident) -> Self {
                use $ident::*;
                match slf {
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

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum ObservationFlag {
    Auto,
    Nil,
    Correction { letter: Option<char> },
    Delayed,
}

impl From<ObservationFlag> for String {
    fn from(slf: ObservationFlag) -> Self {
        use ObservationFlag::*;
        match slf {
            Auto => "AUTO".to_string(),
            Nil => "NIL".to_string(),
            Correction { letter: None } => "COR".to_string(),
            Correction {
                letter: Some(letter),
            } => format!("CC{}", letter),
            Delayed => "RTD".to_string(),
        }
    }
}

impl<'input> std::convert::TryFrom<&'input str> for ObservationFlag {
    type Error = ();

    fn try_from(val: &'input str) -> Result<Self, Self::Error> {
        use ObservationFlag::*;
        match val {
            "AUTO" => Ok(Auto),
            "NIL" => Ok(Nil),
            "COR" => Ok(Correction { letter: None }),
            correction_with_letter if correction_with_letter.starts_with("CC") => {
                let mut it = correction_with_letter.chars().skip(2);
                let letter = it.next();
                let remaining = it.count();
                if let (Some((letter, true)), 0) =
                    (letter.zip(letter.map(char::is_alphabetic)), remaining)
                {
                    Ok(Correction {
                        letter: Some(letter),
                    })
                } else {
                    Err(())
                }
            }
            "RTD" => Ok(Delayed),
            _ => Err(()),
        }
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
pub struct DateTime {
    pub day_of_month: u8,
    pub time: MilitaryTime,
    /// Some stations omit the Z. It is unclear whether this means
    /// that the timestamp is not zulu, or if it is incorrect implementation
    pub is_zulu: bool,
}

impl DateTime {
    #[cfg(feature = "chrono_helpers")]
    pub fn as_datetime(&self, year: i32, month: u32) -> chrono::DateTime<chrono_tz::Tz> {
        self.time.as_datetime(chrono::TimeZone::ymd(
            &chrono_tz::Greenwich,
            year,
            month,
            self.day_of_month as u32,
        ))
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct MilitaryTime {
    pub hour: u8,
    pub minute: u8,
}

impl MilitaryTime {
    #[cfg(feature = "chrono_helpers")]
    pub fn as_datetime(
        &self,
        date: chrono::Date<chrono_tz::Tz>,
    ) -> chrono::DateTime<chrono_tz::Tz> {
        date.and_hms(self.hour as u32, self.minute as u32, 0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct TimeRange {
    pub begin: MilitaryTime,
    pub end: MilitaryTime,
}

impl TimeRange {
    #[cfg(feature = "chrono_helpers")]
    pub fn as_start_and_duration(
        &self,
        date: chrono::Date<chrono_tz::Tz>,
    ) -> (chrono::DateTime<chrono_tz::Tz>, chrono::Duration) {
        let begin = self.begin.as_datetime(date);
        let end = self.end.as_datetime(date);
        (begin, (end - begin))
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Wind {
    /// A lack of direction indicates variable
    pub direction: Option<Angle>,
    pub speed: Option<Velocity>,
    pub peak_gust: Option<Velocity>,
    pub variance: Option<(Angle, Angle)>,
}

impl Wind {
    pub fn is_calm(&self) -> Option<bool> {
        if let (Some(Velocity { value: speed, .. }), Some(Angle { value: angle, .. })) =
            (self.speed, self.direction)
        {
            Some(
                speed < f64::EPSILON
                    && angle < f64::EPSILON
                    && self.peak_gust.is_none()
                    && self.variance.is_none(),
            )
        } else {
            None
        }
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
        lower: RawVisibility,
        upper: RawVisibility,
    },
    Fixed(RawVisibility),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct RawVisibility {
    /// If present, visibility is out of the observable range
    pub out_of_range: Option<OutOfRange>,
    pub distance: Length,
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
    /// Runway has been cleared of any deposits
    Cleared {
        friction: Option<f64>,
    },
    ClosedSnowOrIce,
    Condition {
        deposit: DepositType,
        coverage: Option<Coverage>,
        depth: Option<Length>,
        friction_coefficient: Option<f64>,
        braking_action: Option<BrakingAction>,
    },
}

enum_with_str_repr! {
    DepositType {
        ClearAndDry => "0",
        Damp => "1",
        Wet => "2",
        Frost => "3",
        DrySnow => "4",
        WetSnow => "5",
        Slush => "6",
        Ice => "7",
        CompactedSnow => "8",
        FrozenRidges => "9",
    }
}

enum_with_str_repr! {
    Coverage {
        VeryLow => "1",
        Low => "2",
        Medium => "5",
        High => "9",
    }
}

enum_with_str_repr! {
    BrakingAction {
        Poor => "91",
        PoorToMedium => "92",
        Medium => "93",
        MediumToGood => "94",
        Good => "95",
        UnreliableMeasurement => "99",
    }
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Weather {
    pub intensity: Intensity,
    /// If true, the described phenomenon was not observed above the station, but within 8KM of it
    pub vicinity: bool,
    pub descriptor: Option<Descriptor>,
    /// Condition should always be present, unless this is a VCTS or VCSH
    pub condition: Option<Condition>,
}

impl From<Weather> for String {
    fn from(slf: Weather) -> Self {
        let intensity = <&str>::from(slf.intensity);
        let vicinity = match slf.vicinity {
            true => "VC",
            false => ""
        };
        let descriptor = match slf.descriptor {
            Some(d) => <&str>::from(d),
            None => ""
        };
        let condition = match slf.condition {
            Some(c) => c.into(),
            None => "".to_string()
        };
        format!("{}{}{}{}", intensity, vicinity, descriptor, condition)
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

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Condition {
    /// There can be multiple forms of precipitation observed
    Precipitation(Vec<Precipitation>),
    Obscuration(Obscuration),
    Other(Other),
}

impl From<Condition> for String {
    fn from(slf: Condition) -> Self {
        use Condition::*;
        match slf {
            Precipitation(p) => {
                p.iter()
                    .map(|p| -> &str { From::from(*p) })
                    .flat_map(|s|{s.chars()})
                    .collect::<String>()
            }
            Obscuration(o) => <&str>::from(o).to_string(),
            Other(o) => <&str>::from(o).to_string(),
        }
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

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct CloudCover {
    pub coverage: CloudCoverage,
    /// The absence of a base indicates it is below station level or an inability of an automated system to make an assessment
    pub base: Option<Length>,
    pub cloud_type: Option<CloudType>,
}

/// If negative, these are rounded up to the more positive whole degree
#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Temperatures {
    pub air: ThermodynamicTemperature,
    /// Some stations don't report this, hence it is marked as optional
    pub dewpoint: Option<ThermodynamicTemperature>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct AccumulatedRainfall {
    /// In the 10 minutes prior to the report time
    pub recent: Length,
    /// Since 0900 local
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
pub struct Color {
    /// `BLACK` in a METAR indicates the airfield is closed
    pub is_black: bool,
    pub current_color: ColorState,
    pub next_color: Option<ColorState>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Visibility {
    pub prevailing: Option<RawVisibility>,
    /// Typically reported when visibility in a particular direction differs significantly from prevailing visibility
    /// If there is more than one direction, the most operationally significant direction is used
    ///
    /// Sometimes, the direction may not be reported at all.
    pub minimum: Option<DirectionalOrRawVisiblity>,
    /// Reported in addition to the minimum when there is large difference in directional visibility (i.e. 1500M vs 5000M)
    pub maximum_directional: Option<DirectionalVisibility>,
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum DirectionalOrRawVisiblity {
    Raw(RawVisibility),
    Directional(DirectionalVisibility),
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct DirectionalVisibility {
    pub direction: CompassDirection,
    pub distance: RawVisibility,
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
pub struct WaterConditions {
    /// Water temperature at the surface
    pub temperature: Option<ThermodynamicTemperature>,
    pub surface_state: Option<WaterSurfaceState>,
    pub significant_wave_height: Option<Length>,
}

enum_with_str_repr! {
    /// See Table 3700 on page A-326 in the [WMO Manual on Codes](https://library.wmo.int/doc_num.php?explnum_id=10235)
    WaterSurfaceState {
        GlassyCalm => "0",
        RippledCalm => "1",
        Smooth => "2",
        Slight => "3",
        Moderate => "4",
        Rough => "5",
        VeryRough => "6",
        High => "7",
        VeryHigh => "8",
        Phenomenal => "9",
    }
}

#[derive(Clone, PartialEq, Debug)]
pub enum Trend {
    /// No significant change in weather expected for the next 2 hours
    NoSignificantChange,
    /// Expected changes
    Becoming(TrendReport),
    /// Temporary fluctuations that last less than an hour
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
pub enum TrendTime {
    At(MilitaryTime),
    Range {
        from: MilitaryTime,
        until: MilitaryTime,
    },
    From(MilitaryTime),
    Until(MilitaryTime),
}

#[derive(Clone, PartialEq, Debug)]
pub struct MetarReport<'input> {
    /// Station [ICAO identifier](https://en.wikipedia.org/wiki/ICAO_airport_code)
    pub identifier: &'input str,
    pub observation_time: Option<DateTime>,
    /// Usually used by TAFs, but some stations include this
    pub observation_validity_range: Option<TimeRange>,
    pub observation_flags: Vec<ObservationFlag>,
    pub wind: Option<Wind>,
    pub visibility: Option<Visibility>,
    /// Included by some airport stations
    pub runway_visibilities: Vec<RunwayVisibility<'input>>,
    /// Included by some airport stations
    pub runway_reports: Vec<RunwayReport<'input>>,
    /// Series of active weather conditions
    pub weather: Vec<Weather>,
    /// Describes observed cloud layers at different heights
    pub cloud_cover: Vec<CloudCover>,
    /// Indicative of OK ceiling and visibility
    ///
    /// While in the international standard, some countries do not use this. Notably, Canada
    pub cavok: bool,
    pub temperatures: Option<Temperatures>,
    pub pressure: Option<Pressure>,
    /// Often reported by Australian stations
    ///
    /// See the Australian Government [Bureau of Meteorology FAQ](http://www.bom.gov.au/aviation/about-us/faq/)
    pub accumulated_rainfall: Option<AccumulatedRainfall>,
    /// Often reported by military stations
    pub color: Option<Color>,
    pub recent_weather: Vec<Weather>,
    /// Often reported by stations at sea
    ///
    /// i.e. [ENQA](https://en.wikipedia.org/wiki/Troll_A_platform), an offshore natural gas platform.
    pub water_conditions: Option<WaterConditions>,
    pub trends: Vec<Trend>,
    /// Additional information outside of the METAR specification
    pub remark: Option<&'input str>,
    /// Some automated METARs indicate if the system needs maintenance
    ///
    /// This may indicate that measurements are unreliable
    pub maintenance_needed: bool,
}
