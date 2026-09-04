// src/app/state.rs
use crate::models::DiscordMessage;
use ratatui::widgets::ListState;
use std::collections::HashMap;
use std::time::Instant;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ActiveModal {
    None,
    LogoutPrompt,
    SwitchChannelPrompt,
}

pub struct AppState {
    pub token: String,
    pub target_channel_id: String,
    pub self_username: String,
    pub messages: Vec<DiscordMessage>,
    pub input_text: String,
    pub modal_input: String,
    pub active_modal: ActiveModal,
    pub list_state: ListState,
    pub failed_nonces: Vec<String>,
    pub last_typing_sent: Option<Instant>,
    pub outbound_timers: HashMap<String, Instant>,
    pub gateway_rtt_ms: Option<u64>,
    pub clock_offset_ms: Option<i64>,
    pub show_timestamp: bool,
    pub show_latency: bool,
    pub scroll_offset: usize,
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
            modal_input: String::new(),
            active_modal: ActiveModal::None,
            list_state,
            failed_nonces: Vec::new(),
            last_typing_sent: None,
            outbound_timers: HashMap::new(),
            gateway_rtt_ms: None,
            clock_offset_ms: None,
            show_timestamp: true,
            show_latency: true,
            scroll_offset: 0,
        }
    }
}
