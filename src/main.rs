// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

use clap::Parser;
use tokio::io::stdout;

use account_balance_calculator::{run_with_args, Args};

#[tokio::main]
async fn main() {
    let args = Args::parse();
    run_with_args(args, stdout()).await.unwrap();
}
