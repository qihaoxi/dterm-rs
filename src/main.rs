#![cfg_attr(
debug_assertions,
allow(dead_code, unused_imports, unused_variables, unused_mut)
)]
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

use clap::builder::styling::Reset;
use clap::{Args, Parser, Subcommand};
use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use log4rs;
use log4rs::append::console::ConsoleAppender;
use std::string::ToString;
// use log4rs::append::file::FileAppender;
use log4rs::append::rolling_file::policy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use tokio::{select, signal};
// use tokio::signal;
#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(target_os = "windows")]
use tokio::signal::windows;


use tokio_util::sync::CancellationToken;

mod cancel;

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

	/// daemon mode
	#[arg(short = 'D', long, default_value = "false")]
	daemon: Option<bool>,
}

const APP_NAME: &str = "dterm";

fn init_log(dir: &str) {
	let full_dir = format!("{}/{}/log/{}.log", dir, APP_NAME, APP_NAME);

	let trigger = policy::compound::trigger::size::SizeTrigger::new(128 * 1024 * 1024);

	let roller = policy::compound::roll::fixed_window::FixedWindowRoller::builder()
		.build((full_dir.clone() + ".{}").as_str(), 100)
		.unwrap();

	let policy = policy::compound::CompoundPolicy::new(Box::new(trigger), Box::new(roller));

	let file = RollingFileAppender::builder()
		.encoder(Box::new(PatternEncoder::new(
			"{h([{d(%Y-%m-%d %H:%M:%S)} {l} {t} {f}:{L}])} - {m} {n}",
		)))
		.build(full_dir.clone(), Box::new(policy))
		.unwrap();

	let stdout = ConsoleAppender::builder()
		.encoder(Box::new(PatternEncoder::new(
			"{h([{d(%Y-%m-%d %H:%M:%S)} {l} {f}:{L}])} - {m} {n}",
		)))
		.build();

	let config = Config::builder()
		.appender(Appender::builder().build("stdout", Box::new(stdout)))
		.appender(Appender::builder().build("file", Box::new(file)))
		// .logger(Logger::builder().build("app::backend::db", LevelFilter::Trace))
		.logger(Logger::builder().build("app::".to_owned() + APP_NAME, LevelFilter::Trace))
		.build(
			Root::builder()
				.appender("stdout")
				.appender("file")
				.build(LevelFilter::Trace),
		)
		.unwrap();

	log4rs::init_config(config).unwrap();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
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

	match cli.daemon {
		Some(s) => println!("daemon: {}", s),
		None => println!("daemon: None"),
	}
	init_log(".codigger");

	let (mut cancel_caller, mut cancel_watcher) = cancel::cancel::new_cancel();
	tokio::spawn(async move {
		cancel_watcher.wait().await;
		println!("work task start clean resource");
		tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
		println!("work task end");
	});

	#[cfg(target_os = "linux")]
		let _ = handle_signal(&mut cancel_caller).await;
	#[cfg(target_os = "windows")]
		let _ = handle_signal(&mut cancel_caller).await;
	Ok(())
}

#[cfg(target_os = "linux")]
async fn handle_signal(caller: &mut CancelCaller) -> Result<(), Box<dyn std::error::Error>> {
	let mut term_stream = signal(SignalKind::terminate())?;
	let mut quit_stream = signal(SignalKind::quit())?;
	let mut int_stream = signal(SignalKind::interrupt())?;
	select! {
        _ = term_stream.recv() => {
            println!("received SIGTERM");
        }
        _ = quit_stream.recv() => {
            println!("received SIGQUIT");
        }
        _= int_stream.recv() => {
            println!("received SIGINT");
        }
    }

	println!("start cancel all tasks");
	caller.cancel_and_wait().await;
	Ok(())
}

#[cfg(target_os = "windows")]
async fn handle_signal(caller: &mut cancel::cancel::CancelCaller) -> Result<(), Box<dyn std::error::Error>> {
	let mut term_stream = windows::ctrl_c()?;
	let mut quit_stream = windows::ctrl_break()?;
	let mut close_stream = windows::ctrl_close()?;
	let mut shutdown_stream = windows::ctrl_shutdown()?;
	select! {
        _ = term_stream.recv() => {
            println!("received Ctrl+C");
        }
        _ = quit_stream.recv() => {
            println!("received Ctrl+Break");
        }
        _= close_stream.recv() => {
            println!("received close");
        }
        _= shutdown_stream.recv() => {
            println!("received shutdown");
        }
    }

	println!("notify all task exit");
	caller.cancel_and_wait().await;

	println!("all task exit, main process exit");
	Ok(())
}
