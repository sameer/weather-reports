pub mod parse;
pub mod tokens;

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read};

    use codespan_reporting::term::{
        emit,
        termcolor::{ColorChoice, StandardStream},
    };
    use zstd::Decoder;

    use crate::parse::into_diagnostic;

    #[test]
    fn validate_against_year_of_ktpa_metar_reports() {
        let mut reports = String::new();
        Decoder::new(Cursor::new(include_bytes!("../tests/ktpa.txt.zst")))
            .unwrap()
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
    fn validate_against_all_ogimet_latest_reports_by_country() {
        tar::Archive::new(
            Decoder::new(Cursor::new(include_bytes!("../tests/countries.tar.zst"))).unwrap(),
        )
        .entries()
        .unwrap()
        .for_each(|entry| {
            let mut entry = entry.unwrap();
            let mut html_page = String::default();
            entry.read_to_string(&mut html_page).unwrap();
            if html_page.contains("No METAR/SPECI reports") {
                return;
            }
            let reports_in_country = html_page
                .rsplit("<pre>")
                .next()
                .unwrap()
                .split("</pre>")
                .next()
                .unwrap()
                .rsplit("###################################")
                .next()
                .unwrap();

            let mut acc = 0;
            let errors = reports_in_country
                .split('=')
                .filter(|report| report.len() >= 14)
                .map(|report| report.split_at(13).1)
                .filter(|report| {
                    // Skip Canada SAO observations
                    if report.contains("AUTO8") {
                        false
                    } else {
                        true
                    }
                })
                .filter_map(|report| {
                    acc += 1;
                    if let Err(err) = crate::parse::metar(report) {
                        let mut writer = StandardStream::stderr(ColorChoice::Never);
                        let config = codespan_reporting::term::Config::default();
                        emit(
                            &mut writer,
                            &config,
                            &codespan_reporting::files::SimpleFile::new(
                                format!("countries/{}", entry.path().unwrap().to_string_lossy()),
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
                println!(
                    "### {}: {} failures out of {} total cases ({:.2}% coverage)",
                    entry.path().unwrap().to_string_lossy(),
                    errors.len(),
                    acc,
                    (100. - errors.len() as f64 / acc as f64 * 100.)
                );
            }
        });
    }

    #[test]
    fn validate_against_all_ogimet_august_2021_reports_by_station() {
        let mut zst = vec![];
        reqwest::blocking::get(
            "http://localhost:8080/ipfs/QmSafVXLeEYiSmofSxKcWCAHfRz9EwHAkS1tYDRPtxt7qz",
        )
        .or_else(|_| {
            reqwest::blocking::get(
                "https://ipfs.io/ipfs/QmSafVXLeEYiSmofSxKcWCAHfRz9EwHAkS1tYDRPtxt7qz",
            )
        })
        .unwrap()
        .read_to_end(&mut zst)
        .unwrap();
        tar::Archive::new(Decoder::new(Cursor::new(zst)).unwrap())
            .entries()
            .unwrap()
            .for_each(|entry| {
                let mut entry = entry.unwrap();
                let mut html_page = String::default();
                entry.read_to_string(&mut html_page).unwrap();
                if html_page.contains("No METAR/SPECI reports") {
                    return;
                }
                let reports_in_country = html_page
                    .rsplit("<pre>")
                    .next()
                    .unwrap()
                    .split("</pre>")
                    .next()
                    .unwrap()
                    .rsplit("###################################")
                    .next()
                    .unwrap();

                let mut acc = 0;
                let errors = reports_in_country
                    .split('=')
                    .filter(|report| report.len() >= 14)
                    .map(|report| report.split_at(13).1)
                    .filter(|report| {
                        // Skip Canada SAO observations
                        if report.contains("AUTO8") {
                            false
                        } else {
                            true
                        }
                    })
                    .filter_map(|report| {
                        acc += 1;
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
                    println!(
                        "### {}: {} failures out of {} total cases ({:.2}% coverage)",
                        entry.path().unwrap().to_string_lossy(),
                        errors.len(),
                        acc,
                        (100. - errors.len() as f64 / acc as f64 * 100.)
                    );
                }
            });
    }
}
