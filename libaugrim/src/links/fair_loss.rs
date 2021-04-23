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

use crate::error::InternalError;

use super::{Link, Receiver, Sender};

pub struct FairLossPointToPointLink<P, M> {
    send_via: Box<dyn Sender<P, M>>,
    receive_into: Box<dyn Receiver<P, M>>,
}

impl<P, M> FairLossPointToPointLink<P, M> {
    pub fn new<S, R>(send_via: S, receive_into: R) -> Self
    where
        S: Sender<P, M> + 'static,
        R: Receiver<P, M> + 'static,
    {
        Self::new_with_boxed(Box::new(send_via), Box::new(receive_into))
    }

    pub(crate) fn new_with_boxed(
        send_via: Box<dyn Sender<P, M>>,
        receive_into: Box<dyn Receiver<P, M>>,
    ) -> Self {
        Self {
            send_via,
            receive_into,
        }
    }
}

impl<P, M> Link<P, M> for FairLossPointToPointLink<P, M> {
    type Sender = FairLossPointToPointSender<P, M>;
    type Receiver = FairLossPointToPointReceiver<P, M>;

    fn link(self) -> (Self::Sender, Self::Receiver) {
        (
            FairLossPointToPointSender {
                inner: self.send_via,
            },
            FairLossPointToPointReceiver {
                inner: self.receive_into,
            },
        )
    }
}

pub struct FairLossPointToPointSender<P, M> {
    inner: Box<dyn Sender<P, M>>,
}

impl<P, M> Sender<P, M> for FairLossPointToPointSender<P, M> {
    fn send(&self, process: P, message: M) -> Result<(), InternalError> {
        self.inner.send(process, message)
    }
}

pub struct FairLossPointToPointReceiver<P, M> {
    inner: Box<dyn Receiver<P, M>>,
}

impl<P, M> Receiver<P, M> for FairLossPointToPointReceiver<P, M> {
    fn deliver(&self, process: P, message: M) -> Result<(), InternalError> {
        self.inner.deliver(process, message)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use std::error::Error;
    use std::sync::mpsc::{channel, Sender as MpscSender, TryRecvError};

    #[test]
    fn fair_loss_point_to_point_send() -> Result<(), Box<dyn Error>> {
        let (tx, rx) = channel();
        let link = FairLossPointToPointLink::new(ChanSender::new(tx), SinkReceiver);

        let (sender, _) = link.link();

        sender.send("AlphaProcess".into(), "Test Message".into())?;

        let sent = rx.try_recv()?;

        assert_eq!(("AlphaProcess".into(), "Test Message".into()), sent);

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
