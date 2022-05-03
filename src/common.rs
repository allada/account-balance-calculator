// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

use bigdecimal::BigDecimal;
use serde::{Deserialize, Serialize};

pub type ClientId = u16;
pub type TxId = u32;

/// The type of a given transaction.
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub enum TransactionType {
    #[serde(rename = "deposit")]
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdrawal,
    #[serde(rename = "dispute")]
    Dispute,
    #[serde(rename = "resolve")]
    Resolve,
    #[serde(rename = "chargeback")]
    Chargeback,
}

/// Holds a raw transaction (usually from a csv).
#[derive(Serialize, Deserialize, Debug)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub transaction_type: TransactionType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<BigDecimal>,
}
