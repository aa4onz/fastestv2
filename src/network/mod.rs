// src/network/mod.rs
pub mod gateway;
pub mod http;

use crate::app::state::AppState;
use crate::models::AppEvent;
use crate::network::http::DiscordHttpClient;
use crossterm::event::EventStream;
use futures_util::StreamExt;
use std::sync::Arc;
use tokio::sync::{mpsc, mpsc::Sender, Mutex};

pub fn spawn_network_handlers(
    app_state: Arc<Mutex<AppState>>, 
    event_tx: Sender<AppEvent>, 
    http_client: reqwest::Client,
    net_rx: mpsc::Receiver<AppEvent>,
) {
    let token = {
        let state = app_state.try_lock().expect("Failed to lock state at initialization");
        state.token.clone()
    };

    let client_wrapper = Arc::new(DiscordHttpClient::new(http_client.clone(), token));

    // 1. Zero-latency async event stream for terminal key/mouse inputs
    let input_tx = event_tx.clone();
    tokio::spawn(async move {
        let mut reader = EventStream::new();
        while let Some(Ok(ev)) = reader.next().await {
            if input_tx.send(AppEvent::Terminal(ev)).await.is_err() {
                break;
            }
        }
    });

    // 2. Fire the secure WebSocket Listener loop
    tokio::spawn(gateway::run_gateway_loop(Arc::clone(&app_state), event_tx.clone()));

    // 3. Keep-alive HTTP connection warming loop (pings Discord API every 10s to keep HTTP/2 pool warm)
    let ping_client = Arc::clone(&client_wrapper);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(std::time::Duration::from_secs(10));
        loop {
            interval.tick().await;
            let _ = ping_client.client.get("https://discord.com/api/v10/gateway")
                .header("Authorization", &ping_client.token)
                .send()
                .await;
        }
    });

    // 4. ⚡ NON-BLOCKING CONCURRENT OUTBOUND HTTP WORKER PIPELINE
    let worker_tx = event_tx.clone();
    let worker_state = Arc::clone(&app_state);
    let mut outbound_rx = net_rx;
    
    tokio::spawn(async move {
        while let Some(job) = outbound_rx.recv().await {
            let client = Arc::clone(&client_wrapper);
            let tx = worker_tx.clone();
            let channel_id = worker_state.lock().await.target_channel_id.clone();

            tokio::spawn(async move {
                match job {
                    AppEvent::FetchChannelHistory(cid) => {
                        if let Ok(msgs) = client.fetch_messages(&cid, 50).await {
                            let _ = tx.send(AppEvent::LoadChannelHistory(msgs)).await;
                        }
                    }
                    AppEvent::HttpTriggerTyping => {
                        let _ = client.send_typing(&channel_id).await;
                    }
                    AppEvent::HttpSendChat { nonce, text } => {
                        let res = client.send_message(&channel_id, &text, &nonce).await;

                        if let Ok(resp) = res {
                            if resp.status().is_success() {
                                let _ = tx.send(AppEvent::MessageSent {
                                    nonce,
                                    timestamp: String::new(),
                                }).await;
                            } else {
                                let raw_err = resp.text().await.unwrap_or_else(|_| "Rejected".to_string());
                                let parsed_err = if let Some(idx) = raw_err.find("message\":") {
                                    raw_err[idx + 9..].chars().filter(|c| *c != '"' && *c != '}' && *c != '{').take(80).collect::<String>()
                                } else {
                                    raw_err.chars().filter(|c| !c.is_control() && *c != '{' && *c != '}' && *c != '"').take(80).collect::<String>()
                                };
                                let combined_err_string = format!(
                                    "{} | ❌ {}",
                                    chrono::Local::now().format("%H:%M:%S%.3f"),
                                    parsed_err.trim()
                                );

                                let _ = tx.send(AppEvent::MessageSent {
                                    nonce: nonce.clone(),
                                    timestamp: combined_err_string,
                                }).await;
                                let _ = tx.send(AppEvent::MessageFailed { nonce }).await;
                            }
                        } else {
                            let _ = tx.send(AppEvent::MessageFailed { nonce }).await;
                        }
                    }
                    _ => {}
                }
            });
        }
    });
}
