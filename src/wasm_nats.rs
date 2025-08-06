//! WebSocket-based NATS client for WASM environments
//! 
//! This module provides NATS messaging capabilities in WebAssembly environments
//! by implementing the NATS protocol over WebSocket connections.

#[cfg(feature = "wasm-nats")]
use wasm_bindgen::prelude::*;
#[cfg(feature = "wasm-nats")]
use wasm_bindgen::JsCast;
#[cfg(feature = "wasm-nats")]
use web_sys::{WebSocket, MessageEvent, CloseEvent, ErrorEvent, BinaryType};
#[cfg(feature = "wasm-nats")]
use js_sys::{Uint8Array, ArrayBuffer};
#[cfg(feature = "wasm-nats")]
use std::collections::HashMap;
#[cfg(feature = "wasm-nats")]
use std::sync::{Arc, Mutex};
#[cfg(feature = "wasm-nats")]
use futures::channel::mpsc;
#[cfg(not(feature = "wasm-nats"))]
use futures::channel::mpsc;

use serde::{Deserialize, Serialize};
use std::time::Duration;
use crate::{Result, Error};
#[cfg(feature = "wasm-nats")]
use base64::prelude::*;

/// Configuration for WebSocket NATS connection
#[derive(Debug, Clone)]
pub struct WasmNatsConfig {
    pub websocket_url: String,
    pub timeout: Duration,
    pub max_reconnects: Option<usize>,
    pub reconnect_delay: Duration,
}

impl Default for WasmNatsConfig {
    fn default() -> Self {
        Self {
            // Note: Use WSS for production, WS for local development
            websocket_url: "ws://localhost:8080/nats".to_string(),
            timeout: Duration::from_secs(10),
            max_reconnects: Some(10),
            reconnect_delay: Duration::from_secs(1),
        }
    }
}

/// WebSocket-based NATS connection for WASM environments
#[cfg(feature = "wasm-nats")]
#[derive(Debug)]
pub struct WasmNatsConnection {
    websocket: WebSocket,
    config: WasmNatsConfig,
    message_sender: Arc<Mutex<Option<mpsc::UnboundedSender<Vec<u8>>>>>,
    subscriptions: Arc<Mutex<HashMap<String, mpsc::UnboundedSender<crate::agent::Message>>>>,
    is_connected: Arc<Mutex<bool>>,
}

#[cfg(not(feature = "wasm-nats"))]
#[derive(Debug)]
pub struct WasmNatsConnection {
    config: WasmNatsConfig,
}

#[cfg(feature = "wasm-nats")]
impl WasmNatsConnection {
    /// Create a new WebSocket NATS connection
    pub async fn new(config: WasmNatsConfig) -> Result<Self> {
        let websocket = WebSocket::new(&config.websocket_url)
            .map_err(|e| Error::Custom(format!("Failed to create WebSocket: {:?}", e)))?;
        
        // Set binary type for NATS protocol
        websocket.set_binary_type(BinaryType::Arraybuffer);
        
        let message_sender = Arc::new(Mutex::new(None));
        let subscriptions = Arc::new(Mutex::new(HashMap::new()));
        let is_connected = Arc::new(Mutex::new(false));
        
        let connection = Self {
            websocket,
            config,
            message_sender: message_sender.clone(),
            subscriptions: subscriptions.clone(),
            is_connected: is_connected.clone(),
        };
        
        // Set up WebSocket event handlers
        connection.setup_event_handlers().await?;
        
        Ok(connection)
    }
    
    /// Set up WebSocket event handlers
    async fn setup_event_handlers(&self) -> Result<()> {
        let is_connected = self.is_connected.clone();
        let subscriptions = self.subscriptions.clone();
        
        // On open handler
        let onopen_callback = {
            let is_connected = is_connected.clone();
            Closure::wrap(Box::new(move |_event: web_sys::Event| {
                log::info!("WebSocket NATS connection opened");
                *is_connected.lock().unwrap() = true;
            }) as Box<dyn FnMut(web_sys::Event)>)
        };
        self.websocket.set_onopen(Some(onopen_callback.as_ref().unchecked_ref()));
        onopen_callback.forget();
        
        // On message handler
        let onmessage_callback = {
            let subscriptions = subscriptions.clone();
            Closure::wrap(Box::new(move |event: MessageEvent| {
                if let Ok(array_buffer) = event.data().dyn_into::<ArrayBuffer>() {
                    let uint8_array = Uint8Array::new(&array_buffer);
                    let data = uint8_array.to_vec();
                    
                    // Parse NATS protocol message
                    if let Ok(message) = Self::parse_nats_message(&data) {
                        let subscriptions_guard = subscriptions.lock().unwrap();
                        if let Some(sender) = subscriptions_guard.get(&message.subject) {
                            let agent_message = crate::agent::Message {
                                id: format!("nats_{}", chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)),
                                from: crate::agent::AgentId("nats".to_string()),
                                to: crate::agent::AgentId(message.subject.clone()),
                                payload: serde_json::from_slice(&message.payload)
                                    .unwrap_or_else(|_| serde_json::json!({"raw": base64::prelude::BASE64_STANDARD.encode(&message.payload)})),
                                timestamp: chrono::Utc::now().timestamp() as u64,
                            };
                            
                            if let Err(e) = sender.unbounded_send(agent_message) {
                                log::warn!("Failed to send message to subscriber: {:?}", e);
                            }
                        }
                    }
                }
            }) as Box<dyn FnMut(MessageEvent)>)
        };
        self.websocket.set_onmessage(Some(onmessage_callback.as_ref().unchecked_ref()));
        onmessage_callback.forget();
        
        // On close handler
        let onclose_callback = {
            let is_connected = is_connected.clone();
            Closure::wrap(Box::new(move |event: CloseEvent| {
                log::warn!("WebSocket NATS connection closed: {} - {}", event.code(), event.reason());
                *is_connected.lock().unwrap() = false;
            }) as Box<dyn FnMut(CloseEvent)>)
        };
        self.websocket.set_onclose(Some(onclose_callback.as_ref().unchecked_ref()));
        onclose_callback.forget();
        
        // On error handler
        let onerror_callback = Closure::wrap(Box::new(move |event: ErrorEvent| {
            log::error!("WebSocket NATS connection error: {:?}", event);
        }) as Box<dyn FnMut(ErrorEvent)>);
        self.websocket.set_onerror(Some(onerror_callback.as_ref().unchecked_ref()));
        onerror_callback.forget();
        
        Ok(())
    }
    
    /// Parse NATS protocol message from binary data
    fn parse_nats_message(data: &[u8]) -> Result<NatsMessage> {
        let message_str = String::from_utf8_lossy(data);
        let lines: Vec<&str> = message_str.lines().collect();
        
        if lines.is_empty() {
            return Err(Error::Custom("Empty NATS message".to_string()));
        }
        
        // Parse MSG line: MSG <subject> <sid> [reply-to] <#bytes>
        let first_line_parts: Vec<&str> = lines[0].split_whitespace().collect();
        if first_line_parts.len() < 4 || first_line_parts[0] != "MSG" {
            return Err(Error::Custom("Invalid NATS message format".to_string()));
        }
        
        let subject = first_line_parts[1].to_string();
        let payload_size: usize = first_line_parts.last()
            .and_then(|s| s.parse().ok())
            .ok_or_else(|| Error::Custom("Invalid payload size".to_string()))?;
        
        // Extract payload (should be on the next line(s))
        let payload_start = message_str.find('\n').unwrap_or(0) + 1;
        let payload_end = std::cmp::min(payload_start + payload_size, data.len());
        let payload = data[payload_start..payload_end].to_vec();
        
        Ok(NatsMessage { subject, payload })
    }
    
    /// Publish a message to a NATS subject
    pub async fn publish(&self, subject: &str, data: &[u8]) -> Result<()> {
        if !self.is_connected() {
            return Err(Error::Custom("WebSocket NATS not connected".to_string()));
        }
        
        // Format NATS PUB command: PUB <subject> <#bytes>\r\n<payload>\r\n
        let pub_command = format!("PUB {} {}\r\n", subject, data.len());
        let mut message = pub_command.into_bytes();
        message.extend_from_slice(data);
        message.extend_from_slice(b"\r\n");
        
        // Send binary message through WebSocket
        self.websocket.send_with_u8_array(&message)
            .map_err(|e| Error::Custom(format!("Failed to send WebSocket message: {:?}", e)))?;
        
        log::debug!("Published WebSocket NATS message to subject: {}", subject);
        Ok(())
    }
    
    /// Subscribe to a NATS subject
    pub async fn subscribe(&self, subject: &str) -> Result<mpsc::UnboundedReceiver<crate::agent::Message>> {
        if !self.is_connected() {
            return Err(Error::Custom("WebSocket NATS not connected".to_string()));
        }
        
        let (sender, receiver) = mpsc::unbounded();
        
        // Store subscription
        {
            let mut subscriptions = self.subscriptions.lock().unwrap();
            subscriptions.insert(subject.to_string(), sender);
        }
        
        // Send SUB command: SUB <subject> <sid>\r\n
        let sub_command = format!("SUB {} {}\r\n", subject, subject);
        let message = sub_command.into_bytes();
        
        self.websocket.send_with_u8_array(&message)
            .map_err(|e| Error::Custom(format!("Failed to send subscribe command: {:?}", e)))?;
        
        log::debug!("Subscribed to WebSocket NATS subject: {}", subject);
        Ok(receiver)
    }
    
    /// Check if WebSocket is connected
    pub fn is_connected(&self) -> bool {
        *self.is_connected.lock().unwrap()
    }
    
    /// Get WebSocket ready state
    pub fn ready_state(&self) -> u16 {
        self.websocket.ready_state()
    }
    
    /// Close WebSocket connection
    pub async fn close(&self) -> Result<()> {
        self.websocket.close()
            .map_err(|e| Error::Custom(format!("Failed to close WebSocket: {:?}", e)))?;
        
        log::info!("Closed WebSocket NATS connection");
        Ok(())
    }
    
    /// Get connection statistics (stub for WebSocket)
    pub fn get_stats(&self) -> WasmConnectionStats {
        WasmConnectionStats {
            is_connected: self.is_connected(),
            ready_state: self.ready_state(),
            url: self.config.websocket_url.clone(),
        }
    }
}

#[cfg(not(feature = "wasm-nats"))]
impl WasmNatsConnection {
    pub async fn new(config: WasmNatsConfig) -> Result<Self> {
        log::warn!("WASM NATS feature not enabled - creating stub connection");
        Ok(Self { config })
    }
    
    pub async fn publish(&self, subject: &str, _data: &[u8]) -> Result<()> {
        log::debug!("WASM NATS stub: would publish to subject: {}", subject);
        Ok(())
    }
    
    pub async fn subscribe(&self, subject: &str) -> Result<mpsc::UnboundedReceiver<crate::agent::Message>> {
        log::debug!("WASM NATS stub: would subscribe to subject: {}", subject);
        let (_sender, receiver) = mpsc::unbounded();
        Ok(receiver)
    }
    
    pub fn is_connected(&self) -> bool {
        false
    }
    
    pub fn ready_state(&self) -> u16 {
        3 // CLOSED
    }
    
    pub async fn close(&self) -> Result<()> {
        log::debug!("WASM NATS stub: close called");
        Ok(())
    }
    
    pub fn get_stats(&self) -> WasmConnectionStats {
        WasmConnectionStats {
            is_connected: false,
            ready_state: 3,
            url: self.config.websocket_url.clone(),
        }
    }
}

/// Parsed NATS message structure
#[derive(Debug, Clone)]
struct NatsMessage {
    subject: String,
    payload: Vec<u8>,
}

/// WebSocket NATS connection statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConnectionStats {
    pub is_connected: bool,
    pub ready_state: u16,
    pub url: String,
}

/// Helper trait for JSON publishing over WebSocket NATS
pub trait WasmNatsPublisher {
    fn publish_json<T: Serialize>(&self, subject: &str, data: &T) -> impl std::future::Future<Output = Result<()>>;
}

impl WasmNatsPublisher for WasmNatsConnection {
    async fn publish_json<T: Serialize>(&self, subject: &str, data: &T) -> Result<()> {
        let json_data = serde_json::to_vec(data)?;
        self.publish(subject, &json_data).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wasm_nats_config_default() {
        let config = WasmNatsConfig::default();
        assert_eq!(config.websocket_url, "ws://localhost:8080/nats");
        assert_eq!(config.timeout, Duration::from_secs(10));
        assert_eq!(config.max_reconnects, Some(10));
        assert_eq!(config.reconnect_delay, Duration::from_secs(1));
    }

    #[test]
    fn test_wasm_nats_config_custom() {
        let config = WasmNatsConfig {
            websocket_url: "wss://nats.example.com/ws".to_string(),
            timeout: Duration::from_secs(5),
            max_reconnects: Some(5),
            reconnect_delay: Duration::from_secs(2),
        };
        assert_eq!(config.websocket_url, "wss://nats.example.com/ws");
        assert_eq!(config.timeout, Duration::from_secs(5));
        assert_eq!(config.max_reconnects, Some(5));
        assert_eq!(config.reconnect_delay, Duration::from_secs(2));
    }

    #[cfg(feature = "wasm-nats")]
    #[test]
    fn test_nats_message_parsing() {
        let test_message = b"MSG test.subject 1 5\r\nhello\r\n";
        let parsed = WasmNatsConnection::parse_nats_message(test_message).unwrap();
        
        assert_eq!(parsed.subject, "test.subject");
        assert_eq!(parsed.payload, b"hello");
    }
}