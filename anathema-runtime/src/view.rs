use anathema_widget_core::generator::Nodes;
use flume::{Sender, Receiver};

impl View for () {
    type State = ();

    fn update(&mut self) {}

}

pub trait View {
    type State;

    fn update(&mut self);

    fn chain<B: View>(self, rhs: B) -> ViewChain<Self, B> where Self: Sized {
        ViewChain {
            lhs: self,
            rhs,
        }
    }
}

pub struct ViewState<S, M> {
    state: S,
    tx: Sender<M>, 
    rx: Receiver<M>,
    nodes: Nodes,
}

pub struct ViewChain<A: View, B: View> {
    lhs: A,
    rhs: B,
}

impl<A: View, B: View> View for ViewChain<A, B> {
    type State = ();

    fn update(&mut self) {
        self.lhs.update();
        self.rhs.update();
    }
}
