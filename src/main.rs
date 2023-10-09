#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unused_variables, unused_mut))]
// #![cfg_attr(not(debug_assertions), deny(dead_code, unused_imports, unused_variables, unused_mut))]
// #![allow(unused_must_use)]
// #![allow(unused_parens)]
// #![allow(unused_assignments)]
// #![allow(unused_import_braces)]
// #![allow(unused_macros)]
// #![allow(unused_unsafe)]
// #![allow(unused_doc_comments)]
// #![allow(unused_attributes)]
// #![allow(unused_features)]


use clap::{Parser, Subcommand, Args};
use log::{error, info, trace, warn,debug};
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
// use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy;
// use colored::*;


#[derive(Parser)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
struct Cli {
	/// debug mode
	#[arg(short, long)]
	debug: bool,

	/// log level [error/warn/info/debug/trace]
	#[arg(short, long = "log-level", default_value = "info")]
	log_level: Option<String>,

	/// host address
	// #[arg(short, long)]
	host: Option<String>,

	/// device id
	#[arg(short = 'I', long = "device_id")]
	device_id: Option<String>,
}


fn init_log() {
	let trigger = policy::compound::trigger::size::SizeTrigger::new(128 * 1024 * 1024);
	let roller = policy::compound::roll::fixed_window::FixedWindowRoller::builder()
		.build("log/cdterm.log.{}", 100).unwrap();
	let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
	let file = RollingFileAppender::builder()
		// .encoder(Box::new(PatternEncoder::new("[{d(%Y-%m-%d %H:%M:%S)} {l} {t} {f}:{L}] - {m} {n}")))
		.encoder(Box::new(PatternEncoder::new("{h([{d(%Y-%m-%d %H:%M:%S)} {l} {t} {f}:{L}])} - {m} {n}")))
		.build("log/cdterm.log", Box::new(policy)).unwrap();

	let stdout = ConsoleAppender::builder()
		.encoder(Box::new(PatternEncoder::new("{h([{d(%Y-%m-%d %H:%M:%S)} {l} {f}:{L}])} - {m} {n}")))
		.build();


	let config = Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.appender(Appender::builder().build("file", Box::new(file)))
		.logger(Logger::builder().build("app::backend::db", LevelFilter::Trace))
		.logger(Logger::builder().build("app::cdterm", LevelFilter::Trace))
		.build(Root::builder()
			.appender("stdout")
			.appender("file")
			.build(LevelFilter::Trace))
		.unwrap();

	log4rs::init_config(config).unwrap();
}

fn main() {
	let cli = Cli::parse();

	let d = cli.debug;
	println!("debug: {}", d);

	match cli.log_level {
		Some(s) => println!("log_level: {}", s),
		None => println!("log_level: None"),
	}

	match cli.host {
		Some(h) => println!("host: {}", h),
		None => println!("host: None"),
	}
	match cli.device_id {
		Some(s) => println!("device_id: {}", s),
		None => println!("device_id: None"),
	}
	init_log();


	// let text = "booting up".to_string();
	trace!("{}","booting up");
	debug!("{}","booting up");
	info!("{}","booting up");
	error!("{}","booting up");
	warn!("{}","booting up");
}
