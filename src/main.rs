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

use std::fmt::Debug;
use clap::builder::styling::Reset;
use clap::{Args, Parser, Subcommand};
use colored::*;
// use log::LevelFilter;
// use log::{debug, error, info, trace, warn};
use log4rs::append::console::ConsoleAppender;
use std::path::PathBuf;
use std::string::ToString;
use std::sync::Arc;
use ::log::{error, info, debug, warn, trace, LevelFilter};
use local_ip_address::local_ip;
use log4rs::append::rolling_file::policy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use single_instance::error::SingleInstanceError;
use single_instance::SingleInstance;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
// use tokio::signal;


#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(target_os = "windows")]
use tokio::signal::windows;
use tokio::sync::Mutex;
// use tracing:: {trace, info, error,warn,debug};

mod cancel;
mod config;
mod connections;
mod daemon;
mod packet;
mod tty_manager;
mod myerror;
mod trace;
mod log;
mod dterm;

use config::Config as dterm_config;

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

	#[arg(short, long)]
	trace: bool,

	/// host address
	#[arg(short = 's', long, default_value = "127.0.0.1")]
	host: Option<String>,

	/// host port
	#[arg(short, long, default_value = "8333")]
	port: Option<String>,

	/// device id
	#[arg(short = 'I', long = "device_id", default_value = "1111")]
	device_id: Option<String>,

	/// daemon mode
	#[arg(short = 'D', long, default_value = "false")]
	daemon: Option<bool>,

	/// description
	#[arg(long)]
	description: Option<String>,
}

const APP_NAME: &str = "dterm";

fn parse_args(cli: &mut Cli, cfg: &mut dterm_config) {
	println!("{}", format!("{} arguments", APP_NAME).italic().bold().bright_yellow());
	if cli.trace {
		cfg.set_trace(cli.trace);
	} else {
		match &cli.log_level {
			Some(s) => {
				println!("{}", format!("log_level: {}", s).italic().bright_yellow());
				let level = match s.to_lowercase().as_str() {
					"error" => LevelFilter::Error,
					"warn" => LevelFilter::Warn,
					"info" => LevelFilter::Info,
					"debug" => LevelFilter::Debug,
					"trace" => LevelFilter::Trace,
					_ => LevelFilter::Info,
				};
				cfg.set_log_level(level);
			}
			None => {
				println!("{}", "log_level: None, default: info".italic().bold().bright_yellow());
				cfg.set_log_level(LevelFilter::Info);
			}
		}
	}

	let d = &cli.debug;
	cfg.set_debug(d.to_owned());
	println!("{}", format!("debug: {}", d).italic().bold().bright_yellow());

	let host = cli.host.as_deref().unwrap_or_else(|| {
		println!("{}", "host: none".italic().bold().bright_red());
		""
	});
	println!("{}", format!("host: {}", host).italic().bold().bright_yellow());

	let port = cli.port.as_deref().unwrap_or_else(|| {
		println!("{}", "port: none".italic().bold().bright_red());
		""
	});
	println!("{}", format!("port: {}", port).italic().bold().bright_yellow());
	cfg.set_server(host, port);

	let device_id = cli.device_id.as_deref().unwrap_or_else(|| {
		println!("{}", "device_id: none".italic().bold().bright_red());
		""
	});
	println!("{}", format!("device_id: {}", device_id).italic().bold().bright_yellow());
	cfg.set_device_id(device_id.to_string());

	let daemon = cli.daemon.unwrap_or_else(|| {
		println!("{}", "daemon: none".italic().bold().bright_red());
		false
	});
	println!("{}", format!("daemon: {}", daemon).italic().bold().bright_yellow());
	cfg.set_daemon(daemon);

	let description = cli.description.as_ref().map_or_else(|| {
		println!("{}", "description: none".italic().bold().bright_red());
		local_address()
	}, |desc| desc.clone());
	println!("{}", format!("description: {}", description).italic().bold().bright_yellow());
	cfg.set_description(description.to_string());
	info!("cfg: {:?}", cfg);
}

#[cfg(target_os = "linux")]
async fn handle_signal(
	caller: &mut cancel::CancelCaller,
) -> Result<(), Box<dyn std::error::Error>> {
	let mut term_stream = signal(SignalKind::terminate())?;
	let mut quit_stream = signal(SignalKind::quit())?;
	let mut int_stream = signal(SignalKind::interrupt())?;
	select! {
        _ = term_stream.recv() => {
            info!("received SIGTERM");
        }
        _ = quit_stream.recv() => {
            info!("received SIGQUIT");
        }
        _= int_stream.recv() => {
            info!("received SIGINT");
        }
    }

	info!("start cancel all tasks");
	caller.cancel_and_wait().await;
	Ok(())
}

#[cfg(target_os = "windows")]
async fn handle_signal(
	caller: &mut cancel::CancelCaller,
) -> Result<(), Box<dyn std::error::Error>> {
	let mut term_stream = windows::ctrl_c()?;
	let mut quit_stream = windows::ctrl_break()?;
	let mut close_stream = windows::ctrl_close()?;
	let mut shutdown_stream = windows::ctrl_shutdown()?;
	select! {
        _ = term_stream.recv() => {
            info!("received Ctrl+C");
        }
        _ = quit_stream.recv() => {
            info!("received Ctrl+Break");
        }
        _= close_stream.recv() => {
            info!("received close");
        }
        _= shutdown_stream.recv() => {
            info!("received shutdown");
        }
    }

	info!("notify all task exit");
	caller.cancel_and_wait().await;

	info!("all task exit, main process exit");
	Ok(())
}



fn local_address() -> String {
	let ip = match local_ip() {
		Ok(ip) => {
			info!("local_ip: {:?}", ip);
			ip.to_string()
		}
		Err(e) => {
			error!("local_ip error: {:?}", e);
			"".to_string()
		}
	};
	ip
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
	let home_dir = dirs::home_dir().ok_or_else(|| {
		println!("{}", "can not get home dir".italic().bold().bright_yellow());
		std::io::Error::new(std::io::ErrorKind::NotFound, "Home directory not found")
	})?;

	let mut cfg = dterm_config::new();
	cfg.set_app_dir(home_dir.join(".codigger").join(APP_NAME));
	cfg.set_log_dir(cfg.get_app_dir().join("log"));

	// set config
	let cli = parse_args(&mut Cli::parse(), &mut cfg);

	// init log or trace
	if cfg.get_trace() {
		trace::trace_init();
	} else {
		log::init_log(cfg.get_log_dir(), cfg.get_log_level());
	}

	if cfg.get_device_id().is_empty() {
		error!("device_id is empty, you must specify an id for your device");
		return Ok(());
	}

	let instance = SingleInstance::new(APP_NAME).and_then(|instance| {
		if instance.is_single() {
			info!("{} is single instance", APP_NAME);
			Ok(instance)
		} else {
			error!("{} is already running", APP_NAME);
			Err(SingleInstanceError::MutexError(0))
		}
	}).map_err(|e| {
		error!("{} is already running, error: {}", APP_NAME, e);
		std::io::Error::new(std::io::ErrorKind::AlreadyExists, "Single instance error")
	})?;

	 dterm::dterm_loop(&cfg).await?;
	Ok(())
}
