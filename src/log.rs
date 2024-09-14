use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use log4rs::append::console::ConsoleAppender;
use std::path::PathBuf;
use std::string::ToString;
use std::sync::Arc;

use log4rs::append::rolling_file::policy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use colored::Colorize;


use crate::APP_NAME;

pub(crate) fn init_log(dir: PathBuf, level: LevelFilter) {
	let log_name = dir.join(APP_NAME.to_owned() + ".log");
	let log_name_str = log_name.to_str().unwrap_or_else(|| {
		println!("{}", "log_name: none".italic().bold().bright_red());
		""
	});
	println!(
		"{}",
		format!("log_name: {}", log_name_str)
			.italic()
			.bold()
			.bright_yellow()
	);

	let trigger = policy::compound::trigger::size::SizeTrigger::new(128 * 1024 * 1024);

	let roller = policy::compound::roll::fixed_window::FixedWindowRoller::builder()
		// .build((full_dir.clone() + ".{}").as_str(), 100)
		.build(log_name.join(".{}").to_str().unwrap(), 100)
		.unwrap();

	let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));

	let patten = "{h([{d(%Y-%m-%d %H:%M:%S)} {l} {f}:{L}])} - {m} {n}";
	let file = RollingFileAppender::builder()
		.encoder(Box::new(PatternEncoder::new(patten)))
		.build(log_name, Box::new(policy))
		.unwrap();

	let stdout = ConsoleAppender::builder()
		.encoder(Box::new(PatternEncoder::new(patten)))
		.build();

	let config = Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.appender(Appender::builder().build("file", Box::new(file)))
		// .logger(Logger::builder().build("app::backend::db", LevelFilter::Trace))
		.logger(Logger::builder().build("app::".to_owned() + APP_NAME, level))
		.build(
			Root::builder()
				.appender("stdout")
				.appender("file")
				.build(level),
		)
		.unwrap();

	log4rs::init_config(config).unwrap();
}