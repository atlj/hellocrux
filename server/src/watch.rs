pub fn new_watcher_receiver_pair<Signal, Data>(
    init: Data,
) -> (SignalWatcher<Signal, Data>, SignalReceiver<Signal, Data>) {
    let (signal_sender, signal_receiver) = tokio::sync::mpsc::channel(100);
    let (updater, data) = tokio::sync::watch::channel(init);
    (
        SignalWatcher {
            signal_sender,
            data,
        },
        SignalReceiver {
            signal_receiver,
            updater,
        },
    )
}

pub struct SignalWatcher<Signal, Data> {
    pub signal_sender: tokio::sync::mpsc::Sender<Signal>,
    pub data: tokio::sync::watch::Receiver<Data>,
}

impl<Signal, Data> Clone for SignalWatcher<Signal, Data> {
    fn clone(&self) -> Self {
        Self {
            signal_sender: self.signal_sender.clone(),
            data: self.data.clone(),
        }
    }
}

pub struct SignalReceiver<Signal, Data> {
    pub signal_receiver: tokio::sync::mpsc::Receiver<Signal>,
    pub updater: tokio::sync::watch::Sender<Data>,
}

#[derive(Clone)]
pub struct Watcher<Data> {
    pub updater: tokio::sync::watch::Sender<Data>,
    pub data: tokio::sync::watch::Receiver<Data>,
}

impl<Data> Watcher<Data> {
    pub fn new(initial: Data) -> Watcher<Data> {
        let (updater, data) = tokio::sync::watch::channel(initial);
        Watcher { updater, data }
    }
}
