# Majorsilence Reporting — Rust Wrapper

In-process Rust wrapper for the Majorsilence Reporting `rdlnative` shared library.

The reporting engine runs inside your process via FFI — no subprocess is spawned and no .NET runtime needs to be installed on the host.

| Platform | Library filename |
|---|---|
| Linux | `librdlnative.so` |
| macOS | `librdlnative.dylib` |
| Windows | `rdlnative.dll` |

**Rust 2021 edition** (Rust 1.65 or later recommended).

---

## Setup

### Install Rust

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustc --version
```

Or update an existing installation:

```bash
rustup update stable
```

### Add to your project

```toml
# Cargo.toml
[dependencies]
majorsilence-reporting = { path = "/path/to/reporting-rust" }
# or once published to crates.io:
# majorsilence-reporting = "1.0"
```

### Get the native library

Download the appropriate release package from the [Reporting releases](https://github.com/majorsilence/Reporting/releases):

- `majorsilence-reporting-rdlnative-linux.zip` → `linux-x64/` or `linux-arm64/`
- `majorsilence-reporting-rdlnative-osx.zip` → `osx-x64/` or `osx-arm64/`
- `majorsilence-reporting-rdlnative-windows.zip` → `win-x64/` or `win-arm64/`

Extract the zip — the directory contains the shared library and all its sibling runtime libraries.

On Linux, you may need to make the library directory visible to the dynamic linker:

```bash
export LD_LIBRARY_PATH=/path/to/rdlnative-dir:$LD_LIBRARY_PATH
```

---

## Usage

```rust
use majorsilence_reporting::RdlLibrary;

// Load once per process. On Linux/macOS, sibling .so/.dylib files are
// pre-loaded with RTLD_GLOBAL so .NET's P/Invoke resolver can find them.
let lib = RdlLibrary::load("/path/to/librdlnative.so")?;

let mut rpt = lib.report("/path/to/report.rdl");
rpt.set_connection_string("Data Source=/path/to/northwindEF.db");
rpt.set_parameter("Country", "Germany");

// Export to a file
rpt.export("pdf", "/tmp/output.pdf")?;

// Export to memory (Vec<u8>) — no temp file written
let bytes: Vec<u8> = rpt.export_to_memory("pdf")?;
```

### In-memory data (no database)

Supply rows directly, bypassing any database query:

```rust
use std::collections::HashMap;
use majorsilence_reporting::RdlLibrary;

let lib = RdlLibrary::load("/path/to/librdlnative.so")?;
let mut rpt = lib.report("/path/to/SalesReport.rdl");

rpt.add_data("Data", vec![
    HashMap::from([
        ("Product".into(), "Chai".into()),
        ("Amount".into(),  "1250.00".into()),
    ]),
    HashMap::from([
        ("Product".into(), "Chang".into()),
        ("Amount".into(),  "980.50".into()),
    ]),
]);

let bytes = rpt.export_to_memory("pdf")?;
```

`add_data` enables `SkipDatabaseSchemaValidation` automatically — no connection string is needed.

### Cloning the library handle

`RdlLibrary` is cheap to clone (`Arc`-backed) and can be shared across threads:

```rust
let lib = RdlLibrary::load("/path/to/librdlnative.so")?;
let lib2 = lib.clone();

std::thread::spawn(move || {
    let mut rpt = lib2.report("/path/to/report.rdl");
    // ...
});
```

---

## Supported export formats

| Format | Description |
|---|---|
| `pdf` | PDF (default if format is unrecognised) |
| `csv` | Comma-separated values |
| `xlsx` | Excel workbook |
| `xlsx_table` | Excel workbook (table style) |
| `xml` | XML |
| `rtf` | Rich Text Format |
| `tif` | TIFF image |
| `tifb` | TIFF image (black & white) |
| `html` | HTML |
| `mht` | MHTML |

---

## Running the tests

The integration tests require a published `rdlnative` shared library and the sample RDL/database files from the main Reporting repository.

```bash
# Build the native library first (from the main Reporting repo)
dotnet publish RdlNative -c Release-DrawingCompat -r linux-x64 -f net10.0 \
    --self-contained true -p:PublishAot=true \
    -o /tmp/rdlnative-pub

# Run tests
RDLNATIVE_LIB=/tmp/rdlnative-pub/librdlnative.so \
REPORTING_REPO_ROOT=/path/to/Reporting \
    cargo test
```

On macOS replace `linux-x64` with `osx-arm64` (or `osx-x64`) and `librdlnative.so` with `librdlnative.dylib`.

Tests are skipped gracefully when `RDLNATIVE_LIB` or `REPORTING_REPO_ROOT` are not set.

---

## Running the examples

```bash
RDLNATIVE_LIB=/tmp/rdlnative-pub/librdlnative.so \
REPORTING_REPO_ROOT=/path/to/Reporting \
    cargo run --example test1

cargo run --example test2_parameters
cargo run --example test3_streaming
```

Examples:

- `test1` — basic PDF export to a file
- `test2_parameters` — passing report parameters
- `test3_streaming` — `export_to_memory` for in-memory bytes, plus `add_data`

---

## License

Available under your choice of MIT, Apache 2.0, or BSD 2-Clause — see `LICENSE-MIT`, `LICENSE-APACHE`, `LICENSE-BSD`.
