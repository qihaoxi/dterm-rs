use clap::{Parser, Subcommand, Args};
use log::{error, info, warn};
use log::LevelFilter;
use log4rs;
use log4rs::append::console::ConsoleAppender;
use log4rs::append::file::FileAppender;
use log4rs::encode::pattern::PatternEncoder;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::append::rolling_file::policy;
use colored::*;

#[derive(Parser)]
#[command(author, version, about, long_about)]
#[command(propagate_version = true)]
struct Cli {
	/// debug mode
	#[arg(short, long = "debug", default_value = "false")]
	debug: Option<bool>,

	/// log level [error/warn/info/debug/trace]
	#[arg(short, long = "log-level", default_value = "info")]
	log_level: Option<String>,

	/// server address
	#[arg(short, long)]
	server: Option<String>,

	/// device id
	#[arg(short = 'I', long = "device_id")]
	device_id: Option<String>,
}


fn init_log() {
	let trigger= policy::compound::trigger::size::SizeTrigger::new(128*1024*1024);
	let roller = policy::compound::roll::fixed_window::FixedWindowRoller::builder()
		.build("log/dterm.log.{}", 100).unwrap();
	let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));
	let file = RollingFileAppender::builder()
		.encoder(Box::new(PatternEncoder::new("[{d} {l} {t}] - {m}{n}")))
		.build("log/dterm.log", Box::new(policy)).unwrap();

	let stdout = ConsoleAppender::builder()
		.encoder(Box::new(PatternEncoder::new("{h([{d} {l} {t}])} - {m}{n}")))
		.build();



	let config = Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.appender(Appender::builder().build("file", Box::new(file)))
		.logger(Logger::builder().build("app::backend::db", LevelFilter::Info))
		.logger(Logger::builder().build("app::dterm", LevelFilter::Info))
		.build(Root::builder()
			.appender("stdout")
			.appender("file")
			.build(LevelFilter::Info))
		.unwrap();

	log4rs::init_config(config).unwrap();
}

fn main() {
	let cli = Cli::parse();
	match cli.debug {
		Some(b) => println!("debug: {}", b),
		None => println!("debug: None"),
	}

	match cli.log_level {
		Some(s) => println!("log_level: {}", s),
		None => println!("log_level: None"),
	}

	match cli.server {
		Some(s) => println!("server: {}", s),
		None => println!("server: None"),
	}
	match cli.device_id {
		Some(s) => println!("device_id: {}", s),
		None => println!("device_id: None"),
	}
	init_log();


	// let text = "booting up".to_string();
	info!("{}","booting up");
	error!("{}","booting up");
}
