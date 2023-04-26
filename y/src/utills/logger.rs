use fern::colors::{Color, ColoredLevelConfig};
use std::time::{SystemTime};

pub fn setup_logger(log_level: String) -> Result<(), fern::InitError> {
    let level = match log_level.to_lowercase().trim() {
        "trace" => log::LevelFilter::Trace,
        "debug" => log::LevelFilter::Debug,
        "info" => log::LevelFilter::Info,
        "warn" => log::LevelFilter::Warn,
        "error" => log::LevelFilter::Error,
        _ => log::LevelFilter::Info,
    };

    let colors = ColoredLevelConfig::new()
        .info(Color::Green)
        .debug(Color::Cyan)
        .trace(Color::Magenta);

    fern::Dispatch::new()
        .format(move |out, message, record| {
            out.finish(format_args!(
                "[{} {} {}] {}",
                humantime::format_rfc3339_seconds(SystemTime::now()),
                colors.color(record.level()),
                record.target(),
                message
            ))
        })
        .level(level)
        .chain(std::io::stdout())
        //.chain(fern::log_file("output.log")?)
        .apply()?;
    Ok(())
}
