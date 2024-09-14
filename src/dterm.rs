use log::info;
use tokio::select;
use crate::{cancel, config, handle_signal};

pub(crate) async fn dterm_loop(cfg: &config::Config) -> Result<(), Box<dyn std::error::Error>> {
	let (mut cancel_caller, mut cancel_watcher) = cancel::new_cancel();
	// let mut tty_manager_watcher = cancel_watcher.clone();
	// let mut connection_watcher = cancel_watcher.clone();

	tokio::spawn(async move {
		select! {
            _ = cancel_watcher.wait() => {
                info!("work task start clean resource");
                // tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
                info!("work task end");
            }
        }
	});

	// let mut tty_manager = tty_manager::TtyManager::new(cfg.get_server(), connection_watcher);
	// let tty_manager = Arc::new(Mutex::new(tty_manager));
	// tokio::spawn(async move {
	//     let mut tty_manager_lock = tty_manager.lock().await;
	//     loop {
	//         select! {
	//             tty_manager_result = tty_manager_lock.run() => {
	//                 match tty_manager_result {
	//                     Ok(_) => {
	//                         info!("TtyManager run success");
	//                     }
	//                     Err(e) => {
	//                         error!("TtyManager run failed, {:?}", e);
	//                     }
	//                 }
	//             }
	//             tty_watcher_result = tty_manager_watcher.wait() => {
	//                 info!("tty_watcher wait");
	//                 break;
	//             }
	//
	//         }
	//         tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
	//     }
	// });

	// let mut tty_manager = tty_manager::TtyManager::new(cfg.get_server(), connection_watcher);
	// let tty_manager = Arc::new(Mutex::new(tty_manager));
	// let mut tty_manager_lock = tty_manager.lock().await;
	// loop {
	// 	select! {
	//          tty_manager_result = tty_manager.main_loop() => {
	//              match tty_manager_result {
	//                  Ok(_) => {
	//                      info!("TtyManager run success");
	//                  }
	//                  Err(e) => {
	//                      error!("TtyManager run failed, {:?}", e);
	//                  }
	//              }
	//             tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
	//          }
	//          tty_watcher_result = tty_manager_watcher.wait() => {
	//              info!("tty_watcher wait");
	//              break;
	//          }
	//     }
	// };


	let _ = handle_signal(&mut cancel_caller).await;
	info!("main process exit");
	Ok(())
}