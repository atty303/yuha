//! Test utilities and helpers for yuha client tests

use std::path::PathBuf;
use yuha_client::transport::{LocalTransport, LocalTransportConfig, TransportConfig};
use yuha_client::{Client, client_transport::connect_local};
use yuha_core::protocol::ProtocolResponse;

/// Test categories for filtering
#[derive(Debug, Clone, Copy)]
pub enum TestCategory {
    /// Fast unit tests
    Unit,
    /// Fast integration tests using local transport
    FastIntegration,
    /// Slow integration tests using Docker/SSH
    SlowIntegration,
}

/// Get the built remote binary path
pub fn get_remote_binary_path() -> PathBuf {
    PathBuf::from(yuha_client::client::get_remote_binary_path())
}

/// Create a local transport with optional custom configuration
pub fn create_local_transport(transport_config: Option<TransportConfig>) -> LocalTransport {
    let local_config = LocalTransportConfig {
        binary_path: get_remote_binary_path(),
        args: vec!["--stdio".to_string()],
    };
    LocalTransport::new(local_config, transport_config.unwrap_or_default())
}

/// Create and connect a local client for testing
pub async fn create_local_client() -> anyhow::Result<Client<LocalTransport>> {
    connect_local(get_remote_binary_path(), TransportConfig::default())
        .await
        .map_err(|e| anyhow::anyhow!(e))
}

/// Create and connect a local transport client for testing (alias for compatibility)
pub async fn create_local_transport_client() -> anyhow::Result<Client<LocalTransport>> {
    create_local_client().await
}

/// Helper macro for asserting response types
macro_rules! assert_response {
    (success, $response:expr) => {
        match $response {
            ProtocolResponse::Success => {}
            ProtocolResponse::Error { message } => {
                panic!("Expected success, got error: {}", message)
            }
            ProtocolResponse::Data { .. } => panic!("Expected success, got data response"),
        }
    };
    (data, $response:expr) => {
        match $response {
            ProtocolResponse::Data { items } => items,
            ProtocolResponse::Success => panic!("Expected data response, got success"),
            ProtocolResponse::Error { message } => {
                panic!("Expected data response, got error: {}", message)
            }
        }
    };
    (error, $response:expr) => {
        match $response {
            ProtocolResponse::Error { message } => message,
            ProtocolResponse::Success => panic!("Expected error, got success"),
            ProtocolResponse::Data { .. } => panic!("Expected error, got data response"),
        }
    };
}

/// Assert that a response is successful
pub fn assert_success(response: &ProtocolResponse) {
    assert_response!(success, response);
}

/// Assert that a response contains data
pub fn assert_data(response: &ProtocolResponse) -> &[yuha_core::protocol::ResponseItem] {
    assert_response!(data, response)
}

/// Assert that a response is an error
pub fn assert_error(response: &ProtocolResponse) -> &str {
    assert_response!(error, response)
}

/// Test fixture for common test scenarios
pub struct TestFixture {
    pub client: Client<LocalTransport>,
}

impl TestFixture {
    /// Create a new test fixture with a connected local client
    pub async fn new() -> anyhow::Result<Self> {
        let client = create_local_client().await?;
        Ok(Self { client })
    }

    /// Test clipboard round-trip
    pub async fn test_clipboard_roundtrip(&self, content: &str) -> anyhow::Result<()> {
        // Set clipboard
        self.client.set_clipboard(content.to_string()).await?;

        // Get clipboard
        let retrieved = self.client.get_clipboard().await?;
        assert_eq!(retrieved, content);

        Ok(())
    }
}

/// Macro to skip slow tests when running fast tests
#[macro_export]
macro_rules! slow_test {
    () => {
        if std::env::var("YUHA_FAST_TEST").is_ok() {
            println!("Skipping slow test in fast test mode");
            return Ok(());
        }
    };
}

/// Macro to mark tests that require Docker
#[macro_export]
macro_rules! docker_test {
    () => {
        slow_test!();
        if std::env::var("YUHA_SKIP_DOCKER").is_ok() {
            println!("Skipping Docker test");
            return Ok(());
        }
    };
}
