use std::path::PathBuf;

pub struct Config {
	// debug mode
	debug: bool,

	//app dir path
	app_dir: PathBuf,

	// log dir path
	log_dir: PathBuf,

	// host address
	server: String,

	// device id
	device_id: String,

	// daemon mode
	daemon: bool,
}

impl Config {
	pub fn new() -> Self {
		Self {
			debug: false,
			app_dir: PathBuf::new(),
			log_dir: PathBuf::new(),
			server: "".to_string(),
			device_id: "".to_string(),
			daemon: false,
		}
	}

	pub fn set_debug(&mut self, debug: bool) {
		self.debug = debug;
	}
	pub fn set_app_dir(&mut self, path: PathBuf) {
		self.app_dir = path;
	}

	pub fn set_log_dir(&mut self, path: PathBuf) {
		self.log_dir = path;
	}

	pub fn set_server(&mut self, host: &str, port: &str) {
		self.server = host.to_string() + ":" + port;
	}

	pub fn set_device_id(&mut self, device_id: String) {
		self.device_id = device_id;
	}

	pub fn set_daemon(&mut self, daemon: bool) {
		self.daemon = daemon;
	}

	pub fn get_debug(&self) -> bool {
		self.debug.clone()
	}

	pub fn get_app_dir(&self) -> PathBuf {
		self.app_dir.clone()
	}

	pub fn get_log_dir(&self) -> PathBuf {
		self.log_dir.clone()
	}

	pub fn get_server(&self) -> String {
		self.server.clone()
	}

	pub fn get_device_id(&self) -> String {
		self.device_id.clone()
	}

	pub fn get_daemon(&self) -> bool {
		self.daemon.clone()
	}
}


