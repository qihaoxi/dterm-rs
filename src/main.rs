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
use colored::*;
use log::LevelFilter;
use log::{debug, error, info, trace, warn};
use log4rs::append::console::ConsoleAppender;
use std::path::PathBuf;
use std::string::ToString;

use local_ip_address::local_ip;
use log4rs::append::rolling_file::policy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Logger, Root};
use log4rs::encode::pattern::PatternEncoder;
use single_instance::SingleInstance;
use tokio::{select, signal};
use tokio_util::sync::CancellationToken;
// use tokio::signal;

#[cfg(target_os = "linux")]
use tokio::signal::unix::{signal, SignalKind};
#[cfg(target_os = "windows")]
use tokio::signal::windows;

mod cancel;
mod config;
mod connections;
mod daemon;
mod packet;
mod tty_manager;

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

    /// host address
    #[arg(short = 's', long, default_value = "127.0.0.1")]
    host: Option<String>,

    /// host port
    #[arg(short, long, default_value = "8331")]
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

fn init_log(dir: PathBuf, level: LevelFilter) {
    let log_name = dir.join(APP_NAME.to_owned() + ".log");
    let log_name_str = match log_name.to_str() {
        Some(s) => s,
        None => {
            println!("{}", "log_name: none".italic().bold().bright_red());
            ""
        }
    };
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

fn parse_args(cli: &mut Cli, cfg: &mut dterm_config) {
    println!(
        "{}",
        format!("{} arguments", APP_NAME)
            .italic()
            .bold()
            .bright_yellow()
    );
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
            init_log(cfg.get_log_dir(), level);
        }
        None => {
            println!(
                "{}",
                "log_level: None, default: info"
                    .italic()
                    .bold()
                    .bright_yellow()
            );
            init_log(cfg.get_log_dir(), LevelFilter::Info);
        }
    }

    let d = &cli.debug;
    cfg.set_debug(d.to_owned());
    println!(
        "{}",
        format!("debug: {}", d).italic().bold().bright_yellow()
    );

    let host = match &cli.host {
        Some(h) => {
            println!("{}", format!("host: {}", h).italic().bold().bright_yellow());
            h
        }
        None => {
            println!("{}", "host: none".italic().bold().bright_red());
            ""
        }
    };

    let port = match &cli.port {
        Some(p) => {
            println!("{}", format!("port: {}", p).italic().bold().bright_yellow());
            p
        }
        None => {
            println!("{}", "port: none".italic().bold().bright_red());
            ""
        }
    };
    cfg.set_server(host, port);

    let device_id = match &cli.device_id {
        Some(s) => {
            println!(
                "{}",
                format!("device_id: {}", s).italic().bold().bright_yellow()
            );
            s
        }
        None => {
            println!("{}", "device_id: none".italic().bold().bright_red());
            ""
        }
    };
    cfg.set_device_id(device_id.to_string());

    let daemon = match &cli.daemon {
        Some(s) => {
            println!(
                "{}",
                format!("daemon: {}", s).italic().bold().bright_yellow()
            );
            true
        }
        None => {
            println!("{}", "daemon: none".italic().bold().bright_red());
            false
        }
    };
    cfg.set_daemon(daemon);

    match &cli.description {
        Some(s) => {
            println!(
                "{}",
                format!("description: {}", s)
                    .italic()
                    .bold()
                    .bright_yellow()
            );
            cfg.set_description(s.to_string());
        }
        None => {
            println!("{}", "description: none".italic().bold().bright_red());
            let addr = local_address();
            cfg.set_description(addr);
        }
    };
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

async fn dterm_loop(cfg: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
    let (mut cancel_caller, mut cancel_watcher) = cancel::new_cancel();
    let mut tty_manager_watcher = cancel_watcher.clone();
    let mut connection_watcher = cancel_watcher.clone();

    tokio::spawn(async move {
        select! {
            _ = cancel_watcher.wait() => {
                info!("work task start clean resource");
                tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                info!("work task end");
            }
        }
    });

    let mut tty_manager = tty_manager::TtyManager::new(cfg.get_server(), connection_watcher);
    tokio::spawn(async move {
        loop {
            select! {
                tty_manager_result = tty_manager.run() => {
                    match tty_manager_result {
                        Ok(_) => {
                            info!("TtyManager run success");
                        }
                        Err(e) => {
                            error!("TtyManager run failed, {:?}", e);
                        }
                    }
                }
                tty_watcher_result = tty_manager_watcher.wait() => {
                    info!("tty_watcher wait");
                    break;
                }

            }
            tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
        }
    });

    let _ = handle_signal(&mut cancel_caller).await;
    info!("main process exit");
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
    let home_dir = match dirs::home_dir() {
        Some(p) => p,
        None => {
            println!(
                "{}",
                format!("can not get home dir")
                    .italic()
                    .bold()
                    .bright_yellow()
            );
            return Ok(());
        }
    };
    let mut cfg = dterm_config::new();
    cfg.set_app_dir(home_dir.join(".codigger").join(APP_NAME));
    cfg.set_log_dir(cfg.get_app_dir().join("log"));

    // set config
    let cli = parse_args(&mut Cli::parse(), &mut cfg);

    if cfg.get_device_id().is_empty() {
        error!("device_id is empty, you must specify an id for your device");
        return Ok(());
    }

    let instance = match SingleInstance::new(APP_NAME) {
        Ok(instance) => {
            if instance.is_single() {
                info!("{} is single instance", APP_NAME);
            } else {
                error!("{} is already running", APP_NAME);
                return Ok(());
            }
            instance
        }
        Err(e) => {
            error!("{} is already running, error: {}", APP_NAME, e);
            return Ok(());
        }
    };

    dterm_loop(&cfg).await?;
    Ok(())
}
