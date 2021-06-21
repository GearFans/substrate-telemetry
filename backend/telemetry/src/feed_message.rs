//! This module provides the messages that will be
//! sent to subscribing feeds.

use serde::ser::{SerializeTuple, Serializer};
use serde::Serialize;
use std::mem;

use crate::node::Node;
use serde_json::to_writer;
use common::types::{
    Address, BlockDetails, BlockHash, BlockNumber, NodeHardware, NodeIO, NodeId, NodeStats,
    Timestamp, NodeDetails,
};

pub trait FeedMessage {
    const ACTION: u8;
}

pub trait FeedMessageWrite: FeedMessage {
    fn write_to_feed(&self, ser: &mut FeedMessageSerializer);
}

impl<T> FeedMessageWrite for T
where
    T: FeedMessage + Serialize,
{
    fn write_to_feed(&self, ser: &mut FeedMessageSerializer) {
        ser.write(self)
    }
}

pub struct FeedMessageSerializer {
    /// Current buffer,
    buffer: Vec<u8>,
}

const BUFCAP: usize = 128;

impl FeedMessageSerializer {
    pub fn new() -> Self {
        Self {
            buffer: Vec::with_capacity(BUFCAP),
        }
    }

    pub fn push<Message>(&mut self, msg: Message)
    where
        Message: FeedMessageWrite,
    {
        let glue = match self.buffer.len() {
            0 => b'[',
            _ => b',',
        };

        self.buffer.push(glue);
        self.write(&Message::ACTION);
        self.buffer.push(b',');
        msg.write_to_feed(self);
    }

    fn write<S>(&mut self, value: &S)
    where
        S: Serialize,
    {
        let _ = to_writer(&mut self.buffer, value);
    }

    pub fn finalize(&mut self) -> Option<Vec<u8>> {
        if self.buffer.is_empty() {
            return None;
        }

        self.buffer.push(b']');

        let bytes = mem::replace(&mut self.buffer, Vec::with_capacity(BUFCAP));

        Some(bytes)
    }
}

macro_rules! actions {
    ($($action:literal: $t:ty,)*) => {
        $(
            impl FeedMessage for $t {
                const ACTION: u8 = $action;
            }
        )*
    }
}

actions! {
     0: Version,
     1: BestBlock,
     2: BestFinalized,
     3: AddedNode<'_>,
     4: RemovedNode,
     5: LocatedNode<'_>,
     6: ImportedBlock<'_>,
     7: FinalizedBlock,
     8: NodeStatsUpdate<'_>,
     9: Hardware<'_>,
    10: TimeSync,
    11: AddedChain<'_>,
    12: RemovedChain<'_>,
    13: SubscribedTo<'_>,
    14: UnsubscribedFrom<'_>,
    15: Pong<'_>,
    16: AfgFinalized,
    17: AfgReceivedPrevote,
    18: AfgReceivedPrecommit,
    19: AfgAuthoritySet,
    20: StaleNode,
    21: NodeIOUpdate<'_>,
}

#[derive(Serialize)]
pub struct Version(pub usize);

#[derive(Serialize)]
pub struct BestBlock(pub BlockNumber, pub Timestamp, pub Option<u64>);

#[derive(Serialize)]
pub struct BestFinalized(pub BlockNumber, pub BlockHash);

pub struct AddedNode<'a>(pub NodeId, pub &'a Node);

#[derive(Serialize)]
pub struct RemovedNode(pub NodeId);

#[derive(Serialize)]
pub struct LocatedNode<'a>(pub NodeId, pub f32, pub f32, pub &'a str);

#[derive(Serialize)]
pub struct ImportedBlock<'a>(pub NodeId, pub &'a BlockDetails);

#[derive(Serialize)]
pub struct FinalizedBlock(pub NodeId, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct NodeStatsUpdate<'a>(pub NodeId, pub &'a NodeStats);

#[derive(Serialize)]
pub struct NodeIOUpdate<'a>(pub NodeId, pub &'a NodeIO);

#[derive(Serialize)]
pub struct Hardware<'a>(pub NodeId, pub &'a NodeHardware);

#[derive(Serialize)]
pub struct TimeSync(pub u64);

#[derive(Serialize)]
pub struct AddedChain<'a>(pub &'a str, pub usize);

#[derive(Serialize)]
pub struct RemovedChain<'a>(pub &'a str);

#[derive(Serialize)]
pub struct SubscribedTo<'a>(pub &'a str);

#[derive(Serialize)]
pub struct UnsubscribedFrom<'a>(pub &'a str);

#[derive(Serialize)]
pub struct Pong<'a>(pub &'a str);

#[derive(Serialize)]
pub struct AfgFinalized(pub Address, pub BlockNumber, pub BlockHash);

#[derive(Serialize)]
pub struct AfgReceivedPrevote(
    pub Address,
    pub BlockNumber,
    pub BlockHash,
    pub Option<Address>,
);

#[derive(Serialize)]
pub struct AfgReceivedPrecommit(
    pub Address,
    pub BlockNumber,
    pub BlockHash,
    pub Option<Address>,
);

#[derive(Serialize)]
pub struct AfgAuthoritySet(
    pub Address,
    pub Address,
    pub Address,
    pub BlockNumber,
    pub BlockHash,
);

#[derive(Serialize)]
pub struct StaleNode(pub NodeId);

impl FeedMessageWrite for AddedNode<'_> {
    fn write_to_feed(&self, ser: &mut FeedMessageSerializer) {
        let AddedNode(nid, node) = self;

        let details = node.details();
        let details = (
            &details.name,
            &details.implementation,
            &details.version,
            &details.validator,
            &details.network_id,
        );

        ser.write(&(
            nid,
            details,
            node.stats(),
            node.io(),
            node.hardware(),
            node.block_details(),
            &node.location(),
            &node.startup_time(),
        ));
    }
}