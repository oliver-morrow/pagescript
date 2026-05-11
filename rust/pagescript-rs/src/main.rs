use std::{
    env, fs,
    path::{Path, PathBuf},
    process,
};

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
    template: Option<String>,
    out: Option<String>,
    version: bool,
    force: bool,
    json: bool,
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
        println!("{} {}", cli_name(), env!("CARGO_PKG_VERSION"));
        return 0;
    }
    let Some(command) = options.command.as_deref() else {
        print_usage();
        return 1;
    };
    if command == "guide" {
        if options.file.is_some() {
            eprintln!("guide does not accept a file argument.");
            print_usage();
            return 1;
        }
        println!("{LLM_GUIDE}");
        return 0;
    }
    let Some(file) = options.file.as_deref() else {
        print_usage();
        return 1;
    };

    if !matches!(
        command,
        "validate" | "ast" | "ir" | "convert" | "render" | "new"
    ) {
        eprintln!("Unknown command: {command}");
        print_usage();
        return 1;
    }

    if command == "new" {
        return create_new_page(file, options.template.as_deref(), options.force);
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
        if options.json {
            let json_status = print_json(&diagnostics);
            if json_status != 0 {
                return json_status;
            }
            return if diagnostics.is_empty() { 0 } else { 1 };
        }
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
                if let Some(out) = options.out.as_deref() {
                    match write_file(out, &html) {
                        Ok(()) => {
                            println!("Rendered {file} to {out}");
                            0
                        }
                        Err(error) => {
                            eprintln!("{error}");
                            1
                        }
                    }
                } else {
                    println!("{html}");
                    0
                }
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
            "--template" => options.template = Some(next_flag_value(&mut iter, "--template")?),
            "--out" => options.out = Some(next_flag_value(&mut iter, "--out")?),
            "--force" => options.force = true,
            "--json" => options.json = true,
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

fn create_new_page(file: &str, template: Option<&str>, force: bool) -> i32 {
    let source = match template_source(template.unwrap_or("product")) {
        Ok(source) => source,
        Err(error) => {
            eprintln!("{error}");
            return 1;
        }
    };
    let path = Path::new(file);
    if path.exists() && !force {
        eprintln!("{file} already exists. Re-run with --force to overwrite it.");
        return 1;
    }
    match write_file(file, source) {
        Ok(()) => {
            println!("Created {file}");
            println!("Next: {} render {file} --out index.html", cli_name());
            0
        }
        Err(error) => {
            eprintln!("{error}");
            1
        }
    }
}

fn template_source(template: &str) -> Result<&'static str, String> {
    match template {
        "product" => Ok(PRODUCT_TEMPLATE),
        "dashboard" => Ok(DASHBOARD_TEMPLATE),
        "docs" => Ok(DOCS_TEMPLATE),
        _ => Err(format!(
            "Unknown template: {template}. Expected product, dashboard, or docs."
        )),
    }
}

fn write_file(file: &str, contents: &str) -> Result<(), String> {
    let path = PathBuf::from(file);
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)
            .map_err(|error| format!("Failed to create {}: {error}", parent.display()))?;
    }
    fs::write(&path, contents).map_err(|error| format!("Failed to write {file}: {error}"))
}

fn cli_name() -> &'static str {
    option_env!("CARGO_BIN_NAME").unwrap_or("pagescript")
}

fn print_usage() {
    let name = cli_name();
    eprintln!(
        "PageScript Draft 0.6\nUsage:\n  {name} --version\n  {name} guide\n  {name} new <file.page> [--template product|dashboard|docs] [--force]\n  {name} validate <file> [--json]\n  {name} ast <file>\n  {name} ir <file> [--page id]\n  {name} render <file> [--page id] [--out output.html]\n  {name} convert <file> --target shepherd|intro [--tour id]"
    );
}

const LLM_GUIDE: &str = r##"# PageScript LLM Generation Guide

Use PageScript as a small semantic DSL, not as renamed HTML.

Grammar:
- Every block starts with `::name key=value` and ends with `::/name`.
- Attributes are strings, quoted strings, booleans, numbers, or JSON objects.
- Prefer semantic blocks: hero, section, nav, card, form, filter, table, empty-state, scene, panel, metric.
- Use recipes and Web Core Kernel only when semantic primitives are not enough.
- Do not use raw HTML or JavaScript. `raw` and `script` are rejected in conformance mode.

Minimal page:
```text
::page id=demo title="Demo"
  ::hero heading="Useful page" body="Compact source, deterministic HTML."
  ::/hero
::/page
```

Dashboard/table pattern:
```text
::page id=ops title="Ops"
  ::nav label="Primary"
    ::nav-item label="Overview" href="#overview"
    ::/nav-item
  ::/nav

  ::section id=overview heading="Accounts"
    ::filter id=account-search label="Search accounts" placeholder="Company or owner"
    ::/filter
    ::table id=accounts
      ::column label="Account"
      ::/column
      ::column label="Status"
      ::/column
      ::row
        ::cell value="Acme"
        ::/cell
        ::cell value="Healthy"
        ::/cell
      ::/row
    ::/table
  ::/section
::/page
```

Repair loop:
1. Generate `.page`.
2. Run `pagescript validate file.page --json`.
3. Fix the first diagnostic by line number.
4. Repeat until diagnostics are `[]`.
5. Run `pagescript render file.page --out index.html`.
"##;

const PRODUCT_TEMPLATE: &str = r##"::page id=product-demo title="Product Demo"
  ::tokens
    color.accent="#4dd6a0"
    radius.panel=14
  ::/tokens

  ::hero spacing=xl
    heading="Launch a useful page from one file"
    body="PageScript turns compact semantic source into standalone HTML that AI tools and humans can both edit."
  ::/hero

  ::section spacing=lg
    heading="Why it works"
    body="Start with a source file, render it locally, and publish the generated HTML anywhere static files can go."

    ::grid columns=3 gap=md density=compact
      ::card icon="01" title="Compact source"
        body="Write the intent of the page instead of hand-authoring every div, class, and runtime hook."
      ::/card
      ::card icon="02" title="Standalone output"
        body="The renderer emits browser-native HTML and CSS with no application server required."
      ::/card
      ::card icon="03" title="Agent friendly"
        body="Codex, Claude, and Cursor can review and modify PageScript without losing the page structure."
      ::/card
    ::/grid
  ::/section
::/page
"##;

const DASHBOARD_TEMPLATE: &str = r##"::page id=dashboard title="Dashboard"
  ::tokens
    color.accent="#3b82f6"
    radius.panel=12
  ::/tokens

  ::scene id=overview layout=split title="Operating dashboard"
    heading="Live business health"
    body="Use PageScript to express metrics, panels, and next actions in a compact source file."

    ::panel id=metrics title="Key metrics" density=compact
      ::metric id=activation label="Activation" value="72%" tone=good
      ::/metric
      ::metric id=pipeline label="Pipeline" value="$184k" tone=accent
      ::/metric
      ::metric id=risk label="At risk" value="6 accounts" tone=warning
      ::/metric
    ::/panel

    ::panel id=actions title="Next actions" density=spacious
      ::filter id=account-search label="Search accounts" placeholder="Company or owner"
      ::/filter
      ::stack gap=md
        ::card icon="A" title="Review account drift"
          body="Find accounts where usage dropped after onboarding and assign follow-up owners."
        ::/card
        ::card icon="B" title="Publish update"
          body="Render the latest dashboard and share the standalone HTML with peers."
        ::/card
      ::/stack
      ::table id=accounts
        ::column label="Account"
        ::/column
        ::column label="Status"
        ::/column
        ::row
          ::cell value="Acme"
          ::/cell
          ::cell value="Healthy"
          ::/cell
        ::/row
      ::/table
    ::/panel
  ::/scene
::/page
"##;

const DOCS_TEMPLATE: &str = r##"::page id=docs title="Project Docs"
  ::tokens
    color.accent="#116149"
    radius.panel=14
  ::/tokens

  ::hero spacing=lg
    heading="Project documentation"
    body="A compact PageScript docs page with sections, cards, and standalone rendered output."
  ::/hero

  ::section spacing=lg
    heading="Start here"
    body="Replace these cards with the workflow, API, and examples your peers need first."

    ::grid columns=3 gap=md density=compact
      ::card icon="01" title="Install"
        body="Document the shortest path from a fresh checkout to a working local command."
      ::/card
      ::card icon="02" title="Use"
        body="Show one realistic command sequence that creates a visible result."
      ::/card
      ::card icon="03" title="Extend"
        body="Explain how to customize the source file without learning the whole standard."
      ::/card
    ::/grid
  ::/section
::/page
"##;

#[cfg(test)]
mod tests {
    use super::run;
    use std::{
        env, fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn temp_path(name: &str) -> PathBuf {
        let suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        env::temp_dir().join(format!("pagescript-rs-{name}-{suffix}"))
    }

    #[test]
    fn validate_valid_file_returns_zero() {
        let path = temp_path("valid-minimal.tour");
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
    fn guide_returns_zero_without_file() {
        assert_eq!(run(vec!["guide".into()]), 0);
    }

    #[test]
    fn guide_rejects_file_argument() {
        assert_eq!(run(vec!["guide".into(), "demo.page".into()]), 1);
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
        let path = temp_path("page-selection.page");
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
        let path = temp_path("tour-selection.page");
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

    #[test]
    fn new_creates_default_product_template() {
        let path = temp_path("new-product.page");

        assert_eq!(run(vec!["new".into(), path.display().to_string()]), 0);
        let source = fs::read_to_string(path).unwrap();
        assert!(source.contains("::page id=product-demo"));
        assert!(source.contains("Launch a useful page from one file"));
    }

    #[test]
    fn new_creates_dashboard_template() {
        let path = temp_path("new-dashboard.page");

        assert_eq!(
            run(vec![
                "new".into(),
                path.display().to_string(),
                "--template".into(),
                "dashboard".into()
            ]),
            0
        );
        let source = fs::read_to_string(path).unwrap();
        assert!(source.contains("::page id=dashboard"));
        assert!(source.contains("Operating dashboard"));
    }

    #[test]
    fn new_refuses_overwrite_without_force() {
        let path = temp_path("existing.page");
        fs::write(&path, "keep me").unwrap();

        assert_eq!(run(vec!["new".into(), path.display().to_string()]), 1);
        assert_eq!(fs::read_to_string(path).unwrap(), "keep me");
    }

    #[test]
    fn new_force_overwrites_existing_file() {
        let path = temp_path("overwrite.page");
        fs::write(&path, "replace me").unwrap();

        assert_eq!(
            run(vec![
                "new".into(),
                path.display().to_string(),
                "--force".into()
            ]),
            0
        );
        let source = fs::read_to_string(path).unwrap();
        assert!(source.contains("::page id=product-demo"));
    }

    #[test]
    fn render_out_writes_html_file() {
        let source_path = temp_path("render-source.page");
        let out_path = temp_path("render-output.html");
        fs::write(
            &source_path,
            r##"::page id=actual
  ::hero heading="Rendered page" body="Written to disk"
  ::/hero
::/page
"##,
        )
        .unwrap();

        assert_eq!(
            run(vec![
                "render".into(),
                source_path.display().to_string(),
                "--out".into(),
                out_path.display().to_string()
            ]),
            0
        );
        let html = fs::read_to_string(out_path).unwrap();
        assert!(html.contains("Rendered page"));
    }

    #[test]
    fn render_without_out_preserves_stdout_success_path() {
        let source_path = temp_path("render-stdout.page");
        fs::write(
            &source_path,
            r##"::page id=actual
  ::hero heading="Stdout page" body="Rendered to stdout"
  ::/hero
::/page
"##,
        )
        .unwrap();

        assert_eq!(
            run(vec!["render".into(), source_path.display().to_string()]),
            0
        );
    }

    #[test]
    fn validate_json_returns_zero_for_valid_input() {
        let path = temp_path("valid-json.page");
        fs::write(
            &path,
            r##"::page id=actual
::/page
"##,
        )
        .unwrap();

        assert_eq!(
            run(vec![
                "validate".into(),
                path.display().to_string(),
                "--json".into()
            ]),
            0
        );
    }

    #[test]
    fn validate_json_returns_error_for_invalid_input() {
        let path = temp_path("invalid-json.page");
        fs::write(
            &path,
            r##"::page
::/page
"##,
        )
        .unwrap();

        assert_eq!(
            run(vec![
                "validate".into(),
                path.display().to_string(),
                "--json".into()
            ]),
            1
        );
    }
}
