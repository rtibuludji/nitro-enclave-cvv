use std::io::Write;
use log::LevelFilter;

pub fn init_logging() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::Builder::from_default_env()
        .format(|buf, record| {
            writeln!(
                buf,
                "[{} {} {}] {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S%.3f"),
                record.level(),
                record.target(),
                record.args()
            )
        })
        .filter_level(log::LevelFilter::Debug)
        .filter_module("rustls", LevelFilter::Off)
        .filter_module("rustls::common_state", LevelFilter::Off)
        .filter_module("hyper", LevelFilter::Warn) 
        .filter_module("aws_smithy", LevelFilter::Warn)
        .init();

    Ok(())
}
