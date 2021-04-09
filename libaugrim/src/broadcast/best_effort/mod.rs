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

/// p75
use crate::error::InternalError;
use crate::network::NetworkSender;

struct BestEffortBroadcastSender<PROCESS, MESSAGE, NETWORK>
where
    NETWORK: NetworkSender,
{
    network_sender: NETWORK,
    processes: Vec<PROCESS>,
}

impl<PROCESS, MESSAGE, NETWORK> BestEffortBroadcastSender<PROCESS, MESSAGE, NETWORK> {
    fn new(network_sender: NETWORK, processes: Vec<PROCESS>) -> Self {}

    fn broadcast(&self, message: MESSAGE) -> Result<(), InternalError> {
        for process in self.processes {
            self.network_sender.send(process, message)?;
        }
        Ok(())
    }
}

pub trait BestEffortBroadcastReceiver<PROCESS, MESSAGE> {
    fn deliver(process: PROCESS, message: MESSAGE) -> Result<(), InternalError>;
}
