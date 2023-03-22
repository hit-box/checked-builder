use checked_builder::CheckedBuilder;

#[derive(CheckedBuilder, Debug, Default)]
pub struct Config {
    pub enable_logging: String,
    pub enable_tracing: bool,
    pub host: &'static str,
    pub port: u16,
}

fn main() {
    let builder = Config::builder()
        .enable_logging("Info".to_owned())
        .enable_tracing(false)
        .host("localhost")
        .port(8080)
        .enable_logging("Debug".to_owned());
    dbg!(&builder);
    let config = builder.build();
    dbg!(&config);
}
