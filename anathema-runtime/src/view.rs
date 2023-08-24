use anathema_widget_core::Nodes;
use flume::{Sender, Receiver};

pub trait View {
    type M;

    fn update(&mut self);

    fn chain<B: View>(self, rhs: B) -> StateChain<Self, B> where Self: Sized {
        StateChain {
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

// impl<S, M> View for ViewState<S, M> {
//     fn update(&mut self) {
//     }

// }

pub struct StateChain<A: View, B: View> {
    lhs: A,
    rhs: B,
}

impl<A: View, B: View> View for StateChain<A, B> {
    type M = ();

    fn update(&mut self) {
        self.lhs.update();
        self.rhs.update();
    }
}

struct SignInView {
}

impl View for SignInView {
    type M = String;

    fn update(&mut self) {
        todo!()
    }
}
