use std::fs::File; 
use std::io::Write;
use std::path::PathBuf;

use crate::algorithms::Algorithm;
use crate::post_process::Encoding;
use crate::chunked_stream::ChunkedStream;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct Opts {
    /// Input denotes a filepath (cannot be a directory)
    #[structopt(short, long)]
    pub file: bool,

    /// Prints the list of all the available hashing algorithms
    #[structopt(short, long)]
    pub list: bool,

    /// Chosen hashing algorithm name
    #[structopt(required_unless = "list")]
    pub algorithm: Option<String>,

    /// Input string
    #[structopt(required_unless = "list")]
    pub input: Option<String>,

    /// Encoding type for output hash
    #[structopt(short, long, 
        parse(try_from_str = Encoding::from_str), 
        default_value = "hex")]
    pub encoding: Encoding,

    /// Output file to write digest result to [default: stdout]
    #[structopt(short, long, parse(from_os_str))]
    pub output: Option<PathBuf>
}

impl Opts {
    /// Processes the Opts struct (consumes it) and prints the result
    /// of the digest to stdout
    pub fn process(&self) -> anyhow::Result<()> {
        if self.list {
            self.write_to_output(algo_list())
        }
        else if let (Some(algorithm), Some(input)) = (&self.algorithm, &self.input) {
            let data = 
            if self.file    { ChunkedStream::from_file(input)? }
            else            { ChunkedStream::from_string(input) };

            let digest_bytes = crate::router::digest_from(data, &algorithm)?;
            let digest_encoded = self.encoding.encode(digest_bytes);

            self.write_to_output(digest_encoded)
        }
        else {
            Err(anyhow::anyhow!("Opts are unhandled! Please refer to the developers."))
        }
    }

    // Write to filepath desired by the output field (stdout if None)
    fn write_to_output(&self, msg: String) -> anyhow::Result<()> {
        match &self.output {
            Some(path) => {
                let mut out = File::create(path)?;
                writeln!(out, "{}", msg)?;
            },
            None => println!("{}", msg)
        }

        Ok(())
    }
}

fn algo_list() -> String {
    let mut list = String::new();
    let mut count = 0;
    for algo in crate::algorithms::ALGORITHMS.iter() {
        match algo {
            Algorithm::Single(name) => {
                list.push_str(&format!("· {}\n", name));
                count += 1;
            },
            Algorithm::Family { name, members } => {
                list.push_str(&format!("· {} family\n", name));
                for member in members {
                    list.push_str(&format!("˪→ {}\n", member));
                    count += 1;
                }
            }
        }
    }

    // Remove last newline
    list.pop(); 

    format!("Algorithm count: {}\n{}", count, list)
}