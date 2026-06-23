//! Export to memory (Vec<u8>) — useful for HTTP streaming responses.
//!
//! Usage:
//!   RDLNATIVE_LIB=/path/to/librdlnative.so \
//!   REPORTING_REPO_ROOT=/path/to/Reporting \
//!       cargo run --example test3_streaming

use majorsilence_reporting::RdlLibrary;
use std::{collections::HashMap, env, path::PathBuf};

fn main() {
    let lib_path = env::var("RDLNATIVE_LIB")
        .expect("Set RDLNATIVE_LIB to the path of the rdlnative shared library");
    let root = PathBuf::from(
        env::var("REPORTING_REPO_ROOT")
            .expect("Set REPORTING_REPO_ROOT to the path of the Reporting repository"),
    );

    let lib = RdlLibrary::load(&lib_path).expect("failed to load rdlnative");

    // ── Example 1: export_to_memory from a database-backed report ────────────
    {
        let rdl = root.join("Examples/SqliteExamples/SimpleTest1.rdl");
        let db  = root.join("Examples/northwindEF.db");

        let mut rpt = lib.report(&rdl);
        rpt.set_connection_string(format!("Data Source={}", db.display()));

        let bytes = rpt.export_to_memory("pdf").expect("export_to_memory failed");
        println!("PDF from database: {} bytes, starts with {:?}", bytes.len(), &bytes[..4]);
    }

    // ── Example 2: export_to_memory from in-memory data (no database) ────────
    {
        let rdl = root.join("Examples/SetDataFromCode/SalesReport.rdl");

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
        ]);

        let bytes = rpt.export_to_memory("pdf").expect("add_data export failed");
        println!("PDF from in-memory data: {} bytes", bytes.len());
    }
}
