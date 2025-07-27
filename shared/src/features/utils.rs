use crux_core::{Command, render::render};
use partially::Partial;

use crate::{
    CruxContext, Effect, Event, Model, PartialModel,
    capabilities::navigation::{self, Screen},
};

pub fn handle_update_model(
    model: &mut Model,
    partial_model: Box<PartialModel>,
) -> Command<Effect, Event> {
    model.apply_some(*partial_model);
    render()
}

pub fn handle_push_if_necessary(model: &Model, screen: Screen) -> Command<Effect, Event> {
    let current_screen = model.current_screen.clone();
    Command::new(|ctx| async move {
        if current_screen != screen {
            navigation::push(screen).into_future(ctx).await
        }
    })
}

pub fn update_model(ctx: &CruxContext, model: PartialModel) {
    ctx.send_event(Event::UpdateModel(Box::new(model)));
}
