#[cfg(target_os = "linux")]
mod daemon {
    use crate::Cli;
    use daemonize::Daemonize;

    async fn daemonize(cli: &Cli) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let daemonize = Daemonize::new()
            .pid_file("/tmp/test.pid") // Every method except `new` and `start`
            .chown_pid_file(true) // is optional, see `Daemonize` documentation
            // .working_directory("/tmp") // for default behaviour.
            // .user("nobody")
            // .group("daemon") // Group name
            // .group(2)        // or group id.
            // .umask(0o777)    // Set umask, `0o027` by default.
            // .stdout(stdout)  // Redirect stdout to `/tmp/daemon.out`.
            // .stderr(stderr)  // Redirect stderr to `/tmp/daemon.err`.
            .privileged_action(|| "Executed before drop privileges");

        match daemonize.start() {
            Ok(_) => println!("Success, daemonized"),
            Err(e) => eprintln!("Error, {}", e),
        }

        Ok(())
    }
}
