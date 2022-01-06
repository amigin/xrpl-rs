pub mod account;
pub mod fee;
pub mod submit;

use std::convert::{TryFrom, TryInto};
use std::num::ParseIntError;

use serde;
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// An address used to identify an account.
pub type Address = String;

/// A Marker can be used to paginate the server response. It's content is intentionally undefined. Each server can define a marker as desired.
pub type Marker = Value;

pub type H256 = String;

/// Unique request id.
///
/// NOTE Assigning same id to different requests will cause the previous request to be unsubscribed.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RequestId {
    /// A numerical ID, represented by a `u64`.
    Number(u64),
    /// A non-numerical ID, for example a hash.
    String(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct LedgerInfo {
    /// (Optional) A 20-byte hex string for the ledger version to use. (See Specifying Ledgers)
    pub ledger_hash: Option<String>,
    /// (Optional) The ledger index of the ledger to use, or a shortcut string to choose a ledger automatically. (See Specifying Ledgers)
    pub ledger_index: Option<i64>,
    /// (Omitted if ledger_index is provided instead) The ledger index of the current in-progress ledger, which was used when retrieving this information.
    pub ledger_current_index: Option<i64>,
    /// (May be omitted) If true, the information in this response comes from a validated ledger version. Otherwise, the information is subject to change. New in: rippled 0.90.0
    pub validated: Option<bool>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct PaginationInfo {
    /// (Optional) Limit the number of transactions to retrieve. Cannot be less than 10 or more than 400. The default is 200.
    pub limit: Option<i64>,
    /// (Optional) Value from a previous paginated response. Resume retrieving data where that response left off. Updated in: rippled 1.5.0.
    pub marker: Option<Marker>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Response<T> {
    /// (WebSocket only) ID provided in the request that prompted this response
    pub id: Option<RequestId>,
    /// (WebSocket only) The value success indicates the request was successfully received and understood by the server. Some client libraries omit this field on success.
    pub status: Option<String>,
    /// (WebSocket only) The value response indicates a direct response to an API request. Asynchronous notifications use a different value such as ledgerClosed or transaction.
    pub r#type: Option<String>,
    /// The result of the query; contents vary depending on the command.
    pub result: Result<T>,
    /// (May be omitted) If this field is provided, the value is the string load. This means the client is approaching the rate limiting threshold where the server will disconnect this client.
    pub warning: Option<String>,
    /// (May be omitted) If this field is provided, it contains one or more Warnings Objects with important warnings. For details, see API Warnings. New in: rippled 1.5.0
    /// TODO: Add Warnings Object.
    pub warnings: Option<Vec<Value>>,
    /// (May be omitted) If true, this request and response have been forwarded from a Reporting Mode server to a P2P Mode server (and back) because the request requires data that is not available in Reporting Mode. The default is false.
    pub forwarded: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(untagged)]
pub enum Result<T> {
    Ok(T),
    Error(Value),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Error {
    pub error: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SignerList {
    #[serde(rename = "SignerEntries")]
    pub signer_entries: Vec<SignerEntry>,
    #[serde(rename = "SignerQuorum")]
    pub signer_quorum: u32,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct SignerEntry {
    #[serde(rename = "Account")]
    pub account: String,
    #[serde(rename = "SignerWeight")]
    pub signer_weight: u16,
}

#[derive(Debug, Eq, PartialEq, Ord, PartialOrd, Default, Clone)]
pub struct Drops(u64);

impl Serialize for Drops {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&format!("{}", self.0))
    }
}

impl<'de> Deserialize<'de> for Drops {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Drops, D::Error>
    where
        D: serde::de::Deserializer<'de>,
    {
        deserializer.deserialize_str(DropsVisitor)
    }
}

struct DropsVisitor;

impl<'de> serde::de::Visitor<'de> for DropsVisitor {
    type Value = Drops;

    fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
        formatter.write_str("an unsigned integer")
    }

    fn visit_str<E>(self, value: &str) -> std::result::Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        Ok(value
            .try_into()
            .map_err(|e| serde::de::Error::custom(format!("{:?}", e)))?)
    }
}

impl TryFrom<String> for Drops {
    type Error = ParseIntError;

    fn try_from(value: String) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.parse()?))
    }
}

impl TryFrom<&str> for Drops {
    type Error = ParseIntError;

    fn try_from(value: &str) -> std::result::Result<Self, Self::Error> {
        Ok(Self(value.parse()?))
    }
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(untagged)]
pub enum CurrencyAmount {
    XRP(Drops),
    IssuedCurrency(IssuedCurrencyAmount),
}

impl Default for CurrencyAmount {
    fn default() -> Self {
        return Self::XRP(Drops(0u64));
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct IssuedCurrencyAmount {
    pub value: String,
    pub currency: String,
    pub issuer: Address,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionEntryRequest {
    pub tx_hash: Option<String>,
    pub ledger_index: Option<u64>,
    pub ledger_hash: Option<String>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
pub struct TransactionEntryResponse {
    pub tx_json: Option<Value>,
    pub ledger_index: Option<u64>,
    pub ledger_hash: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(tag = "LedgerEntryType")]
pub enum LedgerEntry {
    Unknown,
    AccountRoot(AccountRoot),
    Check(Check),
}

impl Default for LedgerEntry {
    fn default() -> Self {
        Self::Unknown
    }
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct AccountRoot {
    /// The identifying (classic) address of this account.
    pub account: Address,
    /// The account's current XRP balance in drops, represented as a string.
    pub balance: CurrencyAmount,
    /// A bit-map of boolean flags enabled for this account.
    pub flags: u32,
    /// The number of objects this account owns in the ledger, which contributes to its owner reserve.
    pub owner_count: u32,
    /// The identifying hash of the transaction that most recently modified this object.
    #[serde(rename = "PreviousTxnID")]
    pub previous_txn_id: H256,
    /// The index of the ledger that contains the transaction that most recently modified this object.
    pub previous_txn_lgr_seq: u32,
    /// The sequence number of the next valid transaction for this account.
    pub sequence: u32,
    /// (Optional) The identifying hash of the transaction most recently sent by this account. This field must be enabled to use the AccountTxnID transaction field. To enable it, send an AccountSet transaction with the asfAccountTxnID flag enabled.
    pub account_txn_id: Option<H256>,
    /// (Optional) A domain associated with this account. In JSON, this is the hexadecimal for the ASCII representation of the domain. Cannot be more than 256 bytes in length.
    pub domain: Option<String>,
    /// (Optional) The md5 hash of an email address. Clients can use this to look up an avatar through services such as Gravatar .
    pub email_hash: Option<H256>,
    /// (Optional) A public key that may be used to send encrypted messages to this account. In JSON, uses hexadecimal. Must be exactly 33 bytes, with the first byte indicating the key type: 0x02 or 0x03 for secp256k1 keys, 0xED for Ed25519 keys.
    pub message_key: Option<String>,
    /// (Optional) The address of a key pair that can be used to sign transactions for this account instead of the master key. Use a SetRegularKey transaction to change this value.
    pub regular_key: Option<String>,
    /// (Optional) How many Tickets this account owns in the ledger. This is updated automatically to ensure that the account stays within the hard limit of 250 Tickets at a time. This field is omitted if the account has zero Tickets. (Added by the TicketBatch amendment )
    pub ticket_count: Option<u32>,
    /// (Optional) How many significant digits to use for exchange rates of Offers involving currencies issued by this address. Valid values are 3 to 15, inclusive. (Added by the TickSize amendment.)
    pub tick_size: Option<u8>,
    /// (Optional) A transfer fee to charge other users for sending currency issued by this account to each other.
    pub transfer_rate: Option<u32>,
}

#[derive(Default, Debug, Serialize, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "PascalCase")]
pub struct Check {
    /// The sender of the Check. Cashing the Check debits this address's balance.
    pub account: Address,
    /// The intended recipient of the Check. Only this address can cash the Check, using a CheckCash transaction.
    pub destination: Address,
    /// A bit-map of boolean flags enabled for this account.
    pub flags: u32,
}