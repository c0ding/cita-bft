// CITA
// Copyright 2016-2017 Cryptape Technologies LLC.

// This program is free software: you can redistribute it
// and/or modify it under the terms of the GNU General Public
// License as published by the Free Software Foundation,
// either version 3 of the License, or (at your option) any
// later version.

// This program is distributed in the hope that it will be
// useful, but WITHOUT ANY WARRANTY; without even the implied
// warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR
// PURPOSE. See the GNU General Public License for more details.

// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <http://www.gnu.org/licenses/>.

use bft_rs::algorithm::Step;
use bft_rs::{Feed, Proposal as BftProposal, Status as BftStatus, Vote as BftVote};
use bincode::deserialize;
use core::collector::SignedVote;
use crypto::{pubkey_to_address, Sign, Signature};
use libproto::blockchain::{Block, RichStatus};
use libproto::consensus::SignedProposal;
use libproto::RawBytes;
use std::convert::TryInto;
use types::{Address, H256};
use util::Hashable;

pub fn extract_bft_proposal(signed_proposal: &SignedProposal) -> BftProposal {
    let signature = signed_proposal.get_signature();
    let proto_proposal = signed_proposal.get_proposal();

    let height = proto_proposal.get_height() as usize;
    let round = proto_proposal.get_round() as usize;

    let signature = Signature::from(signature);
    let message: Vec<u8> = proto_proposal.try_into().unwrap();
    let hash = message.crypt_hash();
    let pub_key = signature.recover(&hash).unwrap();
    let address = pubkey_to_address(&pub_key);

    let mut lock_round = None;
    let lock_votes = if proto_proposal.get_islock() {
        lock_round = Some(proto_proposal.get_lock_round() as usize);
        let mut votes = Vec::new();
        for vote in proto_proposal.get_lock_votes() {
            votes.push(
                BftVote{
                    vote_type: Step::Prevote,
                    height,
                    round,
                    proposal: vote.get_proposal().clone().to_vec(),
                    voter: vote.get_sender().clone().to_vec(),
                }
            );
        }
        Some(votes)
    } else {
        None
    };

    let block = proto_proposal.get_block();
    let block_hash = block.crypt_hash();

    BftProposal{
        height,
        round,
        content: block_hash.0.to_vec(),
        lock_round,
        lock_votes,
        proposer: address.0.to_vec(),
    }
}

pub fn extract_signed_vote(raw_bytes: &RawBytes) -> SignedVote {
    let decoded = deserialize(raw_bytes).unwrap();
    let (message, signature): (Vec<u8>, &[u8]) = decoded;
    let signature = Signature::from(signature);
    let decoded = deserialize(&message[..]).unwrap();
    let (_, _, _, _, proposal):(usize, usize, Step, Address, Option<H256>) = decoded;
    SignedVote{
        proposal,
        signature,
    }
}

pub fn extract_bft_vote(raw_bytes: &RawBytes) -> BftVote {
    let decoded = deserialize(raw_bytes).unwrap();
    let (message, _): (Vec<u8>, &[u8]) = decoded;
    let decoded = deserialize(&message[..]).unwrap();
    let (height, round, step, sender, proposal):(usize, usize, Step, Address, Option<H256>) = decoded;
    let bft_proposal;
    if let Some(proposal) = proposal {
        bft_proposal = proposal.0.to_vec();
    } else {
        bft_proposal = Vec::new();
    }
    BftVote{
        vote_type: step,
        height,
        round,
        proposal: bft_proposal,
        voter: sender.0.to_vec(),
    }
}

pub fn extract_feed(block: &Block) -> Feed {
    let height = block.get_header().get_height() as usize;
    let block_hash = block.crypt_hash();
    Feed{
        height,
        proposal: block_hash.0.to_vec(),
    }
}


pub fn extract_bft_status(rich_status: &RichStatus) -> BftStatus {
    let height = rich_status.height as usize;
    let authorities = rich_status.get_nodes().to_vec();
    BftStatus{
        height,
        interval: Some(rich_status.interval),
        authority_list: authorities,
    }
}