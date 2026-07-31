//! Configuration management for Truent.
//!
//! Supports TOML-based configuration with environment variable substitution.
//! All env vars are expanded at load time; missing vars cause an error.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::env;

/// Error type for configuration issues.
#[derive(Debug, Clone)]
pub struct ConfigError {
    /// Error message.
    pub message: String,
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Config error: {}", self.message)
    }
}

impl std::error::Error for ConfigError {}

/// Root configuration structure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Metadata: name, version, description.
    pub metadata: Metadata,

    /// Chain endpoints and configuration.
    pub chains: Vec<ChainConfig>,

    /// Invariant definitions.
    pub invariants: Vec<InvariantConfig>,

    /// Alert routing and filtering.
    #[serde(default)]
    pub alert: AlertConfig,

    /// Evaluation parameters.
    #[serde(default)]
    pub evaluation: EvaluationConfig,

    /// Daemon mode settings.
    #[serde(default)]
    pub daemon: DaemonConfig,

    /// Logging configuration.
    #[serde(default)]
    pub logging: LoggingConfig,

    /// Metrics / observability.
    #[serde(default)]
    pub metrics: MetricsConfig,

    /// Security settings.
    #[serde(default)]
    pub security: SecurityConfig,

    /// Performance tuning.
    #[serde(default)]
    pub performance: PerformanceConfig,
}

impl Config {
    /// Load configuration from a TOML file.
    /// Environment variables in the format `${VAR_NAME}` are expanded.
    ///
    /// # Errors
    /// Returns error if file cannot be read, TOML is invalid, or env var is missing.
    pub fn load_from_file(path: &str) -> anyhow::Result<Self> {
        let content = std::fs::read_to_string(path)?;
        Self::load_from_string(&content)
    }

    /// Load configuration from a TOML string.
    /// Environment variables in the format `${VAR_NAME}` are expanded.
    pub fn load_from_string(content: &str) -> anyhow::Result<Self> {
        // Expand environment variables in the content
        let expanded = Self::expand_env_vars(content).map_err(|e| anyhow::anyhow!("{}", e))?;

        // Parse TOML
        let config: Config = toml::from_str(&expanded)
            .map_err(|e| anyhow::anyhow!("Failed to parse TOML: {}", e))?;

        // Validate
        config.validate().map_err(|e| anyhow::anyhow!("{}", e))?;

        Ok(config)
    }

    /// Expand all ${VAR_NAME} patterns to environment variables.
    fn expand_env_vars(content: &str) -> Result<String, ConfigError> {
        let mut result = content.to_string();
        let mut last_pos = 0;

        // Find all ${...} patterns
        while let Some(start) = result[last_pos..].find("${") {
            let start = last_pos + start;
            if let Some(end) = result[start..].find('}') {
                let end = start + end;
                let var_ref = &result[start + 2..end];

                // Get environment variable
                match env::var(var_ref) {
                    Ok(value) => {
                        result.replace_range(start..=end, &value);
                        last_pos = start + value.len();
                    }
                    Err(_) => {
                        return Err(ConfigError {
                            message: format!(
                                "Environment variable not set: {}. \
                                 Please export it before running Truent. \
                                 E.g.: export {}=<value>",
                                var_ref, var_ref
                            ),
                        });
                    }
                }
            } else {
                return Err(ConfigError {
                    message: "Unclosed environment variable reference: ${".to_string(),
                });
            }
        }

        Ok(result)
    }

    /// Validate the configuration for consistency.
    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate chains
        if self.chains.is_empty() {
            return Err(ConfigError {
                message: "At least one chain must be configured in [[chains]]".to_string(),
            });
        }

        let chain_ids: Vec<&str> = self.chains.iter().map(|c| c.id.as_str()).collect();

        // Check for duplicate chain IDs
        for (i, id1) in chain_ids.iter().enumerate() {
            for id2 in &chain_ids[i + 1..] {
                if id1 == id2 {
                    return Err(ConfigError {
                        message: format!("Duplicate chain ID: {}", id1),
                    });
                }
            }
        }

        // Validate invariants
        if self.invariants.is_empty() {
            return Err(ConfigError {
                message: "At least one invariant must be configured in [[invariants]]".to_string(),
            });
        }

        // Check invariant references
        for inv in &self.invariants {
            if !chain_ids.contains(&inv.chain.as_str()) {
                return Err(ConfigError {
                    message: format!(
                        "Invariant '{}' references unknown chain '{}'. \
                         Configured chains: {:?}",
                        inv.name, inv.chain, chain_ids
                    ),
                });
            }

            // Check for duplicate names
            let names: Vec<_> = self.invariants.iter().map(|i| &i.name).collect();
            for (i, name1) in names.iter().enumerate() {
                for name2 in &names[i + 1..] {
                    if name1 == name2 {
                        return Err(ConfigError {
                            message: format!("Duplicate invariant name: {}", name1),
                        });
                    }
                }
            }
        }

        // Validate alert sinks
        for sink in &self.alert.sinks {
            sink.validate()?;
        }

        Ok(())
    }

    /// Get a chain configuration by ID.
    pub fn get_chain(&self, id: &str) -> Option<&ChainConfig> {
        self.chains.iter().find(|c| c.id == id)
    }

    /// Get invariants for a specific chain.
    pub fn invariants_for_chain(&self, chain_id: &str) -> Vec<&InvariantConfig> {
        self.invariants
            .iter()
            .filter(|inv| inv.chain == chain_id)
            .collect()
    }
}

/// Metadata section.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metadata {
    /// Project name.
    pub name: String,

    /// Project version.
    pub version: String,

    /// Optional description.
    #[serde(default)]
    pub description: Option<String>,
}

/// Chain configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChainConfig {
    /// Unique identifier for this chain.
    pub id: String,

    /// Chain type: "evm", "solana", "cosmos".
    #[serde(rename = "type")]
    pub chain_type: String,

    /// Chain ID (for EVM chains).
    pub chain_id: u64,

    /// Primary and fallback RPC URLs.
    pub rpc_urls: Vec<String>,

    /// Optional WebSocket URL.
    #[serde(default)]
    pub ws_url: Option<String>,

    /// Poll interval in milliseconds.
    #[serde(default = "default_poll_interval")]
    pub poll_interval_ms: u64,

    /// Request timeout in milliseconds.
    #[serde(default = "default_timeout")]
    pub timeout_ms: u64,

    /// Retry configuration.
    #[serde(default)]
    pub retry: RetryConfig,

    /// Connection pool configuration.
    #[serde(default)]
    pub pool: PoolConfig,
}

fn default_poll_interval() -> u64 {
    12000
}

fn default_timeout() -> u64 {
    30000
}

/// Retry configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum retry attempts.
    #[serde(default = "default_max_attempts")]
    pub max_attempts: u32,

    /// Initial backoff in milliseconds.
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff_ms: u64,

    /// Maximum backoff in milliseconds.
    #[serde(default = "default_max_backoff")]
    pub max_backoff_ms: u64,
}

fn default_max_attempts() -> u32 {
    3
}

fn default_initial_backoff() -> u64 {
    100
}

fn default_max_backoff() -> u64 {
    400
}

/// Connection pool configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoolConfig {
    /// Maximum concurrent connections.
    #[serde(default = "default_max_connections")]
    pub max_connections: u32,

    /// Keepalive duration in seconds.
    #[serde(default = "default_keepalive")]
    pub keepalive_seconds: u64,
}

fn default_max_connections() -> u32 {
    10
}

fn default_keepalive() -> u64 {
    90
}

/// Invariant configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvariantConfig {
    /// Unique name for this invariant.
    pub name: String,

    /// Human-readable description.
    #[serde(default)]
    pub description: Option<String>,

    /// Which chain to evaluate on.
    pub chain: String,

    /// Contract address.
    pub contract: String,

    /// The invariant condition (expression).
    pub check: String,

    /// Optional baseline block number.
    #[serde(default)]
    pub baseline_block: Option<u64>,

    /// Severity: critical, high, medium, low.
    #[serde(default = "default_severity")]
    pub severity: String,

    /// Optional tags for filtering.
    #[serde(default)]
    pub tags: Vec<String>,
}

fn default_severity() -> String {
    "high".to_string()
}

/// Alert configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AlertConfig {
    /// Throttle identical alerts (seconds).
    #[serde(default = "default_throttle")]
    pub throttle_seconds: u64,

    /// Enable/disable all alerts.
    #[serde(default = "default_alert_enabled")]
    pub enabled: bool,

    /// Log level for violations: error, warn, info.
    #[serde(default = "default_log_level")]
    pub log_level: String,

    /// Alert sinks (where to send alerts).
    #[serde(default)]
    pub sinks: Vec<AlertSink>,
}

fn default_throttle() -> u64 {
    300
}

fn default_alert_enabled() -> bool {
    true
}

fn default_log_level() -> String {
    "error".to_string()
}

/// Configuration for an alert sink.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertSink {
    /// Sink type: slack, webhook, email, etc.
    #[serde(rename = "type")]
    pub sink_type: String,

    /// Optional name for this sink.
    #[serde(default)]
    pub name: Option<String>,

    /// Optional Slack webhook URL.
    #[serde(default)]
    pub webhook_url: Option<String>,

    /// Optional custom message template.
    #[serde(default)]
    pub message_template: Option<String>,

    /// Optional severity filter.
    #[serde(default)]
    pub severities: Vec<String>,

    /// Generic webhook URL.
    #[serde(default)]
    pub url: Option<String>,

    /// Custom headers for webhook.
    #[serde(default)]
    pub headers: BTreeMap<String, String>,

    /// HTTP method: post, put.
    #[serde(default = "default_method")]
    pub method: String,

    /// Enable webhook retry.
    #[serde(default)]
    pub retry_enabled: bool,

    /// Max retry attempts for webhook.
    #[serde(default)]
    pub retry_max_attempts: u32,
}

fn default_method() -> String {
    "post".to_string()
}

impl AlertSink {
    /// Validate this alert sink configuration.
    pub fn validate(&self) -> Result<(), ConfigError> {
        match self.sink_type.as_str() {
            "slack" => {
                if self.webhook_url.is_none() {
                    return Err(ConfigError {
                        message: "Slack sink requires 'webhook_url'".to_string(),
                    });
                }
                Ok(())
            }
            "webhook" => {
                if self.url.is_none() {
                    return Err(ConfigError {
                        message: "Webhook sink requires 'url'".to_string(),
                    });
                }
                Ok(())
            }
            _ => Err(ConfigError {
                message: format!("Unknown alert sink type: {}", self.sink_type),
            }),
        }
    }
}

/// Evaluation configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EvaluationConfig {
    /// Evaluation interval in seconds.
    #[serde(default = "default_eval_interval")]
    pub interval_secs: u64,

    /// Evaluation timeout in seconds.
    #[serde(default = "default_eval_timeout")]
    pub timeout_secs: u64,

    /// Mode: immediate or block_based.
    #[serde(default = "default_eval_mode")]
    pub mode: String,
}

fn default_eval_interval() -> u64 {
    12
}

fn default_eval_timeout() -> u64 {
    30
}

fn default_eval_mode() -> String {
    "block_based".to_string()
}

/// Daemon mode configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DaemonConfig {
    /// Enable daemon mode.
    #[serde(default)]
    pub enabled: bool,

    /// Metrics HTTP server port.
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,

    /// Metrics server host.
    #[serde(default = "default_metrics_host")]
    pub metrics_host: String,

    /// Enable health check endpoint.
    #[serde(default = "default_health_check")]
    pub health_check_enabled: bool,

    /// Reload config on SIGHUP.
    #[serde(default)]
    pub reload_on_sighup: bool,

    /// Graceful shutdown timeout in seconds.
    #[serde(default = "default_shutdown_timeout")]
    pub graceful_shutdown_timeout_secs: u64,
}

fn default_metrics_port() -> u16 {
    9090
}

fn default_metrics_host() -> String {
    "127.0.0.1".to_string()
}

fn default_health_check() -> bool {
    true
}

fn default_shutdown_timeout() -> u64 {
    5
}

/// Logging configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LoggingConfig {
    /// Log level: trace, debug, info, warn, error.
    #[serde(default = "default_log_level")]
    pub level: String,

    /// Format: pretty or json.
    #[serde(default = "default_format")]
    pub format: String,

    /// Optional log file path.
    #[serde(default)]
    pub file: Option<String>,

    /// Maximum log file size in MB.
    #[serde(default)]
    pub max_size_mb: Option<u32>,

    /// Maximum number of backup log files.
    #[serde(default)]
    pub max_backups: Option<u32>,

    /// Redact sensitive fields from logs.
    #[serde(default = "default_redact")]
    pub redact_sensitive: bool,
}

fn default_format() -> String {
    "pretty".to_string()
}

fn default_redact() -> bool {
    true
}

/// Metrics configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetricsConfig {
    /// Enable Prometheus metrics.
    #[serde(default)]
    pub enabled: bool,

    /// Format: prometheus.
    #[serde(default = "default_metrics_format")]
    pub format: String,

    /// Histogram buckets for latency (milliseconds).
    #[serde(default = "default_histogram_buckets")]
    pub histogram_buckets_ms: Vec<u64>,

    /// Sample rate (0.0 to 1.0).
    #[serde(default = "default_sample_rate")]
    pub sample_rate: f64,
}

fn default_metrics_format() -> String {
    "prometheus".to_string()
}

fn default_histogram_buckets() -> Vec<u64> {
    vec![10, 50, 100, 500, 1000, 5000]
}

fn default_sample_rate() -> f64 {
    1.0
}

/// Security configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Redact sensitive values in logs.
    #[serde(default = "default_redact")]
    pub redact_sensitive_in_logs: bool,
}

/// Performance configuration.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerformanceConfig {
    /// Number of evaluator worker threads.
    #[serde(default = "default_eval_workers")]
    pub eval_workers: u32,

    /// Cache compiled invariant expressions.
    #[serde(default)]
    pub cache_expressions: bool,

    /// Cache chain state snapshots.
    #[serde(default)]
    pub cache_state: bool,

    /// State cache TTL in seconds.
    #[serde(default = "default_cache_ttl")]
    pub cache_ttl_secs: u64,
}

fn default_eval_workers() -> u32 {
    4
}

fn default_cache_ttl() -> u64 {
    60
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let minimal_toml = r#"
            [metadata]
            name = "test"
            version = "1.0"

            [[chains]]
            id = "ethereum"
            type = "evm"
            chain_id = 1
            rpc_urls = ["https://eth.example.com"]

            [[invariants]]
            name = "test_invariant"
            chain = "ethereum"
            contract = "0x1234"
            check = "x > 0"
        "#;

        let config = Config::load_from_string(minimal_toml);
        assert!(config.is_ok());
    }

    #[test]
    fn test_missing_chain_reference() {
        let invalid_toml = r#"
            [metadata]
            name = "test"
            version = "1.0"

            [[chains]]
            id = "ethereum"
            type = "evm"
            chain_id = 1
            rpc_urls = ["https://eth.example.com"]

            [[invariants]]
            name = "test_invariant"
            chain = "solana"
            contract = "0x1234"
            check = "x > 0"
        "#;

        let config = Config::load_from_string(invalid_toml);
        assert!(config.is_err());
    }

    #[test]
    fn test_env_var_substitution() {
        env::set_var("TEST_RPC_URL", "https://test.example.com");

        let toml_with_var = r#"
            [metadata]
            name = "test"
            version = "1.0"

            [[chains]]
            id = "ethereum"
            type = "evm"
            chain_id = 1
            rpc_urls = ["${TEST_RPC_URL}"]

            [[invariants]]
            name = "test_invariant"
            chain = "ethereum"
            contract = "0x1234"
            check = "x > 0"
        "#;

        let config = Config::load_from_string(toml_with_var);
        assert!(config.is_ok());

        let cfg = config.unwrap();
        assert_eq!(cfg.chains[0].rpc_urls[0], "https://test.example.com");

        env::remove_var("TEST_RPC_URL");
    }

    #[test]
    fn test_missing_env_var() {
        let toml_with_missing_var = r#"
            [metadata]
            name = "test"
            version = "1.0"

            [[chains]]
            id = "ethereum"
            type = "evm"
            chain_id = 1
            rpc_urls = ["${NONEXISTENT_VAR_12345"}"]

            [[invariants]]
            name = "test_invariant"
            chain = "ethereum"
            contract = "0x1234"
            check = "x > 0"
        "#;

        let config = Config::load_from_string(toml_with_missing_var);
        assert!(config.is_err());
    }
}
