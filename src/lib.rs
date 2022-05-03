// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

use std::env;

use clap::Parser;
use csv_async::AsyncReaderBuilder as CsvAsyncReaderBuilder;
use futures::StreamExt;
use num_cpus::get as get_num_cpus;
use tokio::fs;
use tokio::io::{AsyncWrite, AsyncWriteExt};
use tokio_util::compat::TokioAsyncReadCompatExt;

mod error;
#[cfg(test)]
mod tests; // Failing to do this results in zero unit tests being run.
use error::{Error, ErrorKind};
mod common;
use common::Transaction;
mod account_manager;
use account_manager::AccountManager;

/// Command line arguments holder.
#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
pub struct Args {
    /// CSV file of all transactions.
    transactions_file: String,
}

/// For the given args will parse the csv file, stream the data to the AccountManager and
/// finally write the output csv to the provided `writer`.
/// Note: This is effectively a main() function, but in order to make unit testing easier
/// it is separated.
pub async fn run_with_args(args: Args, mut writer: impl AsyncWrite + Unpin) -> Result<(), Error> {
    let (account_manager, mut reader) = {
        // Setup and configure our classes and utilities.
        let file = match fs::File::open(&args.transactions_file).await {
            Ok(file) => file,
            Err(e) => {
                return Err(Error::new(
                    ErrorKind::NotFound,
                    format!(
                        "Error, could not open file: '{}', error '{}'",
                        args.transactions_file, e
                    ),
                ));
            }
        };

        // TODO(allada): Use std::thread::available_parallelism() instead of num_cpus::get() when
        // it is on stable long enough.
        let worker_threads = env::var_os("ACCOUNT_WORKER_SPAWNS").map_or(get_num_cpus(), |v| {
            v.into_string()
                .expect("Could not convert OsString to String. Probably UTF8 error.")
                .parse::<usize>()
                .expect("Could not convert ACCOUNT_WORKER_SPAWNS env to usize")
        });
        let reader = CsvAsyncReaderBuilder::new()
            .flexible(true)
            // Sadly, tokio's AsyncRead and Future's AsyncRead are not compatible, so we use
            // tokio_util::compat library to build our compatibility layer.
            .create_deserializer(file.compat());
        (AccountManager::new(worker_threads), reader)
    };

    let account_states = {
        // Process our csv data.
        let mut transaction_stream = reader.deserialize::<Transaction>();
        let mut row_number = 1; // Start at 1 because header was in input, but not in transaction_stream.
        while let Some(transaction_result) = transaction_stream.next().await {
            let transaction = match transaction_result {
                Ok(v) => v,
                Err(err) => {
                    eprintln!("Could not parse line {} due to error {:?}", row_number, err);
                    continue;
                }
            };
            account_manager.process_transaction(transaction).await?;
            row_number += 1;
        }
        account_manager.collect_account_states().await?
    };

    {
        // Print out final output.
        writer
            .write_all("client,available,held,total,locked\n".as_bytes())
            .await?;
        writer.flush().await?; // Be very mindful to flush on very write.
        for account_state in account_states {
            writer
                .write_all(
                    format!(
                        "{},{},{},{},{}\n",
                        &account_state.client,
                        account_state.available.round(4),
                        account_state.held.round(4),
                        (account_state.available + account_state.held).round(4),
                        &account_state.locked
                    )
                    .as_bytes(),
                )
                .await?;
            writer.flush().await?; // Be very mindful to flush on very write.
        }
    }
    Ok(())
}
