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

#[(Clone)]
enum sen_asset {send_asset}
