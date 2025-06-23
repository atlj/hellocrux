use crux_core::{Command, Request, capability::Operation, command::RequestBuilder};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum DelayOperation {
    Random(usize, usize),
    Delay(usize),
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub enum DelayOutput {
    Random(usize),
    TimeUp,
}

impl Operation for DelayOperation {
    type Output = DelayOutput;
}

/// Request a delay for the specified number of milliseconds.
#[must_use]
pub fn random_delay<Effect, Event>(
    min: usize,
    max: usize,
) -> RequestBuilder<Effect, Event, impl Future<Output = DelayOutput>>
where
    Effect: Send + From<Request<DelayOperation>> + 'static,
    Event: Send + 'static,
{
    assert!(min <= max, "min must be less than or equal to max");

    Command::request_from_shell(DelayOperation::Random(min, max)).then_request(|response| {
        let DelayOutput::Random(millis) = response else {
            panic!("Expected millis");
        };

        Command::request_from_shell(DelayOperation::Delay(millis))
    })
}

/// Request a delay for the specified number of milliseconds.
#[must_use]
pub fn delay<Effect, Event>(
    millis: usize,
) -> RequestBuilder<Effect, Event, impl Future<Output = DelayOutput>>
where
    Effect: Send + From<Request<DelayOperation>> + 'static,
    Event: Send + 'static,
{
    Command::request_from_shell(DelayOperation::Delay(millis))
}
