use codespan_reporting::term::{
    emit,
    termcolor::{ColorChoice, StandardStream},
};
use std::io::Read;

use weather_reports::parse::metar;

fn main() {
    let filename = std::env::args().skip(1).next().expect("specify a filename");

    let report: String = match filename.as_ref() {
        "-" => {
            let mut acc = String::default();
            std::io::stdin().read_to_string(&mut acc).unwrap();
            acc
        }
        filename => std::fs::read_to_string(&filename).expect("file isn't readable"),
    };

    match metar(&report) {
        Ok(ast) => {
            println!("{:#?}", ast);
            eprintln!("Success!");
        }
        Err(err) => {
            let mut writer = StandardStream::stderr(ColorChoice::Auto);
            let config = codespan_reporting::term::Config::default();
            emit(
                &mut writer,
                &config,
                &codespan_reporting::files::SimpleFile::new(
                    if filename == "-" {
                        "<stdin>"
                    } else {
                        filename.as_str()
                    },
                    &report,
                ),
                &weather_reports::parse::into_diagnostic(&err),
            )
            .unwrap();
        }
    }
}
