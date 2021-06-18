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

/// This module contains traits for fair-loss links[1].
///
/// A fair-loss link[1] is a connection between a sender and a receipient which provides minimal
/// delivery guarentees. Messages may be dropped by the link, but if a message is retransmitted it
/// will eventually be delivered.
///
/// 1. For a full explanation of fair-loss links, see Cachin, Guerraoui, and Rodrigues, Reliable
///    and Secure Distributed Programming, 2nd ed., 2.4.2.

use crate::error::InternalError;
use crate::message::Message;
use crate::process::Process;

pub trait FairLossSender<P, M>
where
    P: Process,
    M: Message,
{
    fn send(to_process: &P, message: M) -> Result<(), InternalError>;
}

pub trait FairLossReceiver<P, M>
where
    P: Process,
    M: Message,
{
    fn deliver(from_process: &P, message: M) -> Result<(), InternalError>;
}
