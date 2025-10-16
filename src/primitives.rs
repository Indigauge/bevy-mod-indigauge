use bevy::prelude::*;

pub mod feedback;
pub mod session;

#[derive(Event, Debug)]
pub enum IndigaugeInitDoneEvent {
  Success,
  Skipped(String),
  Failure(String),
  UnexpectedFailure(String),
}
