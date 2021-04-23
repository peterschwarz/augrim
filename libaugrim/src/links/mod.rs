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

#[cfg(feature = "links-fair-loss")]
pub mod fair_loss;

use crate::error::InternalError;

pub trait Link<P, M> {
    type Sender: Sender<P, M>;
    type Receiver: Receiver<P, M>;

    fn link(self) -> (Self::Sender, Self::Receiver);
}

pub trait Sender<P, M> {
    fn send(&self, process: P, message: M) -> Result<(), InternalError>;
}

impl<T, P, M> Sender<P, M> for Box<T>
where
    T: Sender<P, M>,
{
    fn send(&self, process: P, message: M) -> Result<(), InternalError> {
        (**self).send(process, message)
    }
}

pub trait Receiver<P, M> {
    fn deliver(&self, process: P, message: M) -> Result<(), InternalError>;
}

impl<T, P, M> Receiver<P, M> for Box<T>
where
    T: Receiver<P, M>,
{
    fn deliver(&self, process: P, message: M) -> Result<(), InternalError> {
        (**self).deliver(process, message)
    }
}
