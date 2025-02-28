//! Integration tests for http_scrape source.
//! The container configuration file is `docker-compose.http_scrape.yml`
//! It leverages a static file server ("dufs"), which serves the files in tests/data/http-scrape

use std::collections::HashMap;
use tokio::time::{Duration, Instant};

use crate::sources::util::http::HttpMethod;
use crate::{
    config::{ComponentKey, SourceConfig, SourceContext},
    http::Auth,
    serde::default_decoding,
    serde::default_framing_message_based,
    tls,
    tls::TlsConfig,
    SourceSender,
};
use codecs::decoding::DeserializerConfig;
use vector_config::NamedComponent;
use vector_core::config::log_schema;

use super::{
    tests::{run_compliance, INTERVAL_SECS},
    HttpScrapeConfig,
};

use crate::test_util::components::{run_and_assert_source_error, COMPONENT_ERROR_TAGS};

fn dufs_address() -> String {
    std::env::var("DUFS_ADDRESS").unwrap_or_else(|_| "http://localhost:5000".into())
}

fn dufs_auth_address() -> String {
    std::env::var("DUFS_AUTH_ADDRESS").unwrap_or_else(|_| "http://localhost:5000".into())
}

fn dufs_https_address() -> String {
    std::env::var("DUFS_HTTPS_ADDRESS").unwrap_or_else(|_| "https://localhost:5000".into())
}

/// The error path should not yield any events and must emit the required error internal events.
/// Consider extracting this function into test_util , if it is always true that if the error
/// internal event metric is fired that no events would be outputted by the source.
pub(crate) async fn run_error(config: HttpScrapeConfig) {
    let events =
        run_and_assert_source_error(config, Duration::from_secs(3), &COMPONENT_ERROR_TAGS).await;

    assert!(events.is_empty());
}

/// An endpoint in the config that is not reachable should generate errors.
#[tokio::test]
async fn invalid_endpoint() {
    run_error(HttpScrapeConfig {
        endpoint: "http://nope".to_string(),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: default_decoding(),
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        auth: None,
        tls: None,
        log_namespace: None,
    })
    .await;
}

/// Logs (raw bytes) should be scraped and decoded successfully.
#[tokio::test]
async fn scraped_logs_bytes() {
    let events = run_compliance(HttpScrapeConfig {
        endpoint: format!("{}/logs/bytes", dufs_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Bytes,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        auth: None,
        tls: None,
        log_namespace: None,
    })
    .await;
    // panics if not log event
    let log = events[0].as_log();
    assert_eq!(
        log[log_schema().source_type_key()],
        HttpScrapeConfig::NAME.into()
    );
}

/// Logs (json) should be scraped and decoded successfully.
#[tokio::test]
async fn scraped_logs_json() {
    let events = run_compliance(HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        auth: None,
        tls: None,
        log_namespace: None,
    })
    .await;
    // panics if not log event
    let log = events[0].as_log();
    assert_eq!(
        log[log_schema().source_type_key()],
        HttpScrapeConfig::NAME.into()
    );
}

/// Metrics should be scraped and decoded successfully.
#[tokio::test]
async fn scraped_metrics_native_json() {
    let events = run_compliance(HttpScrapeConfig {
        endpoint: format!("{}/metrics/native.json", dufs_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::NativeJson,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        auth: None,
        tls: None,
        log_namespace: None,
    })
    .await;

    // panics if not metric event
    let metric = events[0].as_metric();
    assert_eq!(
        metric.tags().unwrap()[log_schema().source_type_key()],
        HttpScrapeConfig::NAME.to_string()
    );
}

/// Traces should be scraped and decoded successfully.
#[tokio::test]
async fn scraped_trace_native_json() {
    let events = run_compliance(HttpScrapeConfig {
        endpoint: format!("{}/traces/native.json", dufs_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::NativeJson,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        auth: None,
        tls: None,
        log_namespace: None,
    })
    .await;

    let trace = events[0].as_trace();
    assert_eq!(
        trace.as_map()[log_schema().source_type_key()],
        HttpScrapeConfig::NAME.into()
    );
}

/// Passing no authentication for the auth-gated endpoint should yield errors.
#[tokio::test]
async fn unauthorized_no_auth() {
    run_error(HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_auth_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        auth: None,
        tls: None,
        log_namespace: None,
    })
    .await;
}

/// Passing the incorrect credentials for the auth-gated endpoint should yield errors.
#[tokio::test]
async fn unauthorized_wrong_auth() {
    run_error(HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_auth_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        tls: None,
        auth: Some(Auth::Basic {
            user: "white_rabbit".to_string(),
            password: "morpheus".to_string().into(),
        }),
        log_namespace: None,
    })
    .await;
}

/// Passing the correct credentials for the auth-gated endpoint should succeed.
#[tokio::test]
async fn authorized() {
    run_compliance(HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_auth_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        tls: None,
        auth: Some(Auth::Basic {
            user: "user".to_string(),
            password: "pass".to_string().into(),
        }),
        log_namespace: None,
    })
    .await;
}

/// Passing an incorrect CA file for TLS should yield errors.
#[tokio::test]
async fn tls_invalid_ca() {
    run_error(HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_https_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        tls: Some(TlsConfig {
            ca_file: Some("tests/data/http-scrape/certs/invalid-ca-cert.pem".into()),
            ..Default::default()
        }),
        auth: None,
        log_namespace: None,
    })
    .await;
}

/// Passing the correct CA file for TLS should succeed.
#[tokio::test]
async fn tls_valid() {
    run_compliance(HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_https_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        tls: Some(TlsConfig {
            ca_file: Some(tls::TEST_PEM_CA_PATH.into()),
            ..Default::default()
        }),
        auth: None,
        log_namespace: None,
    })
    .await;
}

/// The source should shutdown cleanly when the shutdown signal is received.
#[tokio::test]
async fn shutdown() {
    let source_id = ComponentKey::from("http_scrape_shutdown");
    let source = HttpScrapeConfig {
        endpoint: format!("{}/logs/json.json", dufs_address()),
        scrape_interval_secs: INTERVAL_SECS,
        query: HashMap::new(),
        decoding: DeserializerConfig::Json,
        framing: default_framing_message_based(),
        headers: HashMap::new(),
        method: HttpMethod::Get,
        tls: None,
        auth: None,
        log_namespace: None,
    };

    // build the context for the source and get a SourceShutdownCoordinator to signal with
    let (tx, _rx) = SourceSender::new_test();
    let (context, mut shutdown) = SourceContext::new_shutdown(&source_id, tx);

    // start source
    let source = source
        .build(context)
        .await
        .expect("source should not fail to build");
    let source_handle = tokio::spawn(source);

    // signal the source to shut down
    let deadline = Instant::now() + Duration::from_secs(1);
    let shutdown_complete = shutdown.shutdown_source(&source_id, deadline);
    let shutdown_success = shutdown_complete.await;
    assert!(shutdown_success);

    // Ensure source actually shut down successfully.
    let _ = source_handle.await.unwrap();
}
