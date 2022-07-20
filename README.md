# xquo

Command line utility to quote null splited lines for Bash command line.

## Installation

Using `cargo`:

```console
$ cargo install xquo
```

Using `cargo` with `jemalloc` feature:

As far as I tried, it could be built in Ubuntu(`x86_64-unknown-linux-musl`) and Apine(`x86_64-unknown-linux-gnu`).

```console
$ cargo install xquo --features jemalloc
```

From repository relases page:

https://github.com/hankei6km/xquo/releases

## Usage

A file name containing `lf` character exists.

```console
$ ls
'123'$'\n''abc.txt'  '123 abc.txt'  '123"abc.txt'  "123'abc.txt"   abc.txt

$ find . -type f -exec echo ={}= \;
=./123
abc.txt=
=./123'abc.txt=
=./abc.txt=
=./123 abc.txt=
=./123"abc.txt=
```

Import the file list as a command line into the editor(ie. Vim).

```console
$ find . -type f -print0 | xquo | vim "+%s/^/ls -l /" -
```

```bash
ls -l './123'$'\n''abc.txt'
ls -l './123'"'"'abc.txt'
ls -l './abc.txt'
ls -l './123 abc.txt'
ls -l './123"abc.txt'
```

These can access a file correctly.

```text
:w !bash
-rw-r--r-- 1 vscode vscode 0 Jul  8 15:46 './123'$'\n''abc.txt'
-rw-r--r-- 1 vscode vscode 0 Jul  8 15:46 "./123'abc.txt"
-rw-r--r-- 1 vscode vscode 0 Jul  8 15:46 ./abc.txt
-rw-r--r-- 1 vscode vscode 0 Jul  8 15:46 './123 abc.txt'
-rw-r--r-- 1 vscode vscode 0 Jul  8 15:46 './123"abc.txt'

Press ENTER or type command to continue
```

## License

MIT License

Copyright (c) 2022 hankei6km
