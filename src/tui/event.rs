use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// Configuration for the event handler
pub struct EventConfig {
    /// The tick rate (how often to check for events)
    pub tick_rate: Duration,
}

/// Event handler for the TUI
pub struct EventHandler {
    /// Event receiver
    receiver: mpsc::Receiver<CrosstermEvent>,
    /// Event sender
    _sender: mpsc::Sender<CrosstermEvent>,
}

impl EventHandler {
    /// Create a new event handler with the given configuration
    pub fn new(config: EventConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        let tick_rate = config.tick_rate;
        
        // Spawn a thread to listen for events
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            loop {
                // Poll for events with a timeout
                let timeout = tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                
                if event::poll(timeout).expect("Failed to poll for events") {
                    if let Ok(event) = event::read() {
                        if sender.send(event).is_err() {
                            // Channel closed, exit thread
                            break;
                        }
                    }
                }
                
                // Check if it's time to send a tick event
                if last_tick.elapsed() >= tick_rate {
                    if sender.send(CrosstermEvent::Tick).is_err() {
                        // Channel closed, exit thread
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });
        
        Self {
            receiver,
            _sender: sender,
        }
    }
    
    /// Get the next event
    pub fn next(&self) -> Result<CrosstermEvent> {
        Ok(self.receiver.recv()?)
    }
}

/// Extend CrosstermEvent with a Tick variant
pub enum Event {
    Key(KeyEvent),
    Tick,
}

/// Add Tick variant to CrosstermEvent
impl From<CrosstermEvent> for Event {
    fn from(event: CrosstermEvent) -> Self {
        match event {
            CrosstermEvent::Key(key) => Event::Key(key),
            _ => Event::Tick,
        }
    }
} 