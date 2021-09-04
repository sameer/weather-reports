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
        time: ZuluTime {
            hour: 3,
            minute: 53,
        },
        is_zulu: true,
    },
    observation_validity_range: None,
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
    runway_visibilities: [],
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
            air: 292.15 K^1,
            dewpoint: Some(
                282.15 K^1,
            ),
        },
    ),
    pressure: Some(
        300400.0 m^-1 kg^1 s^-2,
    ),
    accumulated_rainfall: None,
    color: None,
    recent_weather: [],
    water_conditions: None,
    trends: [],
    remark: Some(
        "RMK AO2 SLP179 T01940094\n",
    ),
    maintenance_needed: false,
}
Success!
```

## Debugging

Each example generates a parser trace when the trace feature is enabled. To generate and visualize one with [pegviz](https://github.com/fasterthanlime/pegviz):

```
cargo run --release --features trace --example metar - | pegviz --output index.html && firefox index.html
```

## References

- https://sto.iki.fi/metar/
- https://aviation.stackexchange.com/questions/39482/what-is-this-weird-format-r06l-clrd62-in-metar-for-runway-being-cleared
- https://metar-taf.com/explanation
- http://www.bom.gov.au/aviation/about-us/faq/
- https://mediawiki.ivao.aero/index.php?title=METAR_explanation
- https://business.desu.edu/sites/business/files/document/16/metar_and_taf_codes.pdf
- https://meteocentre.com/doc/metar.html
- https://aviation.stackexchange.com/questions/42554/what-does-the-code-fu1fu2fu5-mean-in-this-metar
- https://library.wmo.int/doc_num.php?explnum_id=10474
- https://library.wmo.int/doc_num.php?explnum_id=10235
- https://en.wikipedia.org/wiki/METAR
- https://aviation.stackexchange.com/questions/88908/what-does-w13-s3-mean-in-this-metar-report
- http://www.ogimet.com/index.phtml.en
