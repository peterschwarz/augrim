// Copyright 2021 Cargill Incorporated
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

use std::cell::RefCell;
use std::collections::HashSet;
use std::hash::Hash;

use crate::error::InternalError;

use super::fair_loss::{
    FairLossPointToPointLink, FairLossPointToPointReceiver, FairLossPointToPointSender,
};
use super::{Link, Receiver, Sender};

pub struct StubbornPointToPointLink<P, M> {
    send_via: Box<dyn Sender<P, M>>,
    receive_into: Box<dyn Receiver<P, M>>,
}

impl<P, M> StubbornPointToPointLink<P, M> {
    pub fn new<S, R>(send_via: S, receive_into: R) -> Self
    where
        S: Sender<P, M> + 'static,
        R: Receiver<P, M> + 'static,
    {
        Self {
            send_via: Box::new(send_via),
            receive_into: Box::new(receive_into),
        }
    }
}

impl<P, M> Link<P, M> for StubbornPointToPointLink<P, M>
where
    P: Eq + Hash + Clone + 'static,
    M: Eq + Hash + Clone + 'static,
{
    type Sender = StubbornPointToPointSender<P, M>;
    type Receiver = FairLossPointToPointReceiver<P, M>;

    fn link(self) -> (Self::Sender, Self::Receiver) {
        let stubborn_receiver = StubbornPointToPointReceiver {
            inner: self.receive_into,
        };

        let fair_loss =
            FairLossPointToPointLink::new_with_boxed(self.send_via, Box::new(stubborn_receiver));

        let (fair_loss_sender, fair_loss_receiver) = fair_loss.link();

        (
            StubbornPointToPointSender {
                fair_loss_sender,
                sent: RefCell::new(HashSet::new()),
            },
            fair_loss_receiver,
        )
    }
}

pub struct StubbornPointToPointSender<P, M> {
    fair_loss_sender: FairLossPointToPointSender<P, M>,
    sent: RefCell<HashSet<(P, M)>>,
}

impl<P, M> StubbornPointToPointSender<P, M>
where
    P: Eq + Hash + Clone + 'static,
    M: Eq + Hash + Clone + 'static,
{
    pub fn timeout(&self) -> Result<(), InternalError> {
        for (process, message) in self.sent.borrow().iter() {
            self.fair_loss_sender.send(process.clone(), message.clone())?;
        }

        Ok(())
    }
}

impl<P, M> Sender<P, M> for StubbornPointToPointSender<P, M>
where
    P: Eq + Hash + Clone + 'static,
    M: Eq + Hash + Clone + 'static,
{
    fn send(&self, process: P, message: M) -> Result<(), InternalError> {
        self.fair_loss_sender
            .send(process.clone(), message.clone())?;
        self.sent.borrow_mut().insert((process, message));
        Ok(())
    }
}

pub struct StubbornPointToPointReceiver<P, M> {
    inner: Box<dyn Receiver<P, M>>,
}

impl<P, M> Receiver<P, M> for StubbornPointToPointReceiver<P, M> {
    fn deliver(&self, process: P, message: M) -> Result<(), InternalError> {
        self.inner.deliver(process, message)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use std::error::Error;
    use std::sync::mpsc::{channel, Sender as MpscSender, TryRecvError};

    #[test]
    fn stubborn_point_to_point_send() -> Result<(), Box<dyn Error>> {
        let (tx, rx) = channel();
        let link = StubbornPointToPointLink::new(ChanSender::new(tx), SinkReceiver);

        let (sender, _) = link.link();

        sender.send("AlphaProcess".into(), "Test Message".into())?;

        let sent = rx.try_recv()?;
        assert_eq!(("AlphaProcess".into(), "Test Message".into()), sent);

        // ensure it isn't immediately sent again
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        // trigger the timeout event.
        sender.timeout()?;

        // Assert that it was sent again
        let sent = rx.try_recv()?;
        assert_eq!(("AlphaProcess".into(), "Test Message".into()), sent);

        // ensure it isn't immediately sent again
        assert!(matches!(rx.try_recv(), Err(TryRecvError::Empty)));

        Ok(())
    }

    struct ChanSender {
        tx: MpscSender<(String, String)>,
    }

    impl ChanSender {
        fn new(tx: MpscSender<(String, String)>) -> Self {
            Self { tx }
        }
    }

    impl Sender<String, String> for ChanSender {
        fn send(&self, process: String, message: String) -> Result<(), InternalError> {
            self.tx
                .send((process, message))
                .map_err(|e| InternalError::from_source(Box::new(e)))
        }
    }

    struct SinkSender;

    impl Sender<String, String> for SinkSender {
        fn send(&self, _: String, _: String) -> Result<(), InternalError> {
            Ok(())
        }
    }
    /// Receive messages and drops them.
    struct SinkReceiver;

    impl Receiver<String, String> for SinkReceiver {
        fn deliver(&self, _: String, _: String) -> Result<(), InternalError> {
            Ok(())
        }
    }
}
