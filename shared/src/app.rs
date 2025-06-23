use crux_core::{
    App, Command,
    macros::effect,
    render::{RenderOperation, render},
};
use serde::{Deserialize, Serialize};

use crate::{
    capabilities::delay::random_delay,
    delay::DelayOperation,
    storage::{StorageOperation, get, store},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum Event {
    Startup,
    DelayedIncrement,
    Increment,
    Decrement,
    Reset,
    Set(isize),
    StoreResult(Option<String>),
}

#[effect(typegen)]
pub enum Effect {
    Render(RenderOperation),
    Delay(DelayOperation),
    Store(StorageOperation),
}

#[derive(Default)]
pub struct Model {
    count: isize,
    stored_value: String,
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
            Event::Startup => render(),
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
            Event::Reset => get("dummy".to_string()).then_send(Event::StoreResult),
            Event::Decrement => Command::new(|ctx| async move {
                store::<Effect, Event>("dummy".to_string(), "my dummy value from rust".to_string())
                    .into_future(ctx)
                    .await;
            }),
            Event::StoreResult(stored_value) => {
                model.stored_value = match stored_value {
                    None => "No stored value".to_string(),
                    Some(value) => format!("Stored value is: {value}"),
                };
                render()
            }
            Event::Set(_) => todo!(),
        }
    }

    fn view(&self, model: &Self::Model) -> Self::ViewModel {
        ViewModel {
            count: model.stored_value.clone(),
        }
    }
}
