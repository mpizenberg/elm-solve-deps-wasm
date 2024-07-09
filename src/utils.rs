// SPDX-License-Identifier: MPL-2.0

use log::{LevelFilter, Metadata, Record, SetLoggerError};
use wasm_bindgen::prelude::*;

pub fn set_panic_hook() {
    // When the `console_error_panic_hook` feature is enabled, we can call the
    // `set_panic_hook` function at least once during initialization, and then
    // we will get better error messages if our code ever panics.
    //
    // For more details see
    // https://github.com/rustwasm/console_error_panic_hook#readme
    #[cfg(feature = "console_error_panic_hook")]
    console_error_panic_hook::set_once();
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_namespace = console)]
    pub fn log(s: &str);
}

// Macro console_log! similar to println!
// macro_rules! console_log {
//     ($($t:tt)*) => (crate::utils::log(&format_args!($($t)*).to_string()))
// }

// Log implementation

pub struct WasmLogger;

static LOGGER: WasmLogger = WasmLogger;

impl WasmLogger {
    pub fn init() -> Result<(), SetLoggerError> {
        log::set_logger(&LOGGER)
    }
    pub fn setup(max_level: LevelFilter) {
        log::set_max_level(max_level);
    }
}

impl log::Log for WasmLogger {
    fn enabled(&self, _metadata: &Metadata) -> bool {
        true
    }

    // TODO: instead of silencing logs, we should call a js_sys::Function
    // passed as argument when initializing the wasm logger.
    // WasmLogger will not be able to stay static if we do that.
    // In turn this means we'll need a struct in lib.rs holding a Box<WasmLogger>.
    fn log(&self, _record: &Record) {
        // console_log!("{}: {}", record.level(), record.args());
    }

    fn flush(&self) {}
}

pub fn verbosity_filter(verbosity: u32) -> LevelFilter {
    match verbosity {
        0 => LevelFilter::Error,
        1 => LevelFilter::Warn,
        2 => LevelFilter::Info,
        3 => LevelFilter::Debug,
        _ => LevelFilter::Trace,
    }
}

/// Log the error and convert it into a `JsValue`.
pub fn report_error<E: Into<anyhow::Error>>(error: E) -> JsValue {
    let error_msg = format!("{:?}", error.into());
    log::error!("{}", &error_msg);
    error_msg.into()
}
