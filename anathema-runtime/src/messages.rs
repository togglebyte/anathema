use std::any::Any;

use anathema_widgets::components::ComponentId;
use flume::SendError;

pub struct ViewMessage {
    pub(super) payload: Box<dyn Any + Send + Sync>,
    pub(super) recipient: ComponentId,
}

pub struct Emitter(pub(crate) flume::Sender<ViewMessage>);

impl Emitter {
    pub fn emit<T: 'static + Send + Sync>(
        &self,
        value: T,
        component_id: impl Into<ComponentId>,
    ) -> Result<(), SendError<ViewMessage>> {
        let msg = ViewMessage {
            payload: Box::new(value),
            recipient: component_id.into(),
        };
        self.0.send(msg)
    }

    pub async fn emit_async<T: 'static + Send + Sync>(
        &self,
        value: T,
        component_id: ComponentId,
    ) -> Result<(), SendError<ViewMessage>> {
        let msg = ViewMessage {
            payload: Box::new(value),
            recipient: component_id,
        };
        self.0.send_async(msg).await
    }
}
