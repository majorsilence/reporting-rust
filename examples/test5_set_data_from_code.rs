//! Set data from code — feed rows directly into a report with no database at all.
//!
//! Key patterns shown:
//!   - add_data() injects rows from any Rust data source
//!   - HashMap keys must exactly match the <Field Name="..."> values in the RDL
//!   - No connection string is needed — SkipDatabaseSchemaValidation is set automatically
//!
//! Usage:
//!   RDLNATIVE_LIB=/path/to/librdlnative.so \
//!   REPORTING_REPO_ROOT=/path/to/Reporting \
//!       cargo run --example test5_set_data_from_code

use majorsilence_reporting::RdlLibrary;
use std::{collections::HashMap, env, path::PathBuf};

fn main() {
    let lib_path = env::var("RDLNATIVE_LIB")
        .expect("Set RDLNATIVE_LIB to the path of the rdlnative shared library");
    let root = PathBuf::from(
        env::var("REPORTING_REPO_ROOT")
            .expect("Set REPORTING_REPO_ROOT to the path of the Reporting repository"),
    );

    let rdl = root.join("Examples/SetDataFromCode/SalesReport.rdl");

    let lib = RdlLibrary::load(&lib_path).expect("failed to load rdlnative");

    // Keys must match <Field Name="..."> in SalesReport.rdl exactly.
    let sales_data: Vec<HashMap<String, String>> = vec![
        HashMap::from([("Product".into(), "Chai".into()),                   ("Region".into(), "North America".into()), ("Amount".into(), "1250.00".into()), ("Quantity".into(), "50".into())]),
        HashMap::from([("Product".into(), "Chang".into()),                  ("Region".into(), "North America".into()), ("Amount".into(),  "980.50".into()), ("Quantity".into(), "42".into())]),
        HashMap::from([("Product".into(), "Aniseed Syrup".into()),          ("Region".into(), "Europe".into()),        ("Amount".into(),  "432.00".into()), ("Quantity".into(), "24".into())]),
        HashMap::from([("Product".into(), "Chef Anton's Cajun".into()),     ("Region".into(), "Europe".into()),        ("Amount".into(), "1875.25".into()), ("Quantity".into(), "75".into())]),
        HashMap::from([("Product".into(), "Grandma's Boysenberry".into()),  ("Region".into(), "Asia Pacific".into()),  ("Amount".into(),  "640.00".into()), ("Quantity".into(), "32".into())]),
        HashMap::from([("Product".into(), "Uncle Bob's Organic".into()),    ("Region".into(), "North America".into()), ("Amount".into(),  "315.60".into()), ("Quantity".into(), "18".into())]),
        HashMap::from([("Product".into(), "Northwoods Cranberry".into()),   ("Region".into(), "North America".into()), ("Amount".into(),  "560.00".into()), ("Quantity".into(), "20".into())]),
        HashMap::from([("Product".into(), "Mishi Kobe Niku".into()),        ("Region".into(), "Asia Pacific".into()),  ("Amount".into(), "4500.00".into()), ("Quantity".into(), "30".into())]),
        HashMap::from([("Product".into(), "Ikura".into()),                  ("Region".into(), "Asia Pacific".into()),  ("Amount".into(), "1980.00".into()), ("Quantity".into(), "36".into())]),
        HashMap::from([("Product".into(), "Queso Cabrales".into()),         ("Region".into(), "Europe".into()),        ("Amount".into(),  "850.00".into()), ("Quantity".into(), "25".into())]),
        HashMap::from([("Product".into(), "Queso Manchego La".into()),      ("Region".into(), "Europe".into()),        ("Amount".into(),  "720.00".into()), ("Quantity".into(), "30".into())]),
        HashMap::from([("Product".into(), "Konbu".into()),                  ("Region".into(), "Asia Pacific".into()),  ("Amount".into(),  "180.00".into()), ("Quantity".into(), "24".into())]),
        HashMap::from([("Product".into(), "Tofu".into()),                   ("Region".into(), "Asia Pacific".into()),  ("Amount".into(),  "560.00".into()), ("Quantity".into(), "40".into())]),
        HashMap::from([("Product".into(), "Genen Shouyu".into()),           ("Region".into(), "Asia Pacific".into()),  ("Amount".into(),  "310.00".into()), ("Quantity".into(), "26".into())]),
        HashMap::from([("Product".into(), "Pavlova".into()),                ("Region".into(), "Asia Pacific".into()),  ("Amount".into(),  "825.00".into()), ("Quantity".into(), "55".into())]),
        HashMap::from([("Product".into(), "Alice Mutton".into()),           ("Region".into(), "Europe".into()),        ("Amount".into(), "2340.00".into()), ("Quantity".into(), "26".into())]),
        HashMap::from([("Product".into(), "Carnarvon Tigers".into()),       ("Region".into(), "Asia Pacific".into()),  ("Amount".into(), "6200.00".into()), ("Quantity".into(), "31".into())]),
        HashMap::from([("Product".into(), "Teatime Biscuits".into()),       ("Region".into(), "Europe".into()),        ("Amount".into(),  "291.60".into()), ("Quantity".into(), "36".into())]),
        HashMap::from([("Product".into(), "Sir Rodney's Marmalade".into()), ("Region".into(), "Europe".into()),        ("Amount".into(), "1245.00".into()), ("Quantity".into(), "45".into())]),
        HashMap::from([("Product".into(), "Sir Rodney's Scones".into()),    ("Region".into(), "Europe".into()),        ("Amount".into(),  "350.00".into()), ("Quantity".into(), "50".into())]),
    ];

    let mut rpt = lib.report(&rdl);
    rpt.add_data("Data", sales_data);

    let out = "/tmp/sales-report.pdf";
    rpt.export("pdf", out).expect("export failed");
    println!("Written: {out}");
}
