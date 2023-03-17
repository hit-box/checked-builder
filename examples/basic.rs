use checked_builder::CheckedBuilder;

#[derive(CheckedBuilder, Debug, Default)]
pub struct Config {
    pub enable_logging: String,
    pub enable_tracing: bool,
    pub host: &'static str,
}

fn main() {
    let builder = Config::builder()
        .enable_logging("Info".to_owned())
        .enable_tracing(false)
        .host("localhost")
        .enable_logging("Debug".to_owned());
    dbg!(&builder);
    let config = builder.build();
    dbg!(&config);
}
