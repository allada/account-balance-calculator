// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

use std::collections::{HashMap, VecDeque};

use crate::common::{ClientId, Transaction, TransactionType, TxId};
use crate::{make_other_err, Error, ErrorKind};
use bigdecimal::BigDecimal;

use tokio::sync::mpsc;
use tokio::task::JoinHandle;

/// Number of messages allowed to be in cross-spawn channel before backpressure
/// is applied to writer.
const CHANNEL_BUFFER_SIZE: usize = 32;

/// Holds the current state of a client (account).
#[derive(Default, Debug)]
pub struct ClientState {
    pub client: ClientId,
    pub available: BigDecimal,
    pub held: BigDecimal,
    pub locked: bool,

    tx_for_transaction_state: HashMap<TxId, (TransactionType, BigDecimal)>,
}

impl PartialEq for ClientState {
    fn eq(&self, other: &Self) -> bool {
        self.client == other.client
            && self.available == other.available
            && self.held == other.held
            && self.locked == other.locked
    }
}

/// Checks the state of a given transaction (tx) to ensure it is in the `allowed_tx_state` and
/// returns a mutable reference of the transaction state  and the amount of the transaction.
/// We reuse TransactionType here to also represent the state of the transaction (eg: if it's being
/// disputed it will be in state `TransactionType::Dispute`).
// Note: This was not placed into `ClientState` because it returns a mutable reference and if it
// was a part of `ClientState` the mutable reference would be to `self`, resulting in being unable
// to mutate anything else on `ClientState` as long as the result of this function lived.
fn get_tx_state_and_check_state<'a>(
    tx_for_transaction_state: &'a mut HashMap<TxId, (TransactionType, BigDecimal)>,
    transaction: &Transaction,
    allowed_tx_state: &TransactionType,
) -> Result<(&'a mut TransactionType, &'a BigDecimal), Error> {
    let (tx_state, amount) = tx_for_transaction_state
        .get_mut(&transaction.tx)
        .ok_or_else(|| {
            make_other_err!("Tx ({}) does not exist : {:?}", transaction.tx, transaction)
        })?;
    if tx_state == allowed_tx_state {
        return Ok((tx_state, amount));
    }
    match tx_state {
        TransactionType::Withdrawal => Err(make_other_err!(
            "Cannot dispute a withdrawal : {:?}",
            transaction
        )),
        TransactionType::Dispute => Err(make_other_err!(
            "Tx ({}) is already being disputed : {:?}",
            transaction.tx,
            transaction
        )),
        TransactionType::Chargeback => Err(make_other_err!(
            "Tx ({}) has already been chargebacked : {:?}",
            transaction.tx,
            transaction
        )),
        TransactionType::Resolve => {
            unreachable!("Resolve should never be set in tx_for_transaction_state")
        }
        TransactionType::Deposit => Err(make_other_err!(
            "Tx ({}) is not under dispute : {:?}",
            transaction.tx,
            transaction
        )),
    }
}

impl ClientState {
    #[cfg(test)]
    pub(crate) fn new(
        client: ClientId,
        available: BigDecimal,
        held: BigDecimal,
        locked: bool,
    ) -> Self {
        Self {
            client,
            available,
            held,
            locked,
            tx_for_transaction_state: Default::default(),
        }
    }

    fn deposit(&mut self, transaction: Transaction) -> Result<(), Error> {
        assert!(transaction.transaction_type == TransactionType::Deposit);
        // Not sure if we should prevent deposits if the account is locked?
        // I assume it's ok if a client deposits funds if their account is locked.
        if self.tx_for_transaction_state.contains_key(&transaction.tx) {
            return Err(make_other_err!(
                "Transaction ({}) already processed : {:?}",
                transaction.tx,
                transaction
            ));
        }
        let amount = transaction
            .amount
            .ok_or_else(|| make_other_err!("Amount must be provided in Deposit"))?;
        self.available += &amount;
        // TODO(allada) I am unsure if it is common to have zero amounts here, if it is zero
        // we could avoid creating this transaction record, for now I'll assume it's not
        // common.
        self.tx_for_transaction_state
            .insert(transaction.tx, (TransactionType::Deposit, amount));
        Ok(())
    }

    fn withdrawal(&mut self, transaction: Transaction) -> Result<(), Error> {
        assert!(transaction.transaction_type == TransactionType::Withdrawal);
        if self.locked {
            return Err(make_other_err!(
                "Account ({}) is locked. Transaction not processed : {:?}",
                transaction.client,
                transaction
            ));
        }
        if self.tx_for_transaction_state.contains_key(&transaction.tx) {
            return Err(make_other_err!(
                "Transaction ({}) already processed : {:?}",
                transaction.tx,
                transaction
            ));
        }
        let amount = transaction
            .amount
            .ok_or_else(|| make_other_err!("Amount must be provided in Deposit"))?;
        if self.available <= amount {
            return Err(make_other_err!(
                "Account did not have enough available ({}) funds in Transaction",
                self.available
            ));
        }
        self.available -= &amount;
        self.tx_for_transaction_state
            .insert(transaction.tx, (TransactionType::Withdrawal, amount));
        Ok(())
    }

    fn dispute(&mut self, transaction: Transaction) -> Result<(), Error> {
        assert!(transaction.transaction_type == TransactionType::Dispute);
        let (tx_state, amount) = get_tx_state_and_check_state(
            &mut self.tx_for_transaction_state,
            &transaction,
            &TransactionType::Deposit,
        )?;

        if &self.available < amount {
            return Err(make_other_err!(
                "Account did not have enough available ({}) funds in Transaction : {:?}",
                self.available,
                transaction
            ));
        }
        *tx_state = TransactionType::Dispute;
        self.available -= amount;
        self.held += amount;
        Ok(())
    }

    fn resolve(&mut self, transaction: Transaction) -> Result<(), Error> {
        assert!(transaction.transaction_type == TransactionType::Resolve);
        let (tx_state, amount) = get_tx_state_and_check_state(
            &mut self.tx_for_transaction_state,
            &transaction,
            &TransactionType::Dispute,
        )?;

        if &self.held < amount {
            return Err(make_other_err!(
                "Account did not have enough held ({}) funds in Transaction : {:?}",
                self.available,
                transaction
            ));
        }
        *tx_state = TransactionType::Deposit;
        self.held -= amount;
        self.available += amount;
        Ok(())
    }

    fn chargeback(&mut self, transaction: Transaction) -> Result<(), Error> {
        assert!(transaction.transaction_type == TransactionType::Chargeback);
        let (tx_state, amount) = get_tx_state_and_check_state(
            &mut self.tx_for_transaction_state,
            &transaction,
            &TransactionType::Dispute,
        )?;
        if &self.held < amount {
            return Err(make_other_err!(
                "Account did not have enough held ({}) funds in Transaction : {:?}",
                self.available,
                transaction
            ));
        }
        *tx_state = TransactionType::Chargeback;
        self.held -= amount;
        self.locked = true;
        Ok(())
    }
}

/// This is designed to be run in a `tokio::spawn` and will constantly pull the rx stream
/// and process the given transaction. When the stream is closed it will collect all the
/// final `ClientState`s into a single vector.
async fn process_account_transactions(
    mut rx: mpsc::Receiver<Transaction>,
) -> Result<Vec<ClientState>, Error> {
    // TODO(allada) We should use a database here instead of storing it all in memory.
    let mut state_for_client = HashMap::<ClientId, ClientState>::new();
    while let Some(transaction) = rx.recv().await {
        let state = match state_for_client.get_mut(&transaction.client) {
            Some(state) => state,
            None => {
                state_for_client.insert(
                    transaction.client,
                    ClientState {
                        client: transaction.client,
                        ..Default::default()
                    },
                );
                state_for_client.get_mut(&transaction.client).unwrap()
            }
        };

        let result = match transaction.transaction_type {
            TransactionType::Deposit => state.deposit(transaction),
            TransactionType::Withdrawal => state.withdrawal(transaction),
            TransactionType::Dispute => state.dispute(transaction),
            TransactionType::Resolve => state.resolve(transaction),
            TransactionType::Chargeback => state.chargeback(transaction),
        };
        if let Err(err) = result {
            eprintln!("{}", err.to_string());
        }
    }
    Ok(state_for_client.into_values().collect())
}

type WorkerHandle = (
    mpsc::Sender<Transaction>,
    JoinHandle<Result<Vec<ClientState>, Error>>,
);

/// AccountManager manages the state of each account and gives APIs into sending transactions
/// through it. This class is intentionally multi-threaded and will fan out the transactions
/// onto N number of workers. This is done because it was hinted that if we had thousands of
/// streams pushing transactions we'd want to ensure we could utilize as many threads as possible
/// to ensure we don't bottleneck the TCP sockets. In addition, if this was to be used in
/// production there would almost certainly be a database to hold the processed transactions
/// instead of storing them in memory (like it does here). Since the transactions must be processed
/// in order per account (not globally), we would be spreading the database latency over N number
/// of workers, in theory giving us much higher throughput.
///
/// In the event we might ever want to retrieve the current state of the client, an API could be
/// exposed in this class which would ask the specific worker for the current account state.
/// Although, this would be quite complicated to implement.
///
/// Processing of transactions does not require any locks, thus many immutable references to this
/// struct can be used if many connected clients needed to stream transactions.
pub struct AccountManager {
    workers: Vec<WorkerHandle>,
}

impl AccountManager {
    /// Construct a new AccountManager.
    ///
    /// `num_workers` represents the number of workers spawned in the background to do the
    /// processing.
    pub fn new(num_workers: usize) -> Self {
        assert!(
            num_workers > 0,
            "`num_workers` must be at least 1 in AccountManager"
        );
        assert!(
            num_workers < u16::MAX.into(),
            "`num_workers` must be less than u16::MAX in AccountManager"
        );
        let mut workers = Vec::with_capacity(num_workers);
        for _ in 0..num_workers {
            let (tx, rx) = mpsc::channel(CHANNEL_BUFFER_SIZE);
            workers.push((tx, tokio::spawn(process_account_transactions(rx))));
        }
        Self { workers }
    }

    /// Sends a transaction to a worker to be processed.
    pub async fn process_transaction(&self, transaction: Transaction) -> Result<(), Error> {
        let worker_index = (transaction.client as usize) % self.workers.len();
        self.workers[worker_index].0.send(transaction).await?;
        Ok(())
    }

    /// Closes all the workers and returns a VecDeque of all client states.
    pub async fn collect_account_states(self) -> Result<VecDeque<ClientState>, Error> {
        let mut client_states = VecDeque::new();
        for (sender, join_handle) in self.workers {
            drop(sender); // Close our channel.
            client_states.append(&mut VecDeque::from(join_handle.await??));
        }
        Ok(client_states)
    }
}
