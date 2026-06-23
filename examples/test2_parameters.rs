//! Export with report parameters.
//!
//! Usage:
//!   RDLNATIVE_LIB=/path/to/librdlnative.so \
//!   REPORTING_REPO_ROOT=/path/to/Reporting \
//!       cargo run --example test2_parameters

use majorsilence_reporting::RdlLibrary;
use std::{env, path::PathBuf};

fn main() {
    let lib_path = env::var("RDLNATIVE_LIB")
        .expect("Set RDLNATIVE_LIB to the path of the rdlnative shared library");
    let root = PathBuf::from(
        env::var("REPORTING_REPO_ROOT")
            .expect("Set REPORTING_REPO_ROOT to the path of the Reporting repository"),
    );

    let rdl = root.join("Examples/SqliteExamples/SimpleTest1.rdl");
    let db  = root.join("Examples/northwindEF.db");

    let lib = RdlLibrary::load(&lib_path).expect("failed to load rdlnative");

    let mut rpt = lib.report(&rdl);
    rpt.set_connection_string(format!("Data Source={}", db.display()));
    rpt.set_parameter("Country", "Germany");

    let out = "/tmp/test2_output.pdf";
    rpt.export("pdf", out).expect("export failed");
    println!("Written to {out}");
}
