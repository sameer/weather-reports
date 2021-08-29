# weather-reports

A collection of weather report parsers

[![crates.io](https://img.shields.io/crates/v/weather-reports.svg)](https://crates.io/crates/weather-reports) [![weather-reports](https://docs.rs/weather-reports/badge.svg)](https://docs.rs/weather-reports)[![weather-reports](https://github.com/sameer/weather-reports/actions/workflows/rust.yml/badge.svg)](https://github.com/sameer/weather-reports/actions/workflows/rust.yml) [![codecov](https://codecov.io/gh/sameer/weather-reports/branch/main/graph/badge.svg?token=TPIzIZtbdq)](https://codecov.io/gh/sameer/weather-reports)

## Supported Formats

- [x] [METAR](https://en.wikipedia.org/wiki/METAR)/SPECI
  - [ ] Remark parsing
- [ ] [TAF](https://en.wikipedia.org/wiki/Terminal_aerodrome_forecast)

## Demo

```
> cargo run --release --example parse -
KSEA 290353Z 01008KT 10SM SCT200 19/09 A3004 RMK AO2 SLP179 T01940094
MetarReport {
    identifier: "KSEA",
    observation_time: ZuluDateTime {
        day_of_month: 29,
        hour: 3,
        minute: 53,
        is_zulu: true,
    },
    observation_type: None,
    wind: Some(
        Wind {
            direction: Some(
                0.17453292519943295,
            ),
            speed: 4.115555555555556 m^1 s^-1,
            peak_gust: None,
            variance: None,
        },
    ),
    visibility: Some(
        Visibility {
            prevailing: 16093.44 m^1,
            minimum_directional: None,
            maximum_directional: None,
        },
    ),
    runway_visibility: [],
    runway_reports: [],
    weather: [],
    cloud_cover: [
        CloudCover {
            coverage: Scattered,
            base: Some(
                6096.0 m^1,
            ),
            cloud_type: None,
        },
    ],
    cavok: false,
    temperatures: Some(
        Temperatures {
            air: 19.0 K^1,
            dewpoint: Some(
                9.0 K^1,
            ),
        },
    ),
    pressure: Some(
        300400.0 m^-1 kg^1 s^-2,
    ),
    accumulated_rainfall: None,
    is_closed: false,
    color_state: None,
    next_color_state: None,
    recent_weather: [],
    sea_conditions: None,
    trends: [],
    remark: Some(
        "RMK AO2 SLP179 T01940094\n",
    ),
    maintenance_needed: false,
}
Success!
```
