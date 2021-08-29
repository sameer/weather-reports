pub mod parse;
pub mod tokens;

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use codespan_reporting::term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
    };
    use flate2::read::GzDecoder;

    use crate::parse::into_diagnostic;

    #[test]
    fn cwlc_metar_report() {
        crate::parse::metar("CWLC 270000Z AUTO 14/ RMK AO1 T0137").unwrap();
    }

    #[test]
    fn epsy_metar_report() {
        crate::parse::metar("EPSY 290130Z 19002KT 2000 MIFG").unwrap();
    }

    #[test]
    fn ubbl_metar_report() {
        crate::parse::metar("UBBL 262300Z VRB02KT 9999 NSC 22/17 Q1012 R33/CLRD// NOSIG RMK MT OP")
            .unwrap();
    }

    #[test]
    fn ksea_metar_report() {
        crate::parse::metar("KSEA 251453Z 18004KT 10SM FEW025 SCT250 14/09 A3002 RMK AO2 SLP171 FU FEW025 T01390094 53009").unwrap();
    }

    #[test]
    fn kbna_metar_report() {
        crate::parse::metar(
            "METAR KBNA 261453Z 00000KT 10SM FEW200 31/22 A3016 RMK AO2 SLP204 T03110222 51004",
        )
        .unwrap();
    }

    #[test]
    fn validate_against_year_of_ktpa_metar_reports() {
        let mut reports = String::new();
        GzDecoder::new(Cursor::new(include_bytes!("../tests/ktpa.txt.gz")))
            .read_to_string(&mut reports)
            .unwrap();
        let errors = reports
            .split('\n')
            .map(|report| report.split_at(13).1)
            .filter_map(|report| {
                if let Err(err) = crate::parse::metar(report) {
                    let mut writer = StandardStream::stderr(ColorChoice::Never);
                    let config = codespan_reporting::term::Config::default();
                    emit(
                        &mut writer,
                        &config,
                        &codespan_reporting::files::SimpleFile::new("<metar_report>", report),
                        &into_diagnostic(&err),
                    )
                    .unwrap();
                    Some(err)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            panic!("{:#?}", errors);
        }
    }

    #[test]
    fn validate_against_all_noaa_station_metar_reports() {
        let mut archive = vec![];
        GzDecoder::new(Cursor::new(include_bytes!("../tests/stations.tar.gz")))
            .read_to_end(&mut archive)
            .unwrap();
        let errors = tar::Archive::new(Cursor::new(archive))
            .entries()
            .unwrap()
            .filter_map(|entry| {
                let mut entry = entry.unwrap();
                let mut report = String::default();
                if let Err(err) = entry.read_to_string(&mut report) {
                    return None;
                    // panic!("{} {}", err, entry.path().unwrap().to_string_lossy())
                }
                let report = report.split('\n').skip(1).next().unwrap();
                if let Err(err) = crate::parse::metar(report) {
                    let mut writer = StandardStream::stderr(ColorChoice::Never);
                    let config = codespan_reporting::term::Config::default();
                    emit(
                        &mut writer,
                        &config,
                        &codespan_reporting::files::SimpleFile::new(
                            format!("stations/{}", entry.path().unwrap().to_string_lossy()),
                            report,
                        ),
                        &into_diagnostic(&err),
                    )
                    .unwrap();
                    Some(err)
                } else {
                    None
                }
            })
            .collect::<Vec<_>>();

        if !errors.is_empty() {
            println!("There are {} failures", errors.len());
        }
    }
}
