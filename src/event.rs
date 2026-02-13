use crossterm::event::{Event as CrosstermEvent, EventStream, KeyEvent, KeyEventKind};
use futures::StreamExt;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Debug)]
pub enum Event {
    Key(KeyEvent),
    Tick,
    Render,
}

pub struct EventHandler {
    rx: mpsc::UnboundedReceiver<Event>,
}

impl EventHandler {
    pub fn new(tick_rate: Duration, render_rate: Duration) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();

        tokio::spawn(async move {
            let mut reader = EventStream::new();
            let mut tick_interval = tokio::time::interval(tick_rate);
            let mut render_interval = tokio::time::interval(render_rate);

            loop {
                tokio::select! {
                    event = reader.next() => {
                        if let Some(Ok(CrosstermEvent::Key(key))) = event {
                            if key.kind == KeyEventKind::Press {
                                if tx.send(Event::Key(key)).is_err() {
                                    return;
                                }
                            }
                        }
                    }
                    _ = tick_interval.tick() => {
                        if tx.send(Event::Tick).is_err() {
                            return;
                        }
                    }
                    _ = render_interval.tick() => {
                        if tx.send(Event::Render).is_err() {
                            return;
                        }
                    }
                }
            }
        });

        Self { rx }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.rx.recv().await
    }
}
