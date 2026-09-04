// src/app/handlers.rs
use crate::models::{AppEvent, DiscordMessage, MessageStatus};
use chrono::Local;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use tokio::sync::mpsc::Sender;

impl crate::app::state::AppState {
    pub async fn handle_event(
        &mut self,
        event: AppEvent,
        tx: &Sender<AppEvent>,
    ) -> bool {
        match event {
            AppEvent::SetSelfUsername(username) => {
                if !username.is_empty() {
                    self.self_username = username;
                }
            }
            AppEvent::IncomingMessage(m) => {
                if m.nonce.starts_with("err-") {
                    self.messages.push(m);
                } else if !self
                    .messages
                    .iter()
                    .any(|x| x.nonce == m.nonce && !m.nonce.is_empty())
                {
                    self.messages.push(m);
                }
            }
            AppEvent::MessageSent { nonce, timestamp } => {
                if let Some(m) = self.messages.iter_mut().find(|x| x.nonce == nonce) {
                    m.status = MessageStatus::Delivered;
                    
                    if let Some(stripped_nonce) = nonce.strip_prefix("n-") {
                        if let Ok(creation_ms) = stripped_nonce.parse::<i64>() {
                            let current_ms = chrono::Utc::now().timestamp_millis();
                            let total_diff_ms = (current_ms - creation_ms).max(0);
                            
                            m.timestamp = format!(
                                "{} | {}ms",
                                Local::now().format("%H:%M:%S%.3f"),
                                total_diff_ms
                            );
                        }
                    } else if !timestamp.is_empty() {
                        m.timestamp = timestamp;
                    }
                    
                    self.failed_nonces.retain(|x| x != &nonce);
                }
            }
            AppEvent::MessageFailed { nonce } => {
                if let Some(m) = self.messages.iter_mut().find(|x| x.nonce == nonce) {
                    m.status = MessageStatus::Failed;
                }
                if !self.failed_nonces.contains(&nonce) {
                    self.failed_nonces.push(nonce);
                }
            }
            AppEvent::GatewayClosed => {
                self.messages.push(DiscordMessage {
                    nonce: "err-close".into(),
                    author: "System".into(),
                    content: "⚠️ WebSocket closed. Reconnecting...".into(),
                    timestamp: Local::now().format("%H:%M:%S%.3f").to_string(),
                    status: MessageStatus::Failed,
                });
            }
            AppEvent::Terminal(Event::Key(k))
                if k.kind == crossterm::event::KeyEventKind::Press =>
            {
                if k.code == KeyCode::Char('c') && k.modifiers.contains(KeyModifiers::CONTROL) {
                    return true;
                }

                match k.code {
                    KeyCode::Tab => {
                        if let Some(last_failed_nonce) = self.failed_nonces.last().cloned() {
                            let (text, found) = if let Some(m) = self.messages.iter_mut().find(|x| x.nonce == last_failed_nonce) {
                                m.status = MessageStatus::Sending;
                                m.timestamp = format!("{} | ...", Local::now().format("%H:%M:%S%.3f"));
                                (m.content.clone(), true)
                            } else {
                                (String::new(), false)
                            };

                            if found {
                                let _ = tx.send(AppEvent::HttpSendChat {
                                    nonce: last_failed_nonce,
                                    text,
                                }).await;
                            }
                        }
                    }
                    KeyCode::Char(c) => {
                        self.input_text.push(c);
                        
                        let trigger = match self.last_typing_sent {
                            Some(last_sent) => last_sent.elapsed() >= std::time::Duration::from_secs(7),
                            None => true,
                        };

                        if trigger && !self.target_channel_id.is_empty() {
                            self.last_typing_sent = Some(std::time::Instant::now());
                            let _ = tx.send(AppEvent::HttpTriggerTyping).await;
                        }
                    }
                    KeyCode::Backspace => {
                        self.input_text.pop();
                    }
                    KeyCode::Enter if !self.input_text.is_empty() => {
                        self.last_typing_sent = None;
                        let text = std::mem::take(&mut self.input_text);
                        let creation_ms = chrono::Utc::now().timestamp_millis();
                        let nonce = format!("n-{}", creation_ms);
                        let current_time_str = Local::now().format("%H:%M:%S%.3f").to_string();

                        self.messages.push(DiscordMessage {
                            nonce: nonce.clone(),
                            author: self.self_username.clone(),
                            content: text.clone(),
                            timestamp: format!("{} | ...", current_time_str),
                            status: MessageStatus::Sending,
                        });

                        let _ = tx.send(AppEvent::HttpSendChat { nonce, text }).await;
                    }
                    _ => {}
                }
            }
            _ => {}
        }
        false
    }
}
