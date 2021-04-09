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

pub struct InternalError;

use crate::broadcast::{BestEffortBroadcastReceiver, BestEffortBroadcastSender};

// p50
trait PerfectFailureDetectorReceiver<P> {
    fn crash(&mut self, process: P) -> Result<(), InternalError>;
}

// p205
trait ConsensusProposer<V> {
    fn propose(value: V) -> Result<(), InternalError>;
}

// p205
trait Consensus<V> {
    fn propose(value: V) -> Result<(), InternalError>;
}

// p206
pub enum FloodingConsensusMessage<V, PROPOSAL> {
    Decided {
        value: V,
    },
    Proposal {
        round: u64,
        proposals: Vec<PROPOSAL>,
    },
}

// p206
pub struct FloodingConsensus<D, P, S, V, PROPOSAL>
where
    D: PerfectFailureDetectorReceiver<P>,
    S: BestEffortBroadcastSender<FloodingConsensusMessage<V, PROPOSAL>>,
{
    correct: Vec<P>,
    round: u64,
    decision: Option<V>,
    received_from: Vec<Vec<P>>,
    proposals: Vec<Vec<PROPOSAL>>,

    detector: D,
    sender: S,
    process_phantom: std::marker::PhantomData<P>,
    value_phantom: std::marker::PhantomData<V>,
    proposal_phantom: std::marker::PhantomData<PROPOSAL>,
}

impl<D, P, S, V, PROPOSAL> FloodingConsensus<D, P, S, V, PROPOSAL>
where
    D: PerfectFailureDetectorReceiver<P>,
    S: BestEffortBroadcastSender<FloodingConsensusMessage<V, PROPOSAL>>,
    P: Clone,
{
    pub fn new(processes: Vec<P>, detector: D, sender: S) -> Self {
        FloodingConsensus {
            correct: processes.clone(),
            round: 1,
            decision: None,
            received_from: vec![processes],
            proposals: vec![],
            detector,
            sender,
            process_phantom: std::marker::PhantomData,
            value_phantom: std::marker::PhantomData,
            proposal_phantom: std::marker::PhantomData,
        }
    }
}

impl<D, P, S, V, PROPOSAL> Consensus<V> for FloodingConsensus<D, P, S, V, PROPOSAL>
where
    D: PerfectFailureDetectorReceiver<P>,
    S: BestEffortBroadcastSender<FloodingConsensusMessage<V, PROPOSAL>>,
{
    fn propose(value: V) -> Result<(), InternalError> {
        unimplemented!()
    }
}

impl<D, P, S, V, PROPOSAL> PerfectFailureDetectorReceiver<P>
    for FloodingConsensus<D, P, S, V, PROPOSAL>
where
    D: PerfectFailureDetectorReceiver<P>,
    S: BestEffortBroadcastSender<FloodingConsensusMessage<V, PROPOSAL>>,
    P: Eq,
{
    fn crash(&mut self, process: P) -> Result<(), InternalError> {
        match self.correct.iter().position(|p| *p == process) {
            Some(index) => {
                self.correct.remove(index);
            }
            None => (),
        };
        Ok(())
    }
}
