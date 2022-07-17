use anyhow::Result;
use clap::{ArgEnum, Parser};
use xquo::cli::{XQuo, XQuoArgs, XQuoOutDelimiter};

#[cfg(feature = "jemalloc")]
use tikv_jemallocator::Jemalloc;

#[cfg(feature = "jemalloc")]
#[global_allocator]
static GLOBAL: Jemalloc = Jemalloc;

#[derive(Debug, Clone, ArgEnum)]
pub enum OutDelimiter {
    Null,
    Lf,
}

fn workers_range(s: &str) -> Result<u8, String> {
    let n = s.to_string().parse::<u8>();
    match n {
        Ok(n) => {
            if n > 0 {
                Ok(n)
            } else {
                Ok(1)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

fn bulk_range(s: &str) -> Result<usize, String> {
    let n = s.to_string().parse::<usize>();
    match n {
        Ok(n) => {
            if n > 0 {
                Ok(n)
            } else {
                Ok(1)
            }
        }
        Err(e) => Err(e.to_string()),
    }
}

const COMMAND_USAGE: &'static str = "xquo [OPTIONS] < /path/to/file";

/// Quote null splited lines for Bash command line
#[derive(Parser)]
#[clap(version, usage = COMMAND_USAGE)]
struct Cli {
    /// Disable to escape non-printable chars("\n", "\b")
    #[clap(short, long)]
    no_escape: bool,

    /// The delmiter char to split lines in output.
    #[clap(short, long, arg_enum, default_value = "lf")]
    out_delimiter: OutDelimiter,

    /// The number of workers. If 2 or more is specified, the order of output lines is not preserved.
    #[clap(short, long, default_value = "1", parse(try_from_str=workers_range))]
    workers: u8,

    /// The number of lines bundled in a single bulk.
    #[clap(short, long, default_value = "100", parse(try_from_str=bulk_range))]
    bulk_lines: usize,

    /// Input from tty.
    #[clap(short = 't', long)]
    input_from_tty: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Cli::parse();
    let xquo = XQuo::new(XQuoArgs {
        no_escape: args.no_escape,
        out_delimiter: match args.out_delimiter {
            OutDelimiter::Null => XQuoOutDelimiter::Null,
            _ => XQuoOutDelimiter::Lf,
        },
        workers: args.workers,
        bulk_lines: args.bulk_lines,
        input_from_tty: args.input_from_tty,
    });
    xquo.quote(std::io::stdin(), std::io::stdout())?;

    Ok(())
}
