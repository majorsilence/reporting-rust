//! Integration tests for the rdlnative Rust wrapper.
//!
//! Requires a published rdlnative shared library and the sample RDL/database
//! files from the main Reporting repository.  Set two environment variables:
//!
//!   RDLNATIVE_LIB        — path to the shared library (librdlnative.so / .dylib / .dll)
//!   REPORTING_REPO_ROOT  — path to the cloned Reporting repository
//!
//! If either variable is absent, all tests are skipped gracefully.
//!
//! Example (Linux):
//!   dotnet publish RdlNative -c Release-DrawingCompat -r linux-x64 -f net10.0 \
//!       --self-contained true -p:PublishAot=true -o /tmp/rdlnative-pub
//!
//!   RDLNATIVE_LIB=/tmp/rdlnative-pub/librdlnative.so \
//!   REPORTING_REPO_ROOT=/path/to/Reporting \
//!       cargo test

use std::{collections::HashMap, env, path::PathBuf};
use majorsilence_reporting::RdlLibrary;

fn env_paths() -> Option<(String, PathBuf)> {
    let lib  = env::var("RDLNATIVE_LIB").ok()?;
    let root = env::var("REPORTING_REPO_ROOT").ok()?;
    Some((lib, PathBuf::from(root)))
}

fn load_lib(lib_path: &str) -> RdlLibrary {
    RdlLibrary::load(lib_path).expect("failed to load rdlnative library")
}

#[test]
fn test_export_to_file_pdf() {
    let Some((lib_path, root)) = env_paths() else {
        eprintln!("skipping: RDLNATIVE_LIB / REPORTING_REPO_ROOT not set");
        return;
    };
    let rdl = root.join("Examples/SqliteExamples/SimpleTest1.rdl");
    let db  = root.join("Examples/northwindEF.db");
    if !rdl.exists() || !db.exists() {
        eprintln!("skipping: sample files not found under REPORTING_REPO_ROOT");
        return;
    }

    let lib = load_lib(&lib_path);
    let mut rpt = lib.report(&rdl);
    rpt.set_connection_string(format!("Data Source={}", db.display()));

    let out = std::env::temp_dir().join("rdlnative_rust_test.pdf");
    rpt.export("pdf", &out).expect("export failed");
    let data = std::fs::read(&out).expect("output not written");
    std::fs::remove_file(&out).ok();

    assert!(data.starts_with(b"%PDF"), "expected PDF header");
}

#[test]
fn test_export_to_memory_pdf() {
    let Some((lib_path, root)) = env_paths() else {
        eprintln!("skipping: RDLNATIVE_LIB / REPORTING_REPO_ROOT not set");
        return;
    };
    let rdl = root.join("Examples/SqliteExamples/SimpleTest1.rdl");
    let db  = root.join("Examples/northwindEF.db");
    if !rdl.exists() || !db.exists() {
        eprintln!("skipping: sample files not found under REPORTING_REPO_ROOT");
        return;
    }

    let lib = load_lib(&lib_path);
    let mut rpt = lib.report(&rdl);
    rpt.set_connection_string(format!("Data Source={}", db.display()));

    let bytes = rpt.export_to_memory("pdf").expect("export_to_memory failed");
    assert!(bytes.starts_with(b"%PDF"), "expected PDF header");
    assert!(bytes.len() > 100);
}

#[test]
fn test_export_to_memory_csv() {
    let Some((lib_path, root)) = env_paths() else {
        eprintln!("skipping: RDLNATIVE_LIB / REPORTING_REPO_ROOT not set");
        return;
    };
    let rdl = root.join("Examples/SqliteExamples/SimpleTest1.rdl");
    let db  = root.join("Examples/northwindEF.db");
    if !rdl.exists() || !db.exists() {
        eprintln!("skipping: sample files not found under REPORTING_REPO_ROOT");
        return;
    }

    let lib = load_lib(&lib_path);
    let mut rpt = lib.report(&rdl);
    rpt.set_connection_string(format!("Data Source={}", db.display()));

    let bytes = rpt.export_to_memory("csv").expect("export_to_memory csv failed");
    assert!(!bytes.is_empty());
}

#[test]
fn test_add_data() {
    let Some((lib_path, root)) = env_paths() else {
        eprintln!("skipping: RDLNATIVE_LIB / REPORTING_REPO_ROOT not set");
        return;
    };
    let rdl = root.join("Examples/SetDataFromCode/SalesReport.rdl");
    if !rdl.exists() {
        eprintln!("skipping: SalesReport.rdl not found under REPORTING_REPO_ROOT");
        return;
    }

    let lib = load_lib(&lib_path);
    let mut rpt = lib.report(&rdl);
    rpt.add_data("Data", vec![
        HashMap::from([
            ("Product".into(),  "Chai".into()),
            ("Region".into(),   "North America".into()),
            ("Amount".into(),   "1250.00".into()),
            ("Quantity".into(), "50".into()),
        ]),
        HashMap::from([
            ("Product".into(),  "Chang".into()),
            ("Region".into(),   "Europe".into()),
            ("Amount".into(),   "980.50".into()),
            ("Quantity".into(), "42".into()),
        ]),
        HashMap::from([
            ("Product".into(),  "Tofu".into()),
            ("Region".into(),   "Asia Pacific".into()),
            ("Amount".into(),   "560.00".into()),
            ("Quantity".into(), "40".into()),
        ]),
    ]);

    let bytes = rpt.export_to_memory("pdf").expect("add_data export failed");
    assert!(bytes.starts_with(b"%PDF"), "expected PDF header");
}

#[test]
fn test_invalid_format_defaults_to_pdf() {
    let Some((lib_path, root)) = env_paths() else {
        eprintln!("skipping: RDLNATIVE_LIB / REPORTING_REPO_ROOT not set");
        return;
    };
    let rdl = root.join("Examples/SqliteExamples/SimpleTest1.rdl");
    let db  = root.join("Examples/northwindEF.db");
    if !rdl.exists() || !db.exists() {
        eprintln!("skipping: sample files not found under REPORTING_REPO_ROOT");
        return;
    }

    let lib = load_lib(&lib_path);
    let mut rpt = lib.report(&rdl);
    rpt.set_connection_string(format!("Data Source={}", db.display()));

    let bytes = rpt.export_to_memory("bogus_format").expect("export with invalid format failed");
    assert!(bytes.starts_with(b"%PDF"), "expected fallback to PDF");
}
