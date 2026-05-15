//! Fixture-driven parser regression tests.
//!
//! Loads `tests/fixtures/parser/<locale>.json` files at runtime and feeds each
//! version sample into the corresponding parser. Add a locale by editing one
//! JSON file.
//!
//! See `tests/fixtures/README.md` for the JSON schema.

#![cfg(test)]

use std::path::{Path, PathBuf};

use serde::Deserialize;

use super::version::parse_version_output;
use crate::WslVersion;

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct LocaleParserFixture {
    locale: String,
    version: Vec<VersionCase>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionCase {
    name: String,
    source: String,
    output_lines: Vec<String>,
    expected: WslVersion,
}

#[test]
fn parser_locale_fixtures_match_expected_output() {
    for fixture in load_locale_fixtures() {
        assert_version_cases(&fixture);
    }
}

fn load_locale_fixtures() -> Vec<LocaleParserFixture> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/parser");
    let mut paths = std::fs::read_dir(&root)
        .unwrap_or_else(|err| panic!("failed to read fixture dir {}: {err}", root.display()))
        .map(|entry| {
            entry
                .unwrap_or_else(|err| panic!("failed to read fixture dir entry: {err}"))
                .path()
        })
        .filter(|path| path.extension().is_some_and(|ext| ext == "json"))
        .collect::<Vec<_>>();

    paths.sort();
    assert!(
        !paths.is_empty(),
        "expected at least one parser fixture JSON in {}",
        root.display()
    );

    paths.into_iter().map(load_locale_fixture).collect()
}

fn load_locale_fixture(path: PathBuf) -> LocaleParserFixture {
    let raw = std::fs::read_to_string(&path)
        .unwrap_or_else(|err| panic!("failed to load fixture {}: {err}", path.display()));
    let fixture = serde_json::from_str::<LocaleParserFixture>(&raw)
        .unwrap_or_else(|err| panic!("failed to parse fixture {}: {err}", path.display()));

    let stem = path
        .file_stem()
        .and_then(|value| value.to_str())
        .expect("fixture path should have UTF-8 file stem");
    assert_eq!(
        fixture.locale,
        stem,
        "fixture locale must match file name: {}",
        path.display()
    );
    assert!(
        !fixture.version.is_empty(),
        "fixture {} must contain at least one version parser case",
        path.display()
    );

    fixture
}

fn assert_version_cases(fixture: &LocaleParserFixture) {
    for case in &fixture.version {
        let id = case_id(&fixture.locale, "version", &case.name, &case.source);
        let actual = parse_version_output(&join_output(&case.output_lines))
            .unwrap_or_else(|err| panic!("{id}: {err}"));
        assert_eq!(actual, case.expected, "{id}");
    }
}

fn join_output(lines: &[String]) -> String {
    lines.join("\n")
}

fn case_id(locale: &str, command: &str, name: &str, source: &str) -> String {
    format!("parser fixture {locale}/{command}/{name} ({source})")
}
