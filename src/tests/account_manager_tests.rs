// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

use crate::account_manager::ClientState;
use crate::common::{Transaction, TransactionType};
use crate::{AccountManager, Error};

// Gives easier to read output for assert errors.
use pretty_assertions::assert_eq;

#[tokio::test]
async fn simple_deposit_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn simple_withdrawal_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((2).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some((1).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn simple_dispute_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (0).into(), /* available */
            (1).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn dispute_with_resolve_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn dispute_with_chargeback_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (0).into(), /* available */
            (0).into(), /* held */
            true,       /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn deposit_duplicate_tx_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn chargeback_prevents_withdrawals_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Withdrawal,
            client: 1,
            tx: 3,
            amount: Some((1).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            true,       /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn allow_deposits_if_chargeback_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 3,
            amount: Some((1).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (2).into(), /* available */
            (0).into(), /* held */
            true,       /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn not_enough_funds_for_withdrawal_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Withdrawal,
            client: 1,
            tx: 2,
            amount: Some((2).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn dispute_with_resolve_can_be_disputed_again_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Resolve,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (0).into(), /* available */
            (1).into(), /* held */
            false,      /* locked */
        )
    );
    Ok(())
}

#[tokio::test]
async fn chargeback_does_not_allow_dispute_on_same_transaction_test() -> Result<(), Error> {
    const NUM_WORKERS: usize = 5;
    let account_manager = AccountManager::new(NUM_WORKERS);

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Deposit,
            client: 1,
            tx: 2,
            amount: Some((1).into()),
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;
    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Chargeback,
            client: 1,
            tx: 1,
            amount: None,
        })
        .await?;

    account_manager
        .process_transaction(Transaction {
            transaction_type: TransactionType::Dispute,
            client: 1,
            tx: 1,
            amount: Some((1).into()),
        })
        .await?;

    let mut account_states = account_manager.collect_account_states().await?;

    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    account_states
        .make_contiguous()
        .sort_unstable_by(|a, b| a.client.cmp(&b.client));

    assert_eq!(
        account_states[0],
        ClientState::new(
            1,          /* client */
            (1).into(), /* available */
            (0).into(), /* held */
            true,       /* locked */
        )
    );
    Ok(())
}
