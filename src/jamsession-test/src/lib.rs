mod transport;

use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::time::Duration;

use agent_client_protocol::schema::{McpServer, SessionId};
use agent_client_protocol::{Client, DynConnectTo};
use jamsession::agent::AgentFactory;
use jamsession::error::Error;
use rhaicp::RhaiAgent;
use transport::UnixSocketTransport;

pub use rhaicp::PriorSession;

/// Test implementation of `AgentFactory` that creates in-process RhaiAgent instances.
pub struct RhaiAgentFactory {
    new_session_script: Option<String>,
    prior_sessions: Vec<PriorSession>,
}

impl RhaiAgentFactory {
    pub fn new(config: &TestDaemonConfig) -> Self {
        Self {
            new_session_script: if config.agent_script.is_empty() {
                None
            } else {
                Some(config.agent_script.clone())
            },
            prior_sessions: config.prior_sessions.clone(),
        }
    }
}

impl AgentFactory for RhaiAgentFactory {
    fn create_transport(
        &self,
        session_id: &str,
        _cwd: &Path,
        _mcp_servers: &[McpServer],
    ) -> Result<DynConnectTo<Client>, Error> {
        let mut agent = RhaiAgent::new();
        if let Some(script) = &self.new_session_script {
            agent = agent.new_session_script(script.clone());
            if !session_id.is_empty() {
                let mut prior = self.prior_sessions.clone();
                prior.push(PriorSession {
                    session_id: SessionId::new(session_id),
                    script: script.clone(),
                });
                agent = agent.prior_sessions(prior);
            }
        }
        if !self.prior_sessions.is_empty() && self.new_session_script.is_none() {
            agent = agent.prior_sessions(self.prior_sessions.clone());
        }
        Ok(DynConnectTo::new(agent))
    }
}

/// Configuration for a test daemon instance.
pub struct TestDaemonConfig {
    pub idle_timeout: Duration,
    pub agent_script: String,
    pub prior_sessions: Vec<PriorSession>,
}

impl Default for TestDaemonConfig {
    fn default() -> Self {
        Self {
            idle_timeout: Duration::from_secs(300),
            agent_script: String::new(),
            prior_sessions: Vec::new(),
        }
    }
}

/// An isolated daemon instance for integration testing.
pub struct TestDaemon {
    _temp_dir: tempfile::TempDir,
    socket_path: PathBuf,
    _daemon_handle: tokio::task::JoinHandle<()>,
}

impl TestDaemon {
    /// Start a test daemon with the given configuration.
    /// Panics if the daemon doesn't start within 2 seconds.
    pub async fn start(config: TestDaemonConfig) -> Self {
        let temp_dir = tempfile::TempDir::new().expect("failed to create temp dir");
        let socket_path = temp_dir.path().join("daemon.sock");
        let state_path = temp_dir.path().join("state.json");

        let factory: Arc<dyn AgentFactory> = Arc::new(RhaiAgentFactory::new(&config));
        let idle_timeout = config.idle_timeout;

        let socket_path_clone = socket_path.clone();
        let daemon_handle = tokio::spawn(async move {
            let daemon =
                jamsession::daemon::Daemon::new_with_paths(&state_path, &socket_path_clone)
                    .with_factory(factory)
                    .with_idle_timeout(idle_timeout)
                    .with_quiescence_timeout(Duration::from_millis(10))
                    .with_send_guidelines(false);
            let _ = daemon.run().await;
        });

        let deadline = tokio::time::Instant::now() + Duration::from_secs(2);
        loop {
            if socket_path.exists() {
                break;
            }
            if tokio::time::Instant::now() >= deadline {
                panic!("TestDaemon did not start within 2 seconds");
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }

        Self {
            _temp_dir: temp_dir,
            socket_path,
            _daemon_handle: daemon_handle,
        }
    }

    /// Execute a Rhai client script against this daemon.
    /// Returns the script's last expression as a string.
    pub async fn execute_client(&self, script: &str) -> String {
        self.execute_client_with_cwd(script, Path::new("/tmp"))
            .await
    }

    /// Execute a Rhai client script with a specific cwd.
    pub async fn execute_client_with_cwd(&self, script: &str, cwd: &Path) -> String {
        let transport = UnixSocketTransport::new(&self.socket_path);
        rhaicp::client::RhaiClient::new()
            .cwd(cwd)
            .execute(transport, script)
            .await
            .expect("client script failed")
    }

    pub fn socket_path(&self) -> &Path {
        &self.socket_path
    }
}

impl Drop for TestDaemon {
    fn drop(&mut self) {
        self._daemon_handle.abort();
    }
}
