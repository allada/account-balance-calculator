// Copyright 2022 Nathan (Blaise) Bruer.  All rights reserved.

use pretty_assertions::assert_eq; // Gives easier to read output for assert errors.
use tokio::io::{self, AsyncBufReadExt, BufReader};
use tokio::try_join;

use crate::{run_with_args, Args, Error};

#[tokio::test]
async fn sanity_check_provided_sample_data_test() -> Result<(), Error> {
    let args = Args {
        transactions_file: "src/tests/data/provided_sample_input.csv".to_string(),
    };

    // Configure a future that will process the output of our program into a vector by line.
    const BUFFER_SIZE: usize = 1024;
    let (tx, rx) = io::duplex(BUFFER_SIZE);
    let reader_spawn_fut = async move {
        let mut buf_reader = BufReader::new(rx);
        let mut output_data = vec![];
        loop {
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break; // EOF.
            }
            output_data.push(line);
        }
        Result::<Vec<String>, Error>::Ok(output_data)
    };

    let (_, mut output_lines) = try_join!(run_with_args(args, tx), reader_spawn_fut)?;
    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    output_lines[1..].sort_unstable();
    assert_eq!(
        output_lines,
        vec![
            "client,available,held,total,locked\n",
            "1,1.5000,0,1.5000,false\n",
            "2,2.0000,0,2.0000,false\n",
        ]
    );
    Ok(())
}

#[tokio::test]
async fn sanity_check_generated_sample_data_test() -> Result<(), Error> {
    let args = Args {
        transactions_file: "src/tests/data/generated_sample_input.csv".to_string(),
    };

    // Configure a future that will process the output of our program into a vector by line.
    const BUFFER_SIZE: usize = 1024;
    let (tx, rx) = io::duplex(BUFFER_SIZE);
    let reader_spawn_fut = async move {
        let mut buf_reader = BufReader::new(rx);
        let mut output_data = vec![];
        loop {
            let mut line = String::new();
            let bytes_read = buf_reader.read_line(&mut line).await?;
            if bytes_read == 0 {
                break; // EOF.
            }
            output_data.push(line);
        }
        Result::<Vec<String>, Error>::Ok(output_data)
    };

    let (_, mut output_lines) = try_join!(run_with_args(args, tx), reader_spawn_fut)?;
    // Ordering in the output is undefined, so we must sort here, but cannot include header
    // line in our sorting.
    output_lines[1..].sort_unstable();
    assert_eq!(
        output_lines,
        vec![
            "client,available,held,total,locked\n",
            "1,2.4900,0,2.4900,false\n",
            "2,0.0012,0,0.0012,false\n",
            "3,10.0000,0.0000,10.0000,true\n",
            "4,2.2222,3.3333,5.5555,false\n",
        ]
    );
    Ok(())
}
