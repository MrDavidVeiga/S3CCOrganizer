use s3cc_organizer_lib::{i18n::AppLanguage, scanner::scan_packages};
use std::env;

fn main() {
    let mut args = env::args().skip(1);
    let folder = args.next().unwrap_or_else(|| ".".to_string());
    let language = match args.next().as_deref() {
        Some("pt") => AppLanguage::Pt,
        Some("es") => AppLanguage::Es,
        _ => AppLanguage::En,
    };

    match scan_packages(folder, language) {
        Ok(result) => {
            println!(
                "{}",
                serde_json::to_string_pretty(&result)
                    .expect("serialize scanner result")
            );
        }
        Err(error) => {
            eprintln!("SCAN_ERROR: {error}");
            std::process::exit(2);
        }
    }
}
