use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
use serde::{Deserialize, Serialize};

use crate::storage::{StorageOperation, get, store};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Startup,
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Store(StorageOperation),
}

#[derive(Default)]
pub struct Model {}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {}

#[derive(Default)]
pub struct CounterApp;

impl App for CounterApp {
    type Event = Event;
    type Model = Model;
    type ViewModel = ViewModel;
    type Effect = Effect;
    type Capabilities = ();

    fn update(
        &self,
        event: Self::Event,
        model: &mut Self::Model,
        _caps: &Self::Capabilities,
    ) -> Command<Self::Effect, Self::Event> {
        match event {
            Event::Startup => render(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {}
    }
}
