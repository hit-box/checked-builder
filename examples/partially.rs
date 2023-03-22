use checked_builder::CheckedBuilder;

#[derive(CheckedBuilder, Debug, Default)]
pub struct Config {
    pub enable_logging: String,
    pub enable_tracing: bool,
    pub host: &'static str,
    pub port: u16,
}

fn partially_builder() -> impl ConfigBuilderState
       + config_builder::ConfigBuilderEnableTracing
       + config_builder::ConfigBuilderEnableLogging {
    Config::builder()
        .enable_logging("Warn".to_owned())
        .enable_tracing(true)
}

fn main() {
    let builder = partially_builder()
        .host("localhost")
        .port(8080)
        .enable_tracing(false);
    let config = builder.build();
    println!("{config:#?}");
}
