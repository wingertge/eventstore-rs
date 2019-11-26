use crate::internal::messaging::Msg;
use crate::types::Endpoint;
use futures::future;
use futures::prelude::{ Future, Stream, Sink };
use futures::channel::mpsc;
use futures::sink::SinkExt;
use futures::stream::StreamExt;

pub(crate) async fn discover(
    consumer: mpsc::Receiver<Option<Endpoint>>,
    sender: mpsc::Sender<Msg>,
    endpoint: Endpoint,
) {
    struct State {
        sender: mpsc::Sender<Msg>,
        endpoint: Endpoint,
    }

    let initial =
        State {
            sender,
            endpoint,
        };

    consumer.fold(initial, async move |state, _|
    {
        state.sender
            .clone()
            .send(Msg::Establish(state.endpoint))
            .await;

        state
    }).await;
}
