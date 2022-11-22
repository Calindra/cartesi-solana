/**
 * Copy from solana just to deserialize de input
 */
use generic_array::{typenum::U64, GenericArray};
use solana_program::message::Message;
use serde::Serialize;
use serde::Deserialize;
use solana_program::short_vec;

use crate::anchor_lang::solana_program;

#[derive(Serialize, Deserialize)]
pub struct Signature(GenericArray<u8, U64>);

#[derive(Serialize, Deserialize)]
pub struct Transaction {
    /// A set of signatures of a serialized [`Message`], signed by the first
    /// keys of the `Message`'s [`account_keys`], where the number of signatures
    /// is equal to [`num_required_signatures`] of the `Message`'s
    /// [`MessageHeader`].
    ///
    /// [`account_keys`]: Message::account_keys
    /// [`MessageHeader`]: crate::message::MessageHeader
    /// [`num_required_signatures`]: crate::message::MessageHeader::num_required_signatures
    // NOTE: Serialization-related changes must be paired with the direct read at sigverify.
    #[serde(with = "short_vec")]
    pub signatures: Vec<Signature>,

    /// The message to sign.
    pub message: Message,
}
