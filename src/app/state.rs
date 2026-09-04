// src/app/state.rs
use crate::models::DiscordMessage;
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::time::Instant;

pub struct AppState {
    pub token: String,
    pub target_channel_id: String,
    pub self_username: String,
    pub messages: Vec<DiscordMessage>,
    pub input_text: String,
    pub list_state: ListState,
    pub failed_nonces: Vec<String>,
    pub last_typing_sent: Option<Instant>,
    pub outbound_timers: HashMap<String, Instant>,
    pub gateway_rtt_ms: Option<u64>,
    pub clock_offset_ms: Option<i64>,
}

impl AppState {
    pub fn new(token: String) -> Self {
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        Self {
            token,
            target_channel_id: String::new(),
            self_username: "You".to_string(),
            messages: Vec::new(),
            input_text: String::new(),
            list_state,
            failed_nonces: Vec::new(),
            last_typing_sent: None,
            outbound_timers: HashMap::new(),
            gateway_rtt_ms: None,
            clock_offset_ms: None,
        }
    }
}
