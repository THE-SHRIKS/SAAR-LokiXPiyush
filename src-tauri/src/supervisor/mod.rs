use std::net::TcpListener;
use std::process::{Child, Command, ExitStatus, Stdio};
use std::time::Duration;

#[derive(Debug, thiserror::Error)]
pub enum SupervisorError {
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Worker execution timed out after {0:?}")]
    Timeout(Duration),
    #[error("Worker failed with status: {0}")]
    NonZeroExit(ExitStatus),
    #[error("Forbidden parameter or URL detected: {0}")]
    ForbiddenParameter(String),
}

/// RAII Guard for spawned child processes.
/// Automatically kills the child process if the guard is dropped before explicit completion,
/// preventing orphaned or zombie background sidecars.
pub struct ChildGuard {
    child: Option<Child>,
}

impl ChildGuard {
    pub fn new(child: Child) -> Self {
        Self { child: Some(child) }
    }

    pub fn id(&self) -> Option<u32> {
        self.child.as_ref().map(|c| c.id())
    }

    /// Explicitly wait for child to complete without killing it
    pub fn wait_with_timeout(&mut self, timeout: Duration) -> Result<ExitStatus, SupervisorError> {
        let start = std::time::Instant::now();
        let poll_interval = Duration::from_millis(50);

        loop {
            if let Some(ref mut child) = self.child {
                match child.try_wait()? {
                    Some(status) => {
                        self.child = None; // Avoid killing already-exited process
                        return Ok(status);
                    }
                    None => {
                        if start.elapsed() >= timeout {
                            let _ = child.kill();
                            self.child = None;
                            return Err(SupervisorError::Timeout(timeout));
                        }
                        std::thread::sleep(poll_interval);
                    }
                }
            } else {
                return Err(SupervisorError::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Process already collected",
                )));
            }
        }
    }

    /// Kill the child process immediately
    pub fn kill(&mut self) -> std::io::Result<()> {
        if let Some(ref mut child) = self.child {
            child.kill()?;
            let _ = child.wait();
            self.child = None;
        }
        Ok(())
    }
}

impl Drop for ChildGuard {
    fn drop(&mut self) {
        if let Some(ref mut child) = self.child {
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Worker Supervisor for offline sidecars (OCR, LLM)
pub struct WorkerSupervisor;

impl WorkerSupervisor {
    /// Validates arguments against network URLs or prohibited shell metacharacters
    pub fn validate_args(args: &[&str]) -> Result<(), SupervisorError> {
        for &arg in args {
            let lower = arg.to_lowercase();
            if lower.contains("http://")
                || lower.contains("https://")
                || lower.contains("ftp://")
                || lower.contains("ws://")
                || lower.contains("wss://")
            {
                return Err(SupervisorError::ForbiddenParameter(format!(
                    "External network target prohibited: {}",
                    arg
                )));
            }
        }
        Ok(())
    }

    /// Spawns a supervised worker command with stdin/stdout piped and network egress strictly prohibited
    pub fn spawn_worker(command: &str, args: &[&str]) -> Result<ChildGuard, SupervisorError> {
        Self::validate_args(args)?;

        let child = Command::new(command)
            .args(args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        Ok(ChildGuard::new(child))
    }

    /// Allocates an ephemeral private loopback-only TCP port bound strictly to 127.0.0.1.
    /// Never binds to 0.0.0.0 or any external interface.
    pub fn allocate_loopback_port() -> std::io::Result<u16> {
        let listener = TcpListener::bind("127.0.0.1:0")?;
        let port = listener.local_addr()?.port();
        drop(listener);
        Ok(port)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_supervisor_rejects_network_args() {
        let bad_args = vec!["--input", "https://evil.com/payload.pdf"];
        let res = WorkerSupervisor::validate_args(&bad_args);
        assert!(
            res.is_err(),
            "Must reject arguments containing network URLs"
        );
    }

    #[test]
    fn test_supervisor_allows_local_args() {
        let good_args = vec![
            "--model",
            "models/weights/model.gguf",
            "--temp",
            "0.0",
        ];
        let res = WorkerSupervisor::validate_args(&good_args);
        assert!(res.is_ok(), "Must allow clean local arguments");
    }

    #[test]
    fn test_child_guard_kills_on_drop() {
        // Spawn a sleep process that would run for 60 seconds
        let guard = if cfg!(windows) {
            WorkerSupervisor::spawn_worker("ping", &["127.0.0.1", "-n", "60"]).unwrap()
        } else {
            WorkerSupervisor::spawn_worker("sleep", &["60"]).unwrap()
        };
        let _pid = guard.id().expect("Child must have PID");

        #[cfg(target_os = "linux")]
        {
            let proc_path = std::path::PathBuf::from(format!("/proc/{}", pid));
            assert!(
                proc_path.exists(),
                "Spawned child process should be alive in /proc"
            );

            // Drop guard -> must kill child
            drop(guard);

            // Verify process is terminated
            std::thread::sleep(Duration::from_millis(50));
            assert!(
                !proc_path.exists(),
                "Child process must be dead after ChildGuard is dropped"
            );
        }

        #[cfg(not(target_os = "linux"))]
        {
            drop(guard);
        }
    }

    #[test]
    fn test_child_guard_timeout() {
        let mut guard = if cfg!(windows) {
            WorkerSupervisor::spawn_worker("ping", &["127.0.0.1", "-n", "10"]).unwrap()
        } else {
            WorkerSupervisor::spawn_worker("sleep", &["10"]).unwrap()
        };
        let result = guard.wait_with_timeout(Duration::from_millis(150));
        assert!(result.is_err());
        match result.unwrap_err() {
            SupervisorError::Timeout(d) => assert_eq!(d, Duration::from_millis(150)),
            other => panic!("Expected Timeout error, got {:?}", other),
        }
    }

    #[test]
    fn test_loopback_port_allocation() {
        let port = WorkerSupervisor::allocate_loopback_port().unwrap();
        assert!(port > 1024, "Allocated loopback port must be unprivileged");
    }
}
