use core::fmt;
use std::{pin::Pin, time::Duration};

use crossterm::event::{Event as CrosstermEvent, EventStream as CrosstermEventStream};
use futures::{Stream, StreamExt};
use tokio::time::interval;
use tokio_stream::{StreamMap, wrappers::IntervalStream};

pub struct EventStream {
    streams: StreamMap<StreamName, Pin<Box<dyn Stream<Item = Event>>>>,
}

impl fmt::Debug for EventStream {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Events").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq, Hash)]
enum StreamName {
    SensorRefresh,
    Crossterm,
}

pub enum Event {
    Init,
    Quit,
    Error,
    Closed,
    Tick,
    KeyRefresh,
    Render,
    SensorRefresh,
    Crossterm(CrosstermEvent),
}

impl EventStream {
    pub fn new() -> Self {
        Self {
            streams: StreamMap::from_iter([
                (StreamName::SensorRefresh, sensor_refresh_stream()),
                (StreamName::Crossterm, crossterm_stream()),
            ]),
        }
    }

    pub async fn next(&mut self) -> Option<Event> {
        self.streams.next().await.map(|(_name, event)| event)
    }
}

fn sensor_refresh_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    let sensor_refresh_delay = Duration::from_millis(1000);
    let sensor_refresh_interval = interval(sensor_refresh_delay);
    Box::pin(IntervalStream::new(sensor_refresh_interval).map(|_| Event::SensorRefresh))
}

fn crossterm_stream() -> Pin<Box<dyn Stream<Item = Event>>> {
    Box::pin(
        CrosstermEventStream::new()
            .fuse()
            .filter_map(|event| async move {
                match event {
                    Ok(event) => Some(Event::Crossterm(event)),
                    Err(_) => Some(Event::Error),
                    _ => None,
                }
            }),
    )
}
