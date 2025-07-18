use crux_core::{Command, render::render};
use partially::Partial;

use crate::{Effect, Event, Model, PartialModel};

pub fn handle_update_model(
    model: &mut Model,
    partial_model: PartialModel,
) -> Command<Effect, Event> {
    model.apply_some(partial_model);
    render()
}
