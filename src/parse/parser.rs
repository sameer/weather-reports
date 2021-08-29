use std::convert::TryFrom;
use uom::{
    si::angle::degree,
    si::f64::{Angle, Length, Pressure, TemperatureInterval, Velocity},
    si::length::{foot, kilometer, meter, mile, millimeter},
    si::pressure::{hectopascal, inch_of_mercury},
    si::temperature_interval::degree_celsius,
    si::velocity::{kilometer_per_hour, knot, meter_per_second},
};

use crate::tokens::*;

peg::parser! {
    pub grammar weather_reports() for str {
        /// [METAR report](https://en.wikipedia.org/wiki/METAR) parser
        pub rule metar() -> MetarReport<'input> =
                    whitespace()
                    report_name()? whitespace()
                    identifier:icao_identifier() whitespace()
                    observation_time:observation_time() whitespace()
                    // Some stations incorrectly place METAR here
                    report_name()? whitespace()
                    observation_type:observation_type()? whitespace()
                    wind:wind()? whitespace()
                    pre_temperatures:temperatures()? whitespace()
                    visibility:visibility()? whitespace()
                    runway_visibility:runway_visibility() ** whitespace() whitespace()
                    weather:weather() ** whitespace() whitespace()
                    cloud_cover:cloud_cover() ** whitespace() whitespace()
                    cavok:("CAVOK" whitespace())?
                    temperatures:temperatures()? whitespace()
                    pressure:pressure()? whitespace()
                    // Some stations also report the altimeter setting in a different unit and/or Q Field Elevation, discard it
                    pressure() ** whitespace() whitespace()
                    cloud_cover_post_pressure:cloud_cover() ** whitespace() whitespace()
                    accumulated_rainfall:accumulated_rainfall()? whitespace()
                    recent_weather:recent_weather() ** whitespace() whitespace()
                    // Military stations often report these
                    black:"BLACK"? whitespace()
                    color_state:color_state()? whitespace()
                    next_color_state:color_state()? whitespace()
                    // Some stations report runway visibility after pressure
                    runway_visibility_post_pressure:runway_visibility() ** whitespace() whitespace()
                    runway_reports:runway_report() ** whitespace() whitespace()
                    sea_conditions:sea_conditions()? whitespace()
                    trends:trend()** whitespace() whitespace()
                    // Some machines use = to indicate end of message
                    remark:$("RMK" [_]*)?
                    maintenance_needed:quiet!{"$"}?
                    quiet!{"/"*}
                    // machine indication of EOF
                    quiet!{"=" [_]*}?
                    {
                MetarReport {
                    identifier,
                    observation_time,
                    observation_type,
                    wind: wind.flatten(),
                    visibility: visibility.flatten(),
                    runway_visibility: runway_visibility.iter().copied().chain(runway_visibility_post_pressure).flatten().collect(),
                    runway_reports: runway_reports.iter().copied().flatten().collect(),
                    weather: weather.iter().cloned().flatten().collect(),
                    cloud_cover: cloud_cover.iter().copied().chain(cloud_cover_post_pressure).collect(),
                    cavok: cavok.is_some(),
                    temperatures: pre_temperatures.or(temperatures),
                    pressure: pressure.flatten(),
                    accumulated_rainfall,
                    is_closed: black.is_some(),
                    color_state,
                    next_color_state,
                    recent_weather: recent_weather.iter().cloned().flatten().collect(),
                    sea_conditions,
                    trends,
                    remark,
                    maintenance_needed: maintenance_needed.is_some(),
                }
            }
        rule report_name() -> &'input str = quiet!{$("METAR" / "SPECI")} / expected!("report name");

        pub rule icao_identifier() -> &'input str = quiet!{$(letter() letter_or_digit() letter_or_digit() letter_or_digit())};

        /// This must also consume garbage characters from irregular reports
        rule whitespace() = required_whitespace()?
        rule required_whitespace_or_eof() = (required_whitespace() / ![_])
        rule required_whitespace() = quiet!{ $(((" " "/"+)+ " ") / " " / "\r\n" / "\n" / "\t" / ">")+ } / expected!("whitespace");
        rule digit() -> &'input str = quiet!{$(['0'..='9'])} / expected!("digit");
        rule letter() -> &'input str = quiet!{$(['A'..='Z'])} / expected!("letter");
        rule letter_or_digit() -> &'input str = letter() / digit();

        pub rule observation_time() -> ZuluDateTime = day_of_month:$(digit() digit()) hour:$(digit() digit()) minute:$(digit() digit()) is_zulu:"Z"? {
            // TODO: some stations don't include the Z. Not sure if that could mean it is local time and not GMT.
            ZuluDateTime {
                day_of_month: day_of_month.parse().unwrap(),
                hour: hour.parse().unwrap(),
                minute: minute.parse().unwrap(),
                is_zulu: is_zulu.is_some(),
            }
        }

        rule observation_type() -> ObservationType = val:$(quiet!{"AUTO" / "COR" / "CCA" / "CCB" / "RTD"} / expected!("observation type")) { ObservationType::try_from(val).unwrap() };

        pub rule wind() -> Option<Wind> =
            direction:$("VRB" / (digit() digit() digit())) speed:$(("P" digit() digit()) / digit()+) peak_gust:$("G" ("//" / digit()+))? unit:windspeed_unit() whitespace() variance:wind_variance()? {
                let speed = speed.trim_start_matches("P").parse().unwrap();
                Some(Wind {
                    direction: if direction == "VRB" { None } else { Some(Angle::new::<degree>(direction.parse().unwrap())) },
                    speed: match unit {
                        "MPS" => Velocity::new::<meter_per_second>(speed),
                        "KT" | "KTS" | "KTM" => Velocity::new::<knot>(speed),
                        "KMH" => Velocity::new::<kilometer_per_hour>(speed),
                        _ => unreachable!()
                    },
                    peak_gust: peak_gust.filter(|gusts| *gusts != "G//").map(|gusts| gusts.trim_start_matches("G").parse().unwrap()).map(|gusts| match unit {
                        "MPS" => Velocity::new::<meter_per_second>(gusts),
                        "KT" | "KTS" | "KTM" => Velocity::new::<knot>(gusts),
                        "KMH" => Velocity::new::<kilometer_per_hour>(gusts),
                        _ => unreachable!()
                    }),
                    variance: variance
                })
            }
            / "/////" windspeed_unit()? {
                None
            }
        rule windspeed_unit() -> &'input str = $(quiet!{"MPS" / "KTM" / "KTS" / "KT" / "KMH"} / expected!("velocity unit"))


        rule wind_variance() -> (Angle, Angle) = variance_begin:$(digit()+) "V" variance_end:$(digit()+) {
            (
                Angle::new::<degree>(variance_begin.parse().unwrap()),
                Angle::new::<degree>(variance_end.parse().unwrap()),
            )
        }

        pub rule visibility() -> Option<Visibility> =
            // Some systems will attach a number in front of NDV
            (digit()*) "NDV" { None }
            / "////" "NDV"? unit:visibility_unit()? {
                None
            }
            / prevailing:raw_visibility() whitespace() minimum_directional:raw_directional_visibility() whitespace() maximum_directional:raw_directional_visibility() {
                Some(Visibility {
                    prevailing,
                    minimum_directional: Some(minimum_directional),
                    maximum_directional: Some(maximum_directional),
                })
            }
            / prevailing:raw_visibility() whitespace() minimum_directional:raw_directional_visibility() {
                Some(Visibility {
                    prevailing,
                    minimum_directional: Some(minimum_directional),
                    maximum_directional: None,
                })
            }
            / prevailing:raw_visibility() {
                Some(Visibility {
                    prevailing,
                    minimum_directional: None,
                    maximum_directional: None,
                })
            }
        rule raw_directional_visibility() -> DirectionalVisibility = distance:raw_visibility() direction:compass_direction() {
            DirectionalVisibility {
                distance,
                direction,
            }
        }
        rule raw_visibility() -> Length =
            whole:$(digit()+) whitespace() numerator:$(digit()+) "/" denominator:$(digit()+) whitespace() unit:visibility_unit()? {
                let value = whole.parse::<f64>().unwrap() + numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap();

                match unit {
                    Some("KM") => Length::new::<kilometer>(value),
                    Some("SM") => Length::new::<mile>(value),
                    Some("M") | None => Length::new::<meter>(value),
                    _ => unreachable!()
                }
            }
            / numerator:$(digit()+) "/" denominator:$(digit()+) whitespace() unit:visibility_unit()? {
                let value = numerator.parse::<f64>().unwrap() / denominator.parse::<f64>().unwrap();
                match unit {
                    Some("KM") => Length::new::<kilometer>(value),
                    Some("SM") => Length::new::<mile>(value),
                    Some("M") | None => Length::new::<meter>(value),
                    _ => unreachable!()
                }
            }
            / value:$(digit()+) whitespace() unit:visibility_unit()? {
                let value = value.parse::<f64>().unwrap();
                match unit {
                    Some("KM") => Length::new::<kilometer>(value),
                    Some("SM") => Length::new::<mile>(value),
                    Some("M") | None => Length::new::<meter>(value),
                    _ => unreachable!()
                }
            }

        rule compass_direction() -> CompassDirection = val:$(quiet!{"NE" / "NW" / "N" / "SE" / "SW" / "S" / "E" / "W"} / expected!("8-point compass direction")) {
            CompassDirection::try_from(val).unwrap()
        }
        rule visibility_unit() -> &'input str = val:$(quiet!{"M" / "KM" / "SM"} / expected!("visibility unit")) &required_whitespace_or_eof() { val }

        pub rule runway_visibility() -> Option<RunwayVisibility<'input>> =
            "R" designator:designator() "/" !runway_report_info() lower:raw_runway_visibility() "V" upper:raw_runway_visibility() trend:visibility_trend()? {
                Some(RunwayVisibility {
                    designator,
                    visibility: VisibilityType::Varying {
                        lower,
                        upper,
                    },
                    trend,
                })
            }
            / "R" designator:designator() "/" !runway_report_info() visibility:raw_runway_visibility() trend:visibility_trend()? {
                Some(RunwayVisibility {
                    designator,
                    visibility: VisibilityType::Fixed(visibility),
                    trend,
                })
            }
            // A varying number of slashes and a missing designator has been observed here
            / "R" designator:designator()? ("/////" "/"*) &required_whitespace_or_eof() {
                None
            }
        rule raw_runway_visibility() -> RawRunwayVisibility = out_of_range:out_of_range()? value:$(digit()+) unit:$("FT")? {
            let value = value.parse::<f64>().unwrap();
            if let Some("FT") = unit {
                RawRunwayVisibility {
                    value: Length::new::<foot>(value),
                    out_of_range,
                }
            } else {
                RawRunwayVisibility {
                    value: Length::new::<meter>(value),
                    out_of_range,
                }
            }
        }
        rule out_of_range() -> OutOfRange = val:$(quiet!{"M" / "P"} / expected!("bound")) { OutOfRange::try_from(val).unwrap() };
        rule visibility_trend() -> VisibilityTrend = "/"? val:$(quiet!{("D" / "N" / "U")} / expected!("visibility trend")) { VisibilityTrend::try_from(val.trim_start_matches("/")).unwrap() };

        pub rule runway_report() -> Option<RunwayReport<'input>> =
            "R" designator:designator() "/" report_info:runway_report_info() {
                Some(RunwayReport {
                    designator,
                    report_info,
                })
            }
        rule runway_report_info() -> RunwayReportInfo =
            "CLRD" friction:$("//" / digit()+) {
                RunwayReportInfo::Cleared {
                    friction: if friction == "//" { None } else { Some(friction.parse::<f64>().unwrap()) }
                }
            }

        rule designator() -> &'input str = $(quiet!{digit()+ ("L"/"C"/"R"/"D")?} / expected!("runway designator"));


        rule recent_weather() -> Option<Weather> = "RE" weather:weather() { weather }

        pub rule weather() -> Option<Weather> =
            "//" {
                None
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor()? precipitation:precipitation()+ {
                Some(Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor,
                    condition: Some(Condition::Precipitation(precipitation)),
                })
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor()? obscuration:obscuration() {
                Some(Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor,
                    condition: Some(Condition::Obscuration(obscuration)),
                })
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor()? other:other() {
                Some(Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor,
                    condition: Some(Condition::Other(other)),
                })
            }
            / intensity:intensity() vicinity:"VC"? descriptor:descriptor() {
                Some(Weather {
                    intensity,
                    vicinity: vicinity.is_some(),
                    descriptor: Some(descriptor),
                    condition: None,
                })
            }
        rule intensity() -> Intensity = val:$(quiet!{[ '+' | '-' ]} / expected!("intensity"))? { val.map(Intensity::try_from).transpose().unwrap().unwrap_or(Intensity::Moderate) }
        rule descriptor() -> Descriptor =
            val:$(quiet!{
                "MI"
                / "PR"
                / "BC"
                / "DR"
                / "BL"
                / "SH"
                / "TS"
                / "FZ"
            } / expected!("descriptor")) {
                Descriptor::try_from(val).unwrap()
        }

        rule precipitation() -> Precipitation =
            val:$(quiet!{
                "RA"
                / "DZ"
                / "SN"
                / "SG"
                / "IC"
                / "PL"
                / "GR"
                / "GS"
                / "UP"
            } / expected!("precipitation")) {
                Precipitation::try_from(val).unwrap()
        }

        rule obscuration() -> Obscuration =
        val:$(quiet!{
                "FG"
                / "BR"
                / "HZ"
                / "VA"
                / "DU"
                / "FU"
                / "SA"
                / "PY"
            } / expected!("obscuration")) {
                Obscuration::try_from(val).unwrap()
        }

        rule other() -> Other =
            val:$(quiet!{
                "SQ"
                / "PO"
                / "DS"
                / "SS"
                / "FC"
            } / expected!("other weather condition")) {
                Other::try_from(val).unwrap()
        }


        pub rule cloud_cover() -> CloudCover =
            coverage:cloud_coverage() whitespace() "///" whitespace() cloud_type:cloud_type()? {
                CloudCover {
                    coverage,
                    base: None,
                    cloud_type: cloud_type.flatten(),
                }
            }
            / coverage:cloud_coverage() whitespace() base:$(digit() digit() digit()) whitespace() cloud_type:cloud_type()? {
                CloudCover {
                    coverage,
                    base: Some(Length::new::<foot>(base.parse().unwrap()) * 100.),
                    cloud_type: cloud_type.flatten(),
                }
            }
            / coverage:cloud_coverage() {
                CloudCover {
                    coverage: CloudCoverage::try_from(coverage).unwrap(),
                    base: None,
                    cloud_type: None,
                }
            }

        rule cloud_coverage() -> CloudCoverage =
            val:$(quiet!{
                "SKC"
                / "CLR"
                / "NCD"
                / "NSC"
                / "FEW"
                / "FW"
                / "SCT"
                / "SC"
                / "BKN"
                / "OVC"
                / "VV"
            } / expected!("cloud coverage")) {
                CloudCoverage::try_from(val).unwrap()
            }

        rule cloud_type() -> Option<CloudType> =
            val:$(quiet!{"CB" / "TCU" / "CU" / "CI" / "AC" / "ST"} / expected!("cloud type")) { Some(CloudType::try_from(val).unwrap()) }
            / "///" {
                None
            }


        rule temperature() -> TemperatureInterval = minus:(quiet!{"M" / "-"} / expected!("minus"))? temp:$(digit()+) {
            TemperatureInterval::new::<degree_celsius>(if minus.is_some() { -temp.parse::<f64>().unwrap() } else { temp.parse().unwrap() })
        }

        pub rule temperatures() -> Temperatures =
            air:temperature() ("/" / ".") ("XX" / "//") !"SM" {
                Temperatures {
                    air,
                    dewpoint: None,
                }
            }
            / air:temperature() ("/" / ".") dewpoint:temperature()? !"SM" {
                Temperatures {
                    air,
                    dewpoint
                }
            }

        pub rule pressure() -> Option<Pressure> =
            pressure_unit:pressure_unit() whitespace() pressure:$(digit()+ ("." digit()+)?) {
                match pressure_unit {
                    "A" => Some(Pressure::new::<hectopascal>(pressure.parse().unwrap())),
                    _ => Some(Pressure::new::<inch_of_mercury>(pressure.parse::<f64>().unwrap() / 100.))
                }
            }
            / pressure_unit() whitespace() ("////" / "NIL") { None }
        rule pressure_unit() -> &'input str = $(quiet!{"QFE" / "QNH" / "Q" / "A"} / expected!("pressure unit"));

        rule accumulated_rainfall() -> AccumulatedRainfall = "RF" recent:$(digit()+ "." digit()+) "/" past:$(digit()+ "." digit()+) {
            AccumulatedRainfall {
                recent: Length::new::<millimeter>(recent.parse().unwrap()),
                past: Length::new::<millimeter>(past.parse().unwrap()),
            }
        }

        rule color_state() -> ColorState = val:$(quiet!{"BLU+" / "BLU" / "WHT" / "GRN" / "YLO1" / "YLO2" / "YLO" / "AMB" / "RED"} / expected!("color state")) { ColorState::try_from(val).unwrap() }

        rule sea_conditions() -> SeaConditions = "W" temperature:$("//" / digit()+) "/" "S" wave_intensity:$("/" / digit()) {
            SeaConditions {
                temperature: if temperature == "//" { None } else { Some(TemperatureInterval::new::<degree_celsius>(temperature.parse().unwrap()))},
                wave_intensity: if wave_intensity == "/" { None } else { Some(wave_intensity.parse().unwrap()) },
            }
        }

        rule trend() -> Trend =
            $(quiet!{"NOSIG" / "NSG"} / expected!("trend")) {
                Trend::NoSignificantChange
            }
            /   val:$(quiet!{"BECMG" / "TEMPO"} / expected!("trend")) whitespace()
                time:trend_time()? whitespace()
                wind:wind()? whitespace()
                visibility:visibility()? whitespace()
                weather:weather() ** whitespace() whitespace()
                cloud_cover:cloud_cover() ** whitespace() whitespace()
                color_state:color_state()? whitespace() {
                    let trend = TrendReport {
                        time,
                        wind: wind.flatten(),
                        visibility: visibility.flatten(),
                        weather: weather.iter().cloned().flatten().collect(),
                        cloud_cover,
                        color_state,
                    };
                    match val {
                        "BECMG" => Trend::Becoming(trend),
                        "TEMPO" => Trend::Temporarily(trend),
                        _ => unreachable!()
                    }
            }
        rule trend_time() -> TrendTime = time_type:trend_time_type() time:$(digit() digit() digit() digit()) {
            TrendTime {
                time: time.parse().unwrap(),
                time_type,
            }
        };
        rule trend_time_type() -> TrendTimeType = val:$(quiet!{"AT" / "FM" / "TL"} / expected!("trend time type")) { TrendTimeType::try_from(val).unwrap() }
    }
}
