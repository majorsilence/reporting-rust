//! Rust wrapper for the [`rdlnative`](https://github.com/majorsilence/Reporting) shared library.
//!
//! The reporting engine runs in-process via FFI — no subprocess is spawned and
//! no .NET runtime needs to be installed on the host.
//!
//! # Quick start
//!
//! ```no_run
//! use majorsilence_reporting::RdlLibrary;
//!
//! let lib = RdlLibrary::load("/path/to/librdlnative.so").unwrap();
//!
//! let mut rpt = lib.report("/path/to/report.rdl");
//! rpt.set_connection_string("Data Source=/path/to/db.sqlite");
//! rpt.set_parameter("Country", "Germany");
//!
//! // Export to a file
//! rpt.export("pdf", "/tmp/output.pdf").unwrap();
//!
//! // Export to memory (no temp file)
//! let bytes: Vec<u8> = rpt.export_to_memory("pdf").unwrap();
//! ```

use std::{
    collections::HashMap,
    ffi::{CStr, CString},
    os::raw::{c_char, c_int, c_void},
    path::{Path, PathBuf},
    sync::Arc,
};

/// Supported export format strings.
pub const VALID_FORMATS: &[&str] = &[
    "pdf", "csv", "xlsx", "xlsx_table", "xml", "rtf", "tif", "tifb", "html", "mht",
];

// ─── Error type ───────────────────────────────────────────────────────────────

/// Error returned by this library.
#[derive(Debug)]
pub struct Error(String);

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl std::error::Error for Error {}

impl From<libloading::Error> for Error {
    fn from(e: libloading::Error) -> Self {
        Error(e.to_string())
    }
}

// ─── Raw FFI symbol table ─────────────────────────────────────────────────────

struct Symbols {
    // Keep the library alive for the lifetime of these function pointers.
    _lib: libloading::Library,

    rdl_init:                 unsafe extern "C" fn() -> c_int,
    rdl_report_open:          unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void,
    rdl_report_set_param:     unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int,
    rdl_dataset_set_field:    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, *const c_char) -> c_int,
    rdl_dataset_commit_row:   unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int,
    rdl_report_render_file:   unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int,
    rdl_report_render_buffer: unsafe extern "C" fn(*mut c_void, *const c_char, *mut *mut u8, *mut c_int) -> c_int,
    rdl_free:                 unsafe extern "C" fn(*mut c_void),
    rdl_report_close:         unsafe extern "C" fn(*mut c_void),
    rdl_last_error:           unsafe extern "C" fn() -> *const c_char,
}

// SAFETY: libloading::Library and fn pointers are Send + Sync.
unsafe impl Send for Symbols {}
unsafe impl Sync for Symbols {}

impl Symbols {
    unsafe fn load(path: &Path) -> Result<Self, Error> {
        let lib = libloading::Library::new(path)?;

        macro_rules! sym {
            ($name:literal, $ty:ty) => {{
                let s: libloading::Symbol<$ty> = lib
                    .get($name)
                    .map_err(|e| Error(format!("missing symbol {}: {e}", stringify!($name))))?;
                // Copy the fn pointer out — safe as long as `lib` stays alive (_lib field).
                *s
            }};
        }

        let syms = Symbols {
            rdl_init:                 sym!(b"rdl_init\0",                 unsafe extern "C" fn() -> c_int),
            rdl_report_open:          sym!(b"rdl_report_open\0",          unsafe extern "C" fn(*const c_char, *const c_char) -> *mut c_void),
            rdl_report_set_param:     sym!(b"rdl_report_set_param\0",     unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int),
            rdl_dataset_set_field:    sym!(b"rdl_dataset_set_field\0",    unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char, *const c_char) -> c_int),
            rdl_dataset_commit_row:   sym!(b"rdl_dataset_commit_row\0",   unsafe extern "C" fn(*mut c_void, *const c_char) -> c_int),
            rdl_report_render_file:   sym!(b"rdl_report_render_file\0",   unsafe extern "C" fn(*mut c_void, *const c_char, *const c_char) -> c_int),
            rdl_report_render_buffer: sym!(b"rdl_report_render_buffer\0", unsafe extern "C" fn(*mut c_void, *const c_char, *mut *mut u8, *mut c_int) -> c_int),
            rdl_free:                 sym!(b"rdl_free\0",                 unsafe extern "C" fn(*mut c_void)),
            rdl_report_close:         sym!(b"rdl_report_close\0",         unsafe extern "C" fn(*mut c_void)),
            rdl_last_error:           sym!(b"rdl_last_error\0",           unsafe extern "C" fn() -> *const c_char),
            _lib: lib,
        };
        Ok(syms)
    }

    fn last_error(&self) -> String {
        unsafe {
            let ptr = (self.rdl_last_error)();
            if ptr.is_null() {
                return "unknown error".into();
            }
            CStr::from_ptr(ptr).to_string_lossy().into_owned()
        }
    }
}

// ─── Public API ───────────────────────────────────────────────────────────────

/// A loaded instance of the rdlnative shared library.
///
/// Create one per process with [`RdlLibrary::load`], then derive [`Report`]
/// instances from it.  The underlying library stays loaded as long as any
/// `RdlLibrary` clone or any `Report` derived from it exists (backed by `Arc`).
#[derive(Clone)]
pub struct RdlLibrary(Arc<Symbols>);

impl RdlLibrary {
    /// Load the rdlnative shared library from `path` and initialize the engine.
    ///
    /// On Linux and macOS, all sibling shared libraries in the same directory
    /// are pre-loaded with `RTLD_GLOBAL` so that .NET's P/Invoke resolver can
    /// locate them at render time.
    ///
    /// Call this once at process startup.
    pub fn load(path: impl AsRef<Path>) -> Result<Self, Error> {
        let path = path.as_ref().canonicalize()
            .map_err(|e| Error(format!("cannot resolve library path: {e}")))?;
        let dir = path.parent().unwrap_or(Path::new("."));

        // Tell rdl_init's DllImportResolver where sibling native libs live.
        std::env::set_var("RDLNATIVE_LIB_DIR", dir);

        #[cfg(unix)]
        Self::preload_siblings(dir);

        let syms = unsafe { Symbols::load(&path)? };

        let ret = unsafe { (syms.rdl_init)() };
        if ret != 0 {
            let msg = syms.last_error();
            return Err(Error(format!("rdl_init failed: {msg}")));
        }

        Ok(Self(Arc::new(syms)))
    }

    /// Create a [`Report`] configuration for the given `.rdl` file path.
    pub fn report(&self, rdl_path: impl AsRef<Path>) -> Report {
        Report {
            lib:               Arc::clone(&self.0),
            rdl_path:          rdl_path.as_ref().to_owned(),
            connection_string: None,
            parameters:        HashMap::new(),
            datasets:          HashMap::new(),
        }
    }

    #[cfg(unix)]
    fn preload_siblings(dir: &Path) {
        use std::ffi::OsStr;
        #[cfg(target_os = "macos")]
        let ext: &OsStr = OsStr::new("dylib");
        #[cfg(not(target_os = "macos"))]
        let ext: &OsStr = OsStr::new("so");

        let entries = match std::fs::read_dir(dir) {
            Ok(e) => e,
            Err(_) => return,
        };
        let mut paths: Vec<_> = entries
            .filter_map(|e| e.ok().map(|e| e.path()))
            .filter(|p| p.extension() == Some(ext))
            .collect();
        paths.sort();
        for p in paths {
            unsafe {
                let _ = libloading::os::unix::Library::open(
                    Some(&p),
                    libloading::os::unix::RTLD_LAZY | libloading::os::unix::RTLD_GLOBAL,
                );
            }
        }
    }
}

/// Report configuration: RDL path, connection string, parameters, and optional
/// in-memory dataset rows.
///
/// Create instances with [`RdlLibrary::report`].  A new native report handle is
/// opened for each call to [`Report::export`] or [`Report::export_to_memory`]
/// and closed immediately afterwards.
pub struct Report {
    lib:               Arc<Symbols>,
    rdl_path:          PathBuf,
    connection_string: Option<String>,
    parameters:        HashMap<String, String>,
    datasets:          HashMap<String, Vec<HashMap<String, String>>>,
}

impl Report {
    /// Override the connection string embedded in the RDL file.
    pub fn set_connection_string(&mut self, cs: impl Into<String>) -> &mut Self {
        self.connection_string = Some(cs.into());
        self
    }

    /// Set a named report parameter.
    pub fn set_parameter(&mut self, name: impl Into<String>, value: impl Into<String>) -> &mut Self {
        self.parameters.insert(name.into(), value.into());
        self
    }

    /// Supply in-memory rows for a named dataset, bypassing any database query.
    ///
    /// `dataset_name` must match the `<DataSet Name="...">` attribute in the
    /// RDL file.  Map keys must match `<Field Name="...">` elements.
    ///
    /// `SkipDatabaseSchemaValidation` is enabled automatically when datasets are
    /// present — no database connection is needed.
    ///
    /// ```no_run
    /// # let lib = majorsilence_reporting::RdlLibrary::load("/tmp/librdlnative.so").unwrap();
    /// # let mut rpt = lib.report("/tmp/report.rdl");
    /// use std::collections::HashMap;
    ///
    /// rpt.add_data("SalesData", vec![
    ///     HashMap::from([
    ///         ("Product".into(), "Chai".into()),
    ///         ("Amount".into(),  "1250.00".into()),
    ///     ]),
    ///     HashMap::from([
    ///         ("Product".into(), "Chang".into()),
    ///         ("Amount".into(),  "980.50".into()),
    ///     ]),
    /// ]);
    /// ```
    pub fn add_data(
        &mut self,
        dataset_name: impl Into<String>,
        rows: Vec<HashMap<String, String>>,
    ) -> &mut Self {
        self.datasets.insert(dataset_name.into(), rows);
        self
    }

    /// Render the report and write the output to `output_path`.
    ///
    /// `format` defaults to `"pdf"` if the string is not a recognised value.
    /// See [`VALID_FORMATS`] for the full list.
    pub fn export(&self, format: &str, output_path: impl AsRef<Path>) -> Result<(), Error> {
        let fmt    = validate_format(format);
        let c_fmt  = cstring(fmt)?;
        let c_path = cstring(&output_path.as_ref().to_string_lossy())?;

        let h = self.open_handle()?;
        let ret = unsafe { (self.lib.rdl_report_render_file)(h, c_path.as_ptr(), c_fmt.as_ptr()) };
        unsafe { (self.lib.rdl_report_close)(h) };
        if ret != 0 {
            return Err(Error(format!("rdl_report_render_file failed: {}", self.lib.last_error())));
        }
        Ok(())
    }

    /// Render the report and return the raw output as bytes.
    ///
    /// No temporary file is written; the data comes directly from the native
    /// library's in-memory buffer.  `format` defaults to `"pdf"` if the string
    /// is not a recognised value.
    pub fn export_to_memory(&self, format: &str) -> Result<Vec<u8>, Error> {
        let fmt   = validate_format(format);
        let c_fmt = cstring(fmt)?;

        let h = self.open_handle()?;
        let mut buf:  *mut u8 = std::ptr::null_mut();
        let mut size: c_int   = 0;
        let ret = unsafe {
            (self.lib.rdl_report_render_buffer)(h, c_fmt.as_ptr(), &mut buf, &mut size)
        };
        unsafe { (self.lib.rdl_report_close)(h) };
        if ret != 0 {
            return Err(Error(format!("rdl_report_render_buffer failed: {}", self.lib.last_error())));
        }
        // SAFETY: buf and size are set by rdl_report_render_buffer on success.
        let bytes = unsafe { std::slice::from_raw_parts(buf, size as usize).to_vec() };
        unsafe { (self.lib.rdl_free)(buf.cast::<c_void>()) };
        Ok(bytes)
    }

    // ─── Private helpers ──────────────────────────────────────────────────

    /// Open a native report handle with all parameters and dataset rows applied.
    ///
    /// All C strings are pre-allocated before the handle is opened so that any
    /// conversion failure returns an error before touching native resources.
    fn open_handle(&self) -> Result<*mut c_void, Error> {
        // Pre-convert everything to CStrings so we fail before opening the handle.
        let c_rdl = cstring(&self.rdl_path.to_string_lossy())?;
        let c_cs: Option<CString> = self.connection_string
            .as_deref()
            .map(cstring)
            .transpose()?;

        let c_params: Vec<(CString, CString)> = self.parameters
            .iter()
            .map(|(k, v)| Ok((cstring(k)?, cstring(v)?)))
            .collect::<Result<_, Error>>()?;

        let c_datasets: Vec<(CString, Vec<Vec<(CString, CString)>>)> = self.datasets
            .iter()
            .map(|(ds, rows)| {
                let c_ds = cstring(ds)?;
                let c_rows: Vec<Vec<(CString, CString)>> = rows
                    .iter()
                    .map(|row| {
                        row.iter()
                            .map(|(f, v)| Ok((cstring(f)?, cstring(v)?)))
                            .collect::<Result<_, Error>>()
                    })
                    .collect::<Result<_, Error>>()?;
                Ok((c_ds, c_rows))
            })
            .collect::<Result<_, Error>>()?;

        // Open the native handle.
        let cs_ptr = c_cs.as_ref().map_or(std::ptr::null(), CString::as_ptr);
        let h = unsafe { (self.lib.rdl_report_open)(c_rdl.as_ptr(), cs_ptr) };
        if h.is_null() {
            return Err(Error(format!("rdl_report_open failed: {}", self.lib.last_error())));
        }

        // Apply parameters.
        for (c_name, c_val) in &c_params {
            let ret = unsafe {
                (self.lib.rdl_report_set_param)(h, c_name.as_ptr(), c_val.as_ptr())
            };
            if ret != 0 {
                unsafe { (self.lib.rdl_report_close)(h) };
                return Err(Error(format!("rdl_report_set_param failed: {}", self.lib.last_error())));
            }
        }

        // Inject in-memory dataset rows.
        for (c_ds, c_rows) in &c_datasets {
            for c_row in c_rows {
                for (c_field, c_val) in c_row {
                    let ret = unsafe {
                        (self.lib.rdl_dataset_set_field)(
                            h, c_ds.as_ptr(), c_field.as_ptr(), c_val.as_ptr(),
                        )
                    };
                    if ret != 0 {
                        unsafe { (self.lib.rdl_report_close)(h) };
                        return Err(Error(format!(
                            "rdl_dataset_set_field failed: {}", self.lib.last_error()
                        )));
                    }
                }
                let ret = unsafe { (self.lib.rdl_dataset_commit_row)(h, c_ds.as_ptr()) };
                if ret != 0 {
                    unsafe { (self.lib.rdl_report_close)(h) };
                    return Err(Error(format!(
                        "rdl_dataset_commit_row failed: {}", self.lib.last_error()
                    )));
                }
            }
        }

        Ok(h)
    }
}

// ─── Utilities ────────────────────────────────────────────────────────────────

fn validate_format(fmt: &str) -> &str {
    if VALID_FORMATS.contains(&fmt) { fmt } else { "pdf" }
}

fn cstring(s: &str) -> Result<CString, Error> {
    CString::new(s).map_err(|e| Error(format!("string contains interior NUL byte: {e}")))
}
