# Account Balance Calculator

[![CI](https://github.com/allada/account-balance-calculator/workflows/CI/badge.svg)](https://github.com/allada/account-balance-calculator/actions/workflows/main.yml)

Utility program that will take a CSV of transaction information, keep track of the current balance of each client by their id and print out the final state of all accounts in CSV format (to stdout).

## Build Requirements
* Linux (untested on Windows, but probably will work)
* Cargo 1.56.0 (but earlier versions might also work)

### Running
```
$ cargo run -- ./src/tests/data/provided_sample_input.csv
```

This will print to the console the results from [provided_sample_input.csv](https://github.com/allada/account-balance-calculator/tree/master/src/tests/data/provided_sample_input.csv)

## Assumptions
There were many assumptions made for this project, here are a few:
* Only deposits can be disputed.
* Once a transaction is resolved it can be disputed again.
* Once a chargeback happens no more disputes can happen on same transaction.
* Once a chargeback happens withdrawals are ignored (on same account), all other types are processed.
* If `available` is lower than the `amount` of a withdrawal transaction is ignored.
* If there are not enough `available` funds for a dispute the dispute is ignored.
* Duplicate transactions (`tx`) are ignored (only first one is processed).
* All values input and output are expected to always be positive.
* Ordering of output is undefined.
* Transactions can be processed in any order as long as they are serial for any given client/account.
* If a number has more than 4 decimal places it will round the last digit (not floor it).

## Libraries used
* serde - Provides easier serialize/deserialize of rust structures.
* csv-async - Async version of the `csv` crate.
* futures - Used to make it easier to iterate a stream.
* tokio-util - Used to translate future's AsyncRead and tokio's AsyncRead.
* tokio - Async library.
* clap - Command line argument parser.
* bigdecimal - Utility that makes parsing large/small numbers much easier.
* num_cpus - To calculate the number of cores on the running machine.
* pretty_assertions - [dev] Makes `assert_eq` much easier to read in stdout.

### Security concerns
Probably the biggest concern is `csv-async`. This library is not very widely used and the authors do not appear to host any other notable crate projects.

If this was a huge concern it would not be too difficult to just use the `csv` crate and use something like `block_in_place` or just re-implement the needed functions in the async way.

### Licenses
All direct dependencies us a permissive license.

## Design choices

### Workers
Probably the biggest questionable design decision made here was to pipe the data to be processed into other spawns instead of doing them on the same thread. The instructions hinted that there could be thousands of clients connected streaming data to us. In such event, we would want to have each connection be as light as possible and push as much work onto child threads as possible. Ironically, for the code as it is right now it is probably going to ALWAYS be slower than having it all in one thread, however, I also made the assumption that if this was really used in production we would not be storing the transactions in an in-memory HashMap and instead we'd likely be using some kind of database. If we did use a database, the bottleneck would almost certainly be the latency of interacting with the database. By putting the work into worker spawns we could have different dedicated databases for each worker spawn resulting (in theory) in faster database iops (if the server was configured properly).

### Why HashMap for holding tx's?
By spec we may be asked to dispute/resolve/chargeback any transaction and the only info given is the transaction id (tx). I didn't feel it was worthwhile for this project at this time to have it use a database and so I took the simple route of a simple HashMap. Implementing a database is straightforward, but would require additional parameters at startup on where to place the database and I didn't want to make assumptions about what kind of hardware this will be running on. Lastly Hashmap in the way it is used should be able to hold on the order of 15 million entries per gigabyte, which for this demonstration is plenty.

### Why is all the logic in `run_with_args` instead of `main`?
Unit testing. I wanted to ensure I could write an integration test that was stable. If I put everything in `main` it would cause linking issues if I tried to use it in my unit tests. I could probably have spawned a sub-process of the program in my unit test to test it, but since I usually write my rust programs using `Bazel`'s `rust_rules` instead of `cargo` I wasn't sure how to make a data dependency of a test in cargo.

### Bigdecimal
Bigdecimal is definitely overkill for this project, however it was only defined on how big the exponent needed to be. In other words, the whole number part of the number could be nearly infinite in size. Since this kind of work often deals with blockchain, I wanted to ensure that if a test was run on this with a large mantissa it could definitely handle it.

### Error handling
Since stdout is reserved, any errors that happen are printed for debugging purposes to stderr. This may look funny if you run the program manually through a terminal, but most programs like this would be run automatically and likely have the stdout piped to another program or file. So, be mindful that if you see error-like messages in the terminal, it's likely printed to stderr not stdout. You may suppress stderr messages by using something like: `cargo run -- ./src/tests/data/provided_sample_input.csv 2>/dev/null`

# License

Copyright 2022 Nathan (Blaise) Bruer
