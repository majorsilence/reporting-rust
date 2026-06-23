//! Export multiple formats — render Orders.rdl to PDF, Excel, CSV, and HTML.
//!
//! Usage:
//!   RDLNATIVE_LIB=/path/to/librdlnative.so \
//!   REPORTING_REPO_ROOT=/path/to/Reporting \
//!       cargo run --example test4_export_multiple_formats

use majorsilence_reporting::RdlLibrary;
use std::{env, path::PathBuf};

fn main() {
    let lib_path = env::var("RDLNATIVE_LIB")
        .expect("Set RDLNATIVE_LIB to the path of the rdlnative shared library");
    let root = PathBuf::from(
        env::var("REPORTING_REPO_ROOT")
            .expect("Set REPORTING_REPO_ROOT to the path of the Reporting repository"),
    );

    let rdl = root.join("Examples/ExportMultipleFormats/Orders.rdl");
    let db  = root.join("Examples/ExportMultipleFormats/sqlitetestdb2.db");

    let lib = RdlLibrary::load(&lib_path).expect("failed to load rdlnative");

    let mut rpt = lib.report(&rdl);
    rpt.set_connection_string(format!("Data Source={}", db.display()));

    let formats = [
        ("/tmp/orders.pdf",  "pdf"),
        ("/tmp/orders.xlsx", "xlsx"),
        ("/tmp/orders.csv",  "csv"),
        ("/tmp/orders.html", "html"),
    ];

    for (out, fmt) in formats {
        rpt.export(fmt, out).expect("export failed");
        println!("Written: {out}");
    }
}
