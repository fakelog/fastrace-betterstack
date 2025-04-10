## Installation
```toml
[dependencies]
fastrace = { version = "0.7", features = ["enable"] }
fastrace-betterstack.git = "https://github.com/fakelog/fastrace-betterstack.git"
log = "0.4"
logforth = { version = "0.24", features = ["fastrace"] }
```


## Usage
```rust
use fastrace::{Span, collector::Config, prelude::SpanContext};
use fastrace_betterstack::BetterstackReporter;
use log::error;
use logforth::append;

fn main() {
    // Initialize Betterstack reporter with your endpoint and token
    let reporter = BetterstackReporter::new(
        "https://your-betterstack-endpoint.com", // Replace with your Betterstack URL
        "your-betterstack-token",                 // Replace with your Betterstack token
    );

    // Configure logging
    logforth::builder()
        .dispatch(|d| {
            d.filter(log::LevelFilter::Debug)
                .append(append::FastraceEvent::default())
                .append(append::Stderr::default())
        })
        .apply();

    // Set the reporter for Fastrace
    fastrace::set_reporter(reporter, Config::default());

    // Create spans and log events
    {
        let parent = SpanContext::random();
        let root = Span::root("root", parent);
        let _span_guard = root.set_local_parent();

        error!("test error message"); // This will be sent to Betterstack
    }

    // Ensure all events are flushed before exiting
    fastrace::flush();
}
```
