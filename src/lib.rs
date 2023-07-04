mod bulk;
mod quote;

pub mod cli {
    use anyhow::{Context, Result};
    use crossbeam_channel::{bounded, Receiver, Sender};
    use is_terminal::IsTerminal;
    use std::io::prelude::*;
    use std::io::BufWriter;
    use std::sync::mpsc;
    use std::thread;

    use crate::bulk::BulkReader;
    use crate::quote::DoQuote;
    use crate::quote::QuoteBasic;
    use crate::quote::QuotePrintable;
    struct ChanChanTx<T, U> {
        payload: T,
        tx: Sender<U>,
    }
    struct ChanChanRx<U> {
        rx: Receiver<U>,
    }

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

    const EXMAPLES_MESSAGE: &str = "
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
            if !self.input_from_tty && std::io::stdin().is_terminal() {
                let mut buf_writer = BufWriter::new(writer);
                buf_writer
                    .write_all(EXMAPLES_MESSAGE.to_string().as_bytes())
                    .unwrap();
                return Ok(());
            }
            let mut buf_reader = BulkReader::new(reader, self.bulk_lines);

            // let (out_tx, out_rx) = bounded::<String>(3);
            // let (in_tx, in_rx) = bounded::<String>(3);
            let (out_tx, out_rx) = bounded::<ChanChanTx<Vec<Vec<u8>>, String>>(0);
            let (in_tx, in_rx) = mpsc::sync_channel::<ChanChanRx<String>>(self.workers as usize);

            for _i in 0..self.workers {
                let no_escape = self.no_escape;
                let out_delimiter = self.out_delimiter.clone();
                let out_rx = out_rx.clone();
                thread::spawn(move || {
                    let q = if !no_escape {
                        Box::new(QuotePrintable {}) as Box<dyn DoQuote>
                    } else {
                        Box::new(QuoteBasic {}) as Box<dyn DoQuote>
                    };
                    for chan_chan in out_rx {
                        //let mut s = Vec::<String>::new();
                        let mut s = Vec::<String>::new();
                        for buf in chan_chan.payload {
                            let mut line = std::str::from_utf8(&buf).unwrap().to_string();
                            if line.ends_with('\0') {
                                line.truncate(line.len() - 1)
                            }
                            s.push(q.quote(line));
                        }
                        // TODO: error を受信する用の thread を作成.
                        chan_chan
                            .tx
                            .send(s.join(&out_delimiter) + &out_delimiter)
                            .with_context(|| "could not send lines to printer thread".to_string())
                            .unwrap_or_else(|err| {
                                eprintln!("{}", err);
                                std::process::exit(1);
                            });
                    }
                });
            }

            let t = thread::spawn(move || {
                let mut buf_writer = BufWriter::new(writer);
                for line in in_rx {
                    // TODO: error を受信する用の thread を作成.
                    buf_writer
                        .write_all(line.rx.recv().unwrap().as_bytes())
                        //.with_context(|| format!("could not print lines"))
                        .unwrap_or_else(|err| {
                            if let Some(x) = err.raw_os_error() {
                                // println!("{}", x);
                                if x != 32 {
                                    eprintln!("{}", err);
                                }
                                std::process::exit(1);
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
                let (tx, rx) = bounded::<String>(0);
                out_tx
                    .send(ChanChanTx { payload: bulk, tx })
                    .with_context(|| "could not send lines to quote thread".to_string())
                    .unwrap_or_else(|err| {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    });
                in_tx
                    .send(ChanChanRx { rx })
                    .with_context(|| "could not send lines to quote thread".to_string())
                    .unwrap_or_else(|err| {
                        eprintln!("{}", err);
                        std::process::exit(1);
                    });
            }
            drop(out_tx);
            drop(in_tx);
            t.join().unwrap();

            Ok(())
        }
    }
}
