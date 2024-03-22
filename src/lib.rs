#![no_std]

//! # psp-logger
//! A logger capable of outputting to the PSP's stdout and stderr.
//!
//! This output can than be viewed using [PSPLink](https://github.com/pspdev/psplinkusb)
//!
//! # Usage
//! ```
//! use psp_logger::{PspLogger, PspLoggerConfig, OutputStream};
//! use log::{trace, debug, info, warn, error};
//!
//! // Configure logging to only allow messages with debug-level or above.
//! // Map debug and info to stdout, letting other levels use the default stderr.
//! let config = PspLoggerConfig::new(log::LevelFilter::Debug)
//!     .with_debug_stream(OutputStream::StdOut)
//!     .with_info_stream(OutputStream::StdOut);
//!
//! let _ = psp_logger::PspLogger::init(config);
//!
//! trace!("This will be filtered out.");
//! debug!("This will be logged to stdout.");
//! info!("This will also be logged to stdout");
//! warn!("This will be logged to stderr.");
//! error!("This will also be logged to stder.");
//!
//! ```
extern crate alloc;

use core::fmt::Arguments;

use alloc::format;
use log::{Level, LevelFilter, Metadata, Record};
use psp::sys::*;

/// Enum holding the possible output streams that the logs can be written to.
#[derive(Copy, Clone)]
pub enum OutputStream {
    StdOut,
    StdErr,
}

/// Configuration for the logger.
///
/// # Examples
/// ```
/// // Create logger for Debug and up.
/// // All logs will be written to stderr.
/// let config = psp_logger::PspLoggerConfig::new(log::LevelFilter::Debug);
/// let _ = psp_logger::PspLogger::init(config);
///
/// debug!("I'm a debug log!");
/// ```
///
/// ```
/// // Create logger for Info and up.
/// // Info logs will go to stdout, the rest will go to stderr.
/// let config = psp_logger::PspLoggerConfig::new(log::LevelFilter::Info)
///     .with_info_stream(psp_logger::OutputStream::StdOut);
///
/// let _ = psp_logger::PspLogger::init(config);
///
/// info!("I'm an info log!");
/// ```
pub struct PspLoggerConfig {
    error_stream: OutputStream,
    warn_stream: OutputStream,
    info_stream: OutputStream,
    debug_stream: OutputStream,
    trace_stream: OutputStream,
    level_filter: LevelFilter,
}

/// The actual logger instance.
pub struct PspLogger {}

static LOGGER: PspLogger = PspLogger {};
static LOGGER_CONF: spin::Once<PspLoggerConfig> = spin::Once::new();

unsafe fn psp_write(stream: OutputStream, args: &Arguments) {
    let fh = match stream {
        OutputStream::StdErr => sceKernelStderr(),
        OutputStream::StdOut => sceKernelStdout(),
    };

    let msg = alloc::fmt::format(*args);
    let msg = format!("{}\n\0", msg);

    sceIoWrite(fh, msg.as_ptr() as _, msg.len());
}

impl log::Log for PspLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= LOGGER_CONF.get().unwrap().level_filter
    }

    fn log(&self, record: &Record) {
        let output = LOGGER_CONF
            .get()
            .unwrap()
            .get_stream(record.metadata().level());

        if self.enabled(record.metadata()) {
            unsafe { psp_write(output, record.args()) }
        }
    }

    fn flush(&self) {}
}

impl PspLogger {
    /// Initialise the logger.
    ///
    /// # Arguments
    /// - `config`: Logging configuration to be used.
    pub fn init(config: PspLoggerConfig) -> Result<(), log::SetLoggerError> {
        let level_filter = config.level_filter;

        LOGGER_CONF.call_once(|| config);
        log::set_logger(&LOGGER).map(|()| log::set_max_level(level_filter))
    }
}

impl PspLoggerConfig {
    /// Constructs a new PspLoggerConfig.
    ///
    /// All log levels will initially be mapped to stderr.
    /// Use the `with_*_stream` methods on the returned struct to change this.
    ///
    /// # Arguments
    /// - `level_filter`: Filter to control which log levels are actually logged.
    pub fn new(level_filter: LevelFilter) -> Self {
        PspLoggerConfig {
            error_stream: OutputStream::StdErr,
            warn_stream: OutputStream::StdErr,
            info_stream: OutputStream::StdErr,
            debug_stream: OutputStream::StdErr,
            trace_stream: OutputStream::StdErr,
            level_filter,
        }
    }

    /// Map the error log level to an [OutputStream]
    ///
    /// Returns the struct to allow the method to be chained.
    pub fn with_error_stream(mut self, stream: OutputStream) -> Self {
        self.error_stream = stream;
        self
    }

    /// Map the warn log level to an [OutputStream]
    ///
    /// Returns the struct to allow the method to be chained.
    pub fn with_warn_stream(mut self, stream: OutputStream) -> Self {
        self.warn_stream = stream;
        self
    }

    /// Map the info log level to an [OutputStream]
    ///
    /// Returns the struct to allow the method to be chained.
    pub fn with_info_stream(mut self, stream: OutputStream) -> Self {
        self.info_stream = stream;
        self
    }

    /// Map the debug log level to an [OutputStream]
    ///
    /// Returns the struct to allow the method to be chained.
    pub fn with_debug_stream(mut self, stream: OutputStream) -> Self {
        self.debug_stream = stream;
        self
    }

    /// Map the trace log level to an [OutputStream]
    ///
    /// Returns the struct to allow the method to be chained.
    pub fn with_trace_stream(mut self, stream: OutputStream) -> Self {
        self.trace_stream = stream;
        self
    }

    fn get_stream(&self, level: Level) -> OutputStream {
        match level {
            Level::Error => self.error_stream,
            Level::Warn => self.warn_stream,
            Level::Info => self.info_stream,
            Level::Debug => self.debug_stream,
            Level::Trace => self.trace_stream,
        }
    }
}
