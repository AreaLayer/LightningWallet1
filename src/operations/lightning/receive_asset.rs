use bitcoin::blockdata::script::{Script,Builder};
use bitcoin::blockdata::transaction::{Transaction, EcdsaSighashType};
use bitcoin::util::sighash;
use bitcoin::consensus::encode;

use bitcoin::hashes::Hash;
use bitcoin::hashes::sha256::Hash as Sha256;
use bitcoin::hashes::sha256d::Hash as Sha256d;
use bitcoin::hash_types::{Txid, BlockHash};

use bitcoin::secp256k1::constants::PUBLIC_KEY_SIZE;
use bitcoin::secp256k1::{PublicKey,SecretKey};
use bitcoin::secp256k1::{Secp256k1,ecdsa::Signature};
use bitcoin::secp256k1;

use crate::ln::{PaymentPreimage, PaymentHash};
use crate::ln::features::{ChannelTypeFeatures, InitFeatures};
use crate::ln::msgs;
use crate::ln::msgs::{DecodeError, OptionalField, DataLossProtect};
use crate::ln::script::{self, ShutdownScript};
use crate::ln::channelmanager::{self, CounterpartyForwardingInfo, PendingHTLCStatus, HTLCSource, HTLCFailReason, HTLCFailureMsg, PendingHTLCInfo, RAACommitmentOrder, BREAKDOWN_TIMEOUT, MIN_CLTV_EXPIRY_DELTA, MAX_LOCAL_BREAKDOWN_TIMEOUT};
use crate::ln::chan_utils::{CounterpartyCommitmentSecrets, TxCreationKeys, HTLCOutputInCommitment, htlc_success_tx_weight, htlc_timeout_tx_weight, make_funding_redeemscript, ChannelPublicKeys, CommitmentTransaction, HolderCommitmentTransaction, ChannelTransactionParameters, CounterpartyChannelTransactionParameters, MAX_HTLCS, get_commitment_transaction_number_obscure_factor, ClosingTransaction};
use crate::ln::chan_utils;

se crate::io;
use crate::prelude::*;
use core::{cmp,mem,fmt};
use core::ops::Deref;
#[cfg(any(test, fuzzing, debug_assertions))]
use crate::sync::Mutex;
use bitcoin::hashes::hex::ToHex;

#[cfg(test)]
pub struct ChannelValueStat {
	pub value_to_self_msat: u64,
	pub channel_value_msat: u64,
	pub channel_reserve_msat: u64,
	pub pending_outbound_htlcs_amount_msat: u64,
	pub pending_inbound_htlcs_amount_msat: u64,
	pub holding_cell_outbound_amount_msat: u64,
	pub counterparty_max_htlc_value_in_flight_msat: u64, // outgoing
	pub counterparty_dust_limit_msat: u64,
}

pub struct AvailableBalances {
	/// The amount that would go to us if we close the channel, ignoring any on-chain fees.
	pub balance_msat: u64,
	/// Total amount available for our counterparty to send to us.
	pub inbound_capacity_msat: u64,
	/// Total amount available for us to send to our counterparty.
	pub outbound_capacity_msat: u64,
	/// The maximum value we can assign to the next outbound HTLC
	pub next_outbound_htlc_limit_msat: u64,
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FeeUpdateState {
	// Inbound states mirroring InboundHTLCState
	RemoteAnnounced,
	AwaitingRemoteRevokeToAnnounce,
	// Note that we do not have a AwaitingAnnouncedRemoteRevoke variant here as it is universally
	// handled the same as `Committed`, with the only exception in `InboundHTLCState` being the
	// distinction of when we allow ourselves to forward the HTLC. Because we aren't "forwarding"
	// the fee update anywhere, we can simply consider the fee update `Committed` immediately
	// instead of setting it to AwaitingAnnouncedRemoteRevoke.

	// Outbound state can only be `LocalAnnounced` or `Committed`
	Outbound,
}

enum InboundHTLCRemovalReason {
	FailRelay(msgs::OnionErrorPacket),
	FailMalformed(([u8; 32], u16)),
	Fulfill(PaymentPreimage),
}

enum InboundHTLCState {
	/// Offered by remote, to be included in next local commitment tx. I.e., the remote sent an
	/// update_add_htlc message for this HTLC.
	RemoteAnnounced(PendingHTLCStatus),
	/// Included in a received commitment_signed message (implying we've
	/// revoke_and_ack'd it), but the remote hasn't yet revoked their previous
	/// state (see the example below). We have not yet included this HTLC in a
	/// commitment_signed message because we are waiting on the remote's
	/// aforementioned state revocation. One reason this missing remote RAA
	/// (revoke_and_ack) blocks us from constructing a commitment_signed message
	/// is because every time we create a new "state", i.e. every time we sign a
	/// new commitment tx (see [BOLT #2]), we need a new per_commitment_point,
	/// which are provided one-at-a-time in each RAA. E.g., the last RAA they
	/// sent provided the per_commitment_point for our current commitment tx.
	/// The other reason we should not send a commitment_signed without their RAA
	/// is because their RAA serves to ACK our previous commitment_signed.
	///
	/// Here's an example of how an HTLC could come to be in this state:
	/// remote --> update_add_htlc(prev_htlc)   --> local
	/// remote --> commitment_signed(prev_htlc) --> local
	/// remote <-- revoke_and_ack               <-- local
	/// remote <-- commitment_signed(prev_htlc) <-- local
	/// [note that here, the remote does not respond with a RAA]
	/// remote --> update_add_htlc(this_htlc)   --> local
	/// remote --> commitment_signed(prev_htlc, this_htlc) --> local
	/// Now `this_htlc` will be assigned this state. It's unable to be officially
	/// accepted, i.e. included in a commitment_signed, because we're missing the
	/// RAA that provides our next per_commitment_point. The per_commitment_point
	/// is used to derive commitment keys, which are used to construct the
	/// signatures in a commitment_signed message.
	/// Implies AwaitingRemoteRevoke.
	///
	/// [BOLT #2]: https://github.com/lightning/bolts/blob/master/02-peer-protocol.md
	AwaitingRemoteRevokeToAnnounce(PendingHTLCStatus),
	/// Included in a received commitment_signed message (implying we've revoke_and_ack'd it).
	/// We have also included this HTLC in our latest commitment_signed and are now just waiting
	/// on the remote's revoke_and_ack to make this HTLC an irrevocable part of the state of the
	/// channel (before it can then get forwarded and/or removed).
	/// Implies AwaitingRemoteRevoke.
	AwaitingAnnouncedRemoteRevoke(PendingHTLCStatus),
	Committed,
	/// Removed by us and a new commitment_signed was sent (if we were AwaitingRemoteRevoke when we
	/// created it we would have put it in the holding cell instead). When they next revoke_and_ack
	/// we'll drop it.
	/// Note that we have to keep an eye on the HTLC until we've received a broadcastable
	/// commitment transaction without it as otherwise we'll have to force-close the channel to
	/// claim it before the timeout (obviously doesn't apply to revoked HTLCs that we can't claim
	/// anyway). That said, ChannelMonitor does this for us (see
	/// ChannelMonitor::should_broadcast_holder_commitment_txn) so we actually remove the HTLC from
	/// our own local state before then, once we're sure that the next commitment_signed and
	/// ChannelMonitor::provide_latest_local_commitment_tx will not include this HTLC.
	LocalRemoved(InboundHTLCRemovalReason),
}

struct InboundHTLCOutput {
	htlc_id: u64,
	amount_msat: u64,
	cltv_expiry: u32,
	payment_hash: PaymentHash,
	state: InboundHTLCState,
}

enum OutboundHTLCState {
	/// Added by us and included in a commitment_signed (if we were AwaitingRemoteRevoke when we
	/// created it we would have put it in the holding cell instead). When they next revoke_and_ack
	/// we will promote to Committed (note that they may not accept it until the next time we
	/// revoke, but we don't really care about that:
	///  * they've revoked, so worst case we can announce an old state and get our (option on)
	///    money back (though we won't), and,
	///  * we'll send them a revoke when they send a commitment_signed, and since only they're
	///    allowed to remove it, the "can only be removed once committed on both sides" requirement
	///    doesn't matter to us and it's up to them to enforce it, worst-case they jump ahead but
	///    we'll never get out of sync).
	/// Note that we Box the OnionPacket as it's rather large and we don't want to blow up
	/// OutboundHTLCOutput's size just for a temporary bit
	LocalAnnounced(Box<msgs::OnionPacket>),
	Committed,
	/// Remote removed this (outbound) HTLC. We're waiting on their commitment_signed to finalize
	/// the change (though they'll need to revoke before we fail the payment).
	RemoteRemoved(OutboundHTLCOutcome),
	/// Remote removed this and sent a commitment_signed (implying we've revoke_and_ack'ed it), but
	/// the remote side hasn't yet revoked their previous state, which we need them to do before we
	/// can do any backwards failing. Implies AwaitingRemoteRevoke.
	/// We also have not yet removed this HTLC in a commitment_signed message, and are waiting on a
	/// remote revoke_and_ack on a previous state before we can do so.
	AwaitingRemoteRevokeToRemove(OutboundHTLCOutcome),
	/// Remote removed this and sent a commitment_signed (implying we've revoke_and_ack'ed it), but
	/// the remote side hasn't yet revoked their previous state, which we need them to do before we
	/// can do any backwards failing. Implies AwaitingRemoteRevoke.
	/// We have removed this HTLC in our latest commitment_signed and are now just waiting on a
	/// revoke_and_ack to drop completely.
	AwaitingRemovedRemoteRevoke(OutboundHTLCOutcome),
}

#[derive(Clone)]
enum OutboundHTLCOutcome {
	Success(Option<PaymentPreimage>),
	Failure(HTLCFailReason)

#[derive(Clone)]
enum invoice {
	Success (Option<Invoice>),
	Failure (InvoiceFail)
