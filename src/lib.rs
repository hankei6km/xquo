mod bulk;
mod quote;

pub mod cli {
    use anyhow::{Context, Result};
    use atty::Stream;
    use crossbeam_channel::unbounded;
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::sync::mpsc;
    use std::thread;

    use crate::bulk::bulk::BulkReader;
    use crate::quote::DoQuote;
    use crate::quote::QuoteBasic;
    use crate::quote::QuotePrintable;

    pub enum XQuoOutDelimiter {
        Null,
        Lf,
    }

    pub struct XQuoArgs {
        pub no_escape: bool,
        pub out_delimiter: XQuoOutDelimiter,
        pub workers: u8,
        pub bulk_lines: usize,
        pub input_from_tty: bool,
    }

    pub struct XQuo {
        no_escape: bool,
        out_delimiter: String,
        workers: u8,
        bulk_lines: usize,
        input_from_tty: bool,
    }

    const EXMAPLES_MESSAGE: &'static str = "
xquo reads lines from standard input.

EXAMPLES:
    $ find . -type f -print0 | xqua

For more information try --help

";

    impl XQuo {
        pub fn new(args: XQuoArgs) -> XQuo {
            XQuo {
                no_escape: args.no_escape,
                out_delimiter: match args.out_delimiter {
                    XQuoOutDelimiter::Null => "\0".to_string(),
                    // XQuoOutDelimiter::Lf => "\n".to_string(),
                    _ => "\n".to_string(),
                },
                workers: args.workers,
                bulk_lines: args.bulk_lines,
                input_from_tty: args.input_from_tty,
            }
        }
        pub fn quote(
            &self,
            reader: impl std::io::Read,
            writer: std::io::Stdout,
        ) -> Result<(), Box<dyn std::error::Error>> {
            if self.input_from_tty == false && atty::is(Stream::Stdin) {
                let mut buf_writer = BufWriter::new(writer);
                buf_writer
                    .write(format!("{}", EXMAPLES_MESSAGE).as_bytes())
                    .unwrap();
                return Ok(());
            }
            let mut buf_reader = BulkReader::new(reader, self.bulk_lines);

            // let (out_tx, out_rx) = bounded::<String>(3);
            // let (in_tx, in_rx) = bounded::<String>(3);
            let (out_tx, out_rx) = unbounded::<Vec<Vec<u8>>>();
            let (in_tx, in_rx) = mpsc::channel::<String>();

            for _i in 0..self.workers {
                let no_escape = self.no_escape;
                let out_delimiter = self.out_delimiter.clone();
                let out_rx = out_rx.clone();
                let in_tx = in_tx.clone();
                thread::spawn(move || {
                    let q = if !no_escape {
                        Box::new(QuotePrintable {}) as Box<dyn DoQuote>
                    } else {
                        Box::new(QuoteBasic {}) as Box<dyn DoQuote>
                    };
                    for bulked in out_rx {
                        //let mut s = Vec::<String>::new();
                        let mut s = Vec::<String>::new();
                        for buf in bulked {
                            let mut line = std::str::from_utf8(&buf).unwrap().to_string();
                            if line.ends_with("\0") {
                                line.truncate(line.len() - 1)
                            }
                            s.push(q.quote(line));
                        }
                        // TODO: error を受信する用の thread を作成.
                        in_tx
                            .send(s.join(&out_delimiter) + &out_delimiter)
                            .with_context(|| format!("could not send lines to printer thread"))
                            .unwrap_or_else(|err| {
                                eprintln!("{}", err);
                                std::process::exit(1);
                            });
                    }
                });
            }
            // clone の大本になった sender を drop しておく.
            // これがないと in_rx の iterator が終了しない.
            drop(in_tx);

            let t = thread::spawn(move || {
                let mut buf_writer = BufWriter::new(writer);
                for line in in_rx {
                    // TODO: error を受信する用の thread を作成.
                    buf_writer
                        .write(line.as_bytes())
                        //.with_context(|| format!("could not print lines"))
                        .unwrap_or_else(|err| {
                            match err.raw_os_error() {
                                Some(x) => {
                                    // println!("{}", x);
                                    if x != 32 {
                                        eprintln!("{}", err);
                                    }
                                    std::process::exit(1);
                                }
                                _ => {}
                            }
                            eprintln!("{}", err);
                            std::process::exit(1);
                        });
                }
            });

            loop {
                //let mut buf = Vec::<u8>::new();
                //if buf_reader.read_until(0, &mut buf)? == 0 {
                //    break;
                //}
                //out_tx.send(buf).unwrap();
                let (bulk, line_cnt) = buf_reader.read(0);
                if line_cnt == 0 {
                    break;
                }
                // TODO: error を受信する用の thread を作成.
                out_tx
                    .send(bulk)
                    .with_context(|| format!("could not send lines to quote thread"))
                    .unwrap_or_else(|err| {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    });
            }
            drop(out_tx);
            t.join().unwrap();

            Ok(())
        }
    }
}
