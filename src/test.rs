use std::fmt::Debug;

#[derive(Debug)]
struct Initial;
impl State for Initial {}
trait State
where
    Self: Debug,
{
}

trait EnableLogging: State + Sized {
    fn enable_logging(self, level: String) -> LoggingEnabled<Self>;
}
impl<S: State> EnableLogging for S {
    fn enable_logging(self, level: String) -> LoggingEnabled<S> {
        LoggingEnabled { prev: self, level }
    }
}
trait Logging {
    fn logging(&self) -> String;
}
impl<S: State> Logging for LoggingEnabled<S> {
    fn logging(&self) -> String {
        self.level.clone()
    }
}
impl<S: State> State for LoggingEnabled<S> {}
#[derive(Debug)]
struct LoggingEnabled<S: State> {
    prev: S,
    level: String,
}
impl<S> Tracing for LoggingEnabled<S>
where
    S: State + Tracing,
{
    fn tracing(&self) -> bool {
        self.prev.tracing()
    }
}

trait EnableTracing: State + Sized {
    fn enable_tracing(self, enabled: bool) -> TracingEnabled<Self>;
}
impl<S: State> EnableTracing for S {
    fn enable_tracing(self, enabled: bool) -> TracingEnabled<S> {
        TracingEnabled {
            prev: self,
            enabled,
        }
    }
}
trait Tracing {
    fn tracing(&self) -> bool;
}
impl<S: State> Tracing for TracingEnabled<S> {
    fn tracing(&self) -> bool {
        self.enabled
    }
}
impl<S: State> State for TracingEnabled<S> {}
#[derive(Debug)]
struct TracingEnabled<S: State> {
    prev: S,
    enabled: bool,
}
impl<S> Logging for TracingEnabled<S>
where
    S: State + Logging,
{
    fn logging(&self) -> String {
        self.prev.logging()
    }
}

#[derive(Debug, Default)]
struct Config {
    enable_logging: String,
    enable_tracing: bool,
    default_value: u8,
}

impl Config {
    fn builder() -> Initial {
        Initial
    }
}

trait Builder
where
    Self: State + Logging + Tracing,
{
    fn build(self) -> Config;
}

impl<S> Builder for S
where
    S: State + Logging + Tracing,
{
    fn build(self) -> Config {
        Config {
            enable_logging: self.logging(),
            enable_tracing: self.tracing(),
            default_value: Default::default(),
        }
    }
}

fn partially_builder() -> impl State {
    Config::builder()
        .enable_logging("Debug".to_owned())
        .enable_tracing(true)
        .enable_logging("Warn".to_owned())
}

#[test]
fn test_builder() {
    let builder = partially_builder()
        .enable_tracing(true)
        .enable_logging("Info".to_owned())
        .enable_tracing(false);
    dbg!(&builder);
    let config = builder.build();
    dbg!(&config);
    dbg!(&config.enable_logging);
    dbg!(&config.enable_tracing);
    dbg!(&config.default_value);
}
