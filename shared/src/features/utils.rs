use crux_core::{Command, render::render};
use partially::Partial;

use crate::{CruxContext, Effect, Event, Model, PartialModel};

pub fn handle_update_model(
    model: &mut Model,
    partial_model: Box<PartialModel>,
) -> Command<Effect, Event> {
    model.apply_some(*partial_model);
    render()
}

pub fn update_model(ctx: &CruxContext, model: PartialModel) {
    ctx.send_event(Event::UpdateModel(Box::new(model)));
}
