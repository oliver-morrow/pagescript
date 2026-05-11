use std::{env, fs, process};

use pagescript_rs::{
    Resolver, compile_page_ir, parse_page_script, render_to_html, to_intro_config,
    to_shepherd_config, validate_document,
};

#[derive(Default)]
struct CliOptions {
    command: Option<String>,
    file: Option<String>,
    target: Option<String>,
    tour_id: Option<String>,
    page_id: Option<String>,
    version: bool,
}

fn main() {
    process::exit(run(env::args().skip(1).collect()));
}

fn run(args: Vec<String>) -> i32 {
    let options = match parse_args(args) {
        Ok(options) => options,
        Err(error) => {
            eprintln!("{error}");
            print_usage();
            return 1;
        }
    };
    if options.version {
        println!("pagescript-rs {}", env!("CARGO_PKG_VERSION"));
        return 0;
    }
    let Some(command) = options.command.as_deref() else {
        print_usage();
        return 1;
    };
    let Some(file) = options.file.as_deref() else {
        print_usage();
        return 1;
    };

    if !matches!(command, "validate" | "ast" | "ir" | "convert" | "render") {
        eprintln!("Unknown command: {command}");
        print_usage();
        return 1;
    }

    let source = match fs::read_to_string(file) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("Failed to read {file}: {error}");
            return 1;
        }
    };
    let file_path = std::path::Path::new(file);
    let base_path = file_path.parent().map(|p| p.to_path_buf());
    let resolver = Resolver::new(base_path);

    let document = parse_page_script(&source);
    let diagnostics = validate_document(&document, &resolver);

    if command == "validate" {
        if diagnostics.is_empty() {
            println!("{file} is valid");
            return 0;
        }
        print_diagnostics(&diagnostics);
        return 1;
    }

    if !diagnostics.is_empty() {
        print_diagnostics(&diagnostics);
        return 1;
    }

    if command == "ast" {
        return print_json(&document);
    }

    if command == "ir" {
        return match compile_page_ir(&document, options.page_id.as_deref(), &resolver) {
            Ok(ir) => print_json(&ir),
            Err(error) => {
                eprintln!("{error}");
                1
            }
        };
    }

    if command == "render" {
        return match render_to_html(&document, options.page_id.as_deref(), &resolver) {
            Ok(html) => {
                println!("{html}");
                0
            }
            Err(error) => {
                eprintln!("{error}");
                1
            }
        };
    }

    let Some(target) = options.target.as_deref() else {
        eprintln!("Missing --target shepherd|intro");
        return 1;
    };
    let result = match target {
        "shepherd" => to_shepherd_config(&document, options.tour_id.as_deref())
            .and_then(|config| serde_json::to_value(config).map_err(|error| error.to_string())),
        "intro" => to_intro_config(&document, options.tour_id.as_deref())
            .and_then(|config| serde_json::to_value(config).map_err(|error| error.to_string())),
        _ => Err("Missing --target shepherd|intro".to_string()),
    };

    match result {
        Ok(value) => print_json(&value),
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn parse_args(args: Vec<String>) -> Result<CliOptions, String> {
    let mut options = CliOptions::default();
    let mut iter = args.into_iter();
    options.command = iter.next();
    if options.command.as_deref() == Some("--version") {
        options.version = true;
        if iter.next().is_some() {
            return Err("--version does not accept extra arguments.".to_string());
        }
        return Ok(options);
    }
    options.file = iter.next();

    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--target" => options.target = Some(next_flag_value(&mut iter, "--target")?),
            "--tour" => options.tour_id = Some(next_flag_value(&mut iter, "--tour")?),
            "--page" => options.page_id = Some(next_flag_value(&mut iter, "--page")?),
            _ if arg.starts_with('-') => return Err(format!("Unknown flag: {arg}")),
            _ => return Err(format!("Unexpected argument: {arg}")),
        }
    }

    Ok(options)
}

fn next_flag_value(iter: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
    match iter.next() {
        Some(value) if !value.starts_with('-') => Ok(value),
        Some(value) => Err(format!("Missing value for {flag}; found {value}.")),
        None => Err(format!("Missing value for {flag}.")),
    }
}

fn print_diagnostics(diagnostics: &[pagescript_rs::Diagnostic]) {
    for diagnostic in diagnostics {
        let severity = match diagnostic.severity {
            pagescript_rs::Severity::Error => "ERROR",
            pagescript_rs::Severity::Warning => "WARNING",
        };
        eprintln!(
            "{} {} at {}:{}: {}",
            severity, diagnostic.code, diagnostic.line, diagnostic.column, diagnostic.message
        );
    }
}

fn print_json<T: serde::Serialize>(value: &T) -> i32 {
    match serde_json::to_string_pretty(value) {
        Ok(json) => {
            println!("{json}");
            0
        }
        Err(error) => {
            eprintln!("Failed to serialize JSON: {error}");
            1
        }
    }
}

fn print_usage() {
    eprintln!(
        "PageScript Draft 0.6\nUsage:\n  pagescript-rs --version\n  pagescript-rs validate <file>\n  pagescript-rs ast <file>\n  pagescript-rs ir <file> [--page id]\n  pagescript-rs render <file> [--page id]\n  pagescript-rs convert <file> --target shepherd|intro [--tour id]"
    );
}

#[cfg(test)]
mod tests {
    use super::run;
    use std::{env, fs};

    #[test]
    fn validate_valid_file_returns_zero() {
        let path = env::temp_dir().join("pagescript-rs-valid-minimal.tour");
        fs::write(
            &path,
            r##"::tour id=minimal
  ::step id=one target="#one"
  ::/step
  ::trigger type=manual
  ::/trigger
::/tour
"##,
        )
        .unwrap();

        assert_eq!(run(vec!["validate".into(), path.display().to_string()]), 0);
    }

    #[test]
    fn version_returns_zero_without_file() {
        assert_eq!(run(vec!["--version".into()]), 0);
    }

    #[test]
    fn unknown_command_returns_error() {
        assert_eq!(run(vec!["nope".into(), "file.page".into()]), 1);
    }

    #[test]
    fn unknown_flag_returns_error() {
        assert_eq!(
            run(vec![
                "validate".into(),
                "file.page".into(),
                "--bogus".into()
            ]),
            1
        );
    }

    #[test]
    fn missing_flag_value_returns_error() {
        assert_eq!(
            run(vec!["render".into(), "file.page".into(), "--page".into()]),
            1
        );
    }

    #[test]
    fn missing_selected_page_returns_error() {
        let path = env::temp_dir().join("pagescript-rs-page-selection.page");
        fs::write(
            &path,
            r##"::page id=actual
::/page
"##,
        )
        .unwrap();

        assert_eq!(
            run(vec![
                "ir".into(),
                path.display().to_string(),
                "--page".into(),
                "missing".into()
            ]),
            1
        );
    }

    #[test]
    fn missing_selected_tour_returns_error() {
        let path = env::temp_dir().join("pagescript-rs-tour-selection.page");
        fs::write(
            &path,
            r##"::tour id=actual
  ::step id=one target="#one"
  ::/step
::/tour
"##,
        )
        .unwrap();

        assert_eq!(
            run(vec![
                "convert".into(),
                path.display().to_string(),
                "--target".into(),
                "shepherd".into(),
                "--tour".into(),
                "missing".into()
            ]),
            1
        );
    }
}
