use std::sync::mpsc::{self, Receiver, RecvError, Sender};
use std::thread;
use std::time::{Duration, Instant};

use anyhow::Result;
use crossterm::event::{self, Event as CrosstermEvent, KeyEvent};

/// Event handler
pub struct EventHandler {
    /// Event receiver
    receiver: Receiver<AppEvent>,
}

/// Application events
pub enum AppEvent {
    /// Terminal events (key presses, mouse events, etc.)
    Input(CrosstermEvent),
    /// Timer tick events
    Tick,
}

/// Event loop configuration
pub struct EventConfig {
    /// Tick rate
    pub tick_rate: Duration,
}

impl EventHandler {
    /// Create a new event handler
    pub fn new(config: EventConfig) -> Self {
        let (sender, receiver) = mpsc::channel();
        
        // Start the event loop in a separate thread
        thread::spawn(move || {
            let mut last_tick = Instant::now();
            
            loop {
                // Calculate timeout
                let timeout = config.tick_rate
                    .checked_sub(last_tick.elapsed())
                    .unwrap_or_else(|| Duration::from_secs(0));
                
                // Poll for events
                if event::poll(timeout).unwrap() {
                    // If we have an event, send it to the channel
                    if let Ok(event) = event::read() {
                        if sender.send(AppEvent::Input(event)).is_err() {
                            break;
                        }
                    }
                }
                
                // Check if it's time for a tick
                if last_tick.elapsed() >= config.tick_rate {
                    if sender.send(AppEvent::Tick).is_err() {
                        break;
                    }
                    last_tick = Instant::now();
                }
            }
        });
        
        Self { receiver }
    }
    
    /// Wait for the next event
    pub fn next(&self) -> Result<CrosstermEvent, RecvError> {
        loop {
            match self.receiver.recv()? {
                AppEvent::Input(event) => return Ok(event),
                AppEvent::Tick => {
                    // Ignore tick events when waiting for input
                }
            }
        }
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