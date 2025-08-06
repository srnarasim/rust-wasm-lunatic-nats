//! NATS communication module for distributed messaging between agent nodes

#[cfg(feature = "nats")]
use async_trait::async_trait;
#[cfg(feature = "nats")]
use async_nats::{Client, ConnectOptions, Message as NatsMessage};
#[cfg(feature = "nats")]
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use std::time::Duration;
#[cfg(feature = "nats")]
use std::sync::atomic::Ordering;
#[cfg(feature = "nats")]
use bytes::Bytes;
use crate::{Result, Error};

#[derive(Debug, Clone)]
pub struct NatsConfig {
    pub url: String,
    pub timeout: Duration,
    pub max_reconnects: Option<usize>,
    pub reconnect_delay: Duration,
}

impl Default for NatsConfig {
    fn default() -> Self {
        Self {
            url: "nats://localhost:4222".to_string(),
            timeout: Duration::from_secs(10),
            max_reconnects: Some(10),
            reconnect_delay: Duration::from_secs(1),
        }
    }
}

#[cfg(feature = "nats")]
#[derive(Debug)]
pub struct NatsConnection {
    client: Client,
    config: NatsConfig,
}

#[cfg(not(feature = "nats"))]
#[derive(Debug)]
pub struct NatsConnection {
    config: NatsConfig,
}

#[cfg(feature = "nats")]
impl NatsConnection {
    pub async fn new(config: NatsConfig) -> Result<Self> {
        let mut connect_options = ConnectOptions::new();
        
        if let Some(max_reconnects) = config.max_reconnects {
            connect_options = connect_options.max_reconnects(max_reconnects);
        }
        
        connect_options = connect_options
            .connection_timeout(config.timeout)
            .reconnect_delay_callback(move |attempts| {
                std::cmp::min(Duration::from_secs(attempts as u64), Duration::from_secs(30))
            });

        let client = connect_options.connect(&config.url).await
            .map_err(|e| Error::Nats(format!("Failed to connect to NATS: {}", e)))?;

        log::info!("Successfully connected to NATS at {}", config.url);

        Ok(Self {
            client,
            config,
        })
    }

    pub async fn publish(&self, subject: &str, data: &[u8]) -> Result<()> {
        let data_bytes = Bytes::copy_from_slice(data);
        self.client.publish(subject.to_string(), data_bytes).await
            .map_err(|e| Error::Nats(format!("Failed to publish: {}", e)))?;
        
        log::debug!("Published message to subject: {}", subject);
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<Vec<crate::agent::Message>> {
        let mut subscriber = self.client.subscribe(subject.to_string()).await
            .map_err(|e| Error::Nats(format!("Failed to subscribe: {}", e)))?;

        let mut messages = Vec::new();
        
        // Non-blocking check for messages with timeout
        match tokio::time::timeout(Duration::from_millis(100), subscriber.next()).await {
            Ok(Some(msg)) => {
                match serde_json::from_slice::<crate::agent::Message>(&msg.payload) {
                    Ok(parsed_msg) => {
                        messages.push(parsed_msg);
                        log::debug!("Received message from subject: {}", subject);
                    },
                    Err(e) => log::warn!("Failed to parse message: {}", e),
                }
            },
            Ok(None) => {
                log::debug!("No messages available on subject: {}", subject);
            },
            Err(_) => {
                // Timeout - no messages available
                log::trace!("No messages received within timeout for subject: {}", subject);
            }
        }

        Ok(messages)
    }

    pub async fn request(&self, subject: &str, data: &[u8]) -> Result<Vec<u8>> {
        let data_bytes = Bytes::copy_from_slice(data);
        let response = self.client
            .request(subject.to_string(), data_bytes).await
            .map_err(|e| Error::Nats(format!("Failed to send request: {}", e)))?;
        
        log::debug!("Received response from request to subject: {}", subject);
        Ok(response.payload.to_vec())
    }

    pub fn is_connected(&self) -> bool {
        self.client.connection_state() == async_nats::connection::State::Connected
    }

    pub async fn flush(&self) -> Result<()> {
        self.client.flush().await
            .map_err(|e| Error::Nats(format!("Failed to flush: {}", e)))?;
        
        log::debug!("Flushed NATS connection");
        Ok(())
    }

    pub async fn drain(&self) -> Result<()> {
        // async-nats doesn't have a direct drain method, but we can close gracefully
        log::debug!("Draining NATS connection (closing gracefully)");
        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        // The client will automatically close when dropped
        log::info!("Closing NATS connection");
        Ok(())
    }

    pub fn get_stats(&self) -> ConnectionStats {
        let stats = self.client.statistics();
        ConnectionStats {
            messages_sent: stats.out_messages.load(Ordering::Relaxed),
            messages_received: stats.in_messages.load(Ordering::Relaxed),
            bytes_sent: stats.out_bytes.load(Ordering::Relaxed),
            bytes_received: stats.in_bytes.load(Ordering::Relaxed),
            reconnects: stats.connects.load(Ordering::Relaxed),
        }
    }
}

#[cfg(not(feature = "nats"))]
impl NatsConnection {
    pub async fn new(config: NatsConfig) -> Result<Self> {
        log::warn!("NATS feature not enabled - creating stub connection");
        Ok(Self { config })
    }

    pub async fn publish(&self, subject: &str, _data: &[u8]) -> Result<()> {
        log::debug!("NATS stub: would publish to subject: {}", subject);
        Ok(())
    }

    pub async fn subscribe(&self, subject: &str) -> Result<Vec<crate::agent::Message>> {
        log::debug!("NATS stub: would subscribe to subject: {}", subject);
        Ok(Vec::new())
    }

    pub async fn request(&self, subject: &str, _data: &[u8]) -> Result<Vec<u8>> {
        log::debug!("NATS stub: would send request to subject: {}", subject);
        Ok(Vec::new())
    }

    pub fn is_connected(&self) -> bool {
        false
    }

    pub async fn flush(&self) -> Result<()> {
        log::debug!("NATS stub: flush called");
        Ok(())
    }

    pub async fn drain(&self) -> Result<()> {
        log::debug!("NATS stub: drain called");
        Ok(())
    }

    pub async fn close(&self) -> Result<()> {
        log::debug!("NATS stub: close called");
        Ok(())
    }

    pub fn get_stats(&self) -> ConnectionStats {
        ConnectionStats {
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            reconnects: 0,
        }
    }
}

#[derive(Debug, Clone)]
pub struct ConnectionStats {
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub reconnects: u64,
}

// Helper trait for better error handling
#[cfg(feature = "nats")]
#[async_trait]
pub trait NatsPublisher {
    async fn publish_json<T: Serialize + Send + Sync>(&self, subject: &str, data: &T) -> Result<()>;
}

#[cfg(feature = "nats")]
#[async_trait]
impl NatsPublisher for NatsConnection {
    async fn publish_json<T: Serialize + Send + Sync>(&self, subject: &str, data: &T) -> Result<()> {
        let json_data = serde_json::to_vec(data)?;
        self.publish(subject, &json_data).await
    }
}

#[cfg(not(feature = "nats"))]
pub trait NatsPublisher {
    fn publish_json<T: Serialize + Send + Sync>(&self, subject: &str, data: &T) -> Result<()>;
}

#[cfg(not(feature = "nats"))]
impl NatsPublisher for NatsConnection {
    fn publish_json<T: Serialize + Send + Sync>(&self, subject: &str, _data: &T) -> Result<()> {
        log::debug!("NATS stub: would publish JSON to subject: {}", subject);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_nats_config_default() {
        let config = NatsConfig::default();
        assert_eq!(config.url, "nats://localhost:4222");
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_reconnects, Some(10));
        assert_eq!(config.reconnect_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_nats_config_custom() {
        let config = NatsConfig {
            url: "nats://custom:4222".to_string(),
            timeout: Duration::from_secs(5),
            max_reconnects: Some(5),
            reconnect_delay: Duration::from_secs(2),
        };
        assert_eq!(config.url, "nats://custom:4222");
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.max_reconnects, Some(5));
        assert_eq!(config.reconnect_delay, Duration::from_secs(2));
    }

    // Integration tests would require a running NATS server
    // Uncomment these when you have a NATS server running for testing
    
    // #[tokio::test]
    // async fn test_nats_connection() {
    //     let config = NatsConfig::default();
    //     let connection = NatsConnection::new(config).await;
    //     assert!(connection.is_ok());
    // }
    
    // #[tokio::test]
    // async fn test_publish_subscribe() {
    //     let config = NatsConfig::default();
    //     let connection = NatsConnection::new(config).await.unwrap();
    //     
    //     let test_data = b"test message";
    //     connection.publish("test.subject", test_data).await.unwrap();
    //     
    //     let messages = connection.subscribe("test.subject").await.unwrap();
    //     // Note: This test may be flaky due to timing
    // }
}