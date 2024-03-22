# psp-logger
A logger capable of outputting to the PSP's stdout and stderr.

This output can than be viewed using [PSPLink](https://github.com/pspdev/psplinkusb)

# Usage
```rust
use psp_logger::{PspLogger, PspLoggerConfig, OutputStream};
use log::{trace, debug, info, warn, error};

// Configure logging to only allow messages with debug-level or above.
// Map debug and info to stdout, letting other levels use the default stderr.
let config = PspLoggerConfig::new(log::LevelFilter::Debug)
    .with_debug_stream(OutputStream::StdOut)
    .with_info_stream(OutputStream::StdOut);

let _ = psp_logger::PspLogger::init(config);

trace!("This will be filtered out.");
debug!("This will be logged to stdout.");
info!("This will also be logged to stdout");
warn!("This will be logged to stderr.");
error!("This will also be logged to stder.");

```