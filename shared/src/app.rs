use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
use serde::{Deserialize, Serialize};

use crate::{DelayOperation, random_delay};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    DelayedIncrement,
    Increment,
    Decrement,
    Reset,
    Set(isize),
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Delay(DelayOperation),
}

#[derive(Default)]
pub struct Model {
    count: isize,
    update_in_progress: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct ViewModel {
    pub count: String,
}

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
            Event::DelayedIncrement => {
                model.update_in_progress = true;

                render()
                    .then(random_delay::<Effect, Event>(2000, 5000).then_send(|_| Event::Increment))
            }
            Event::Increment => {
                model.update_in_progress = false;
                model.count += 1;
                render()
            }
            Event::Decrement => {
                model.count -= 1;
                render()
            }
            Event::Reset => {
                model.count = 0;
                render()
            }
            Event::Set(value) => {
                model.count = value;
                render()
            }
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            count: if model.update_in_progress {
                "Count is being calculated".to_string()
            } else {
                format!("Count is: {}", model.count)
            },
        }
    }
}
