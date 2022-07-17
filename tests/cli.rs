use std::str::from_utf8;

use assert_cmd::Command;
use predicates::prelude::*;

#[test]
fn quote_lines_and_joined_by_lf() -> Result<(), Box<dyn std::error::Error>> {
    //let file = assert_fs::NamedTempFile::new("sample.txt")?;
    //file.write_str("A test\nActual content\nMore content\nAnother test")?;

    let mut cmd = Command::cargo_bin("xquo")?;
    let lines = ["test", "test test", "テスト🦀", "テスト テスト"];
    let input_lines = lines.join("\0");
    let ex = lines.map(|v| format!("'{}'", v)).join("\n") + "\n";

    cmd.write_stdin(input_lines);
    cmd.assert().success().stdout(predicate::eq(ex.as_bytes()));
    Ok(())
}

#[test]
fn wrap_sngle_quote_char() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("xquo")?;
    let lines = ["test", "test'test", "テスト🦀'", "テスト'テスト"];
    let input_lines = lines.join("\0");
    let ex_lines = [
        "test",
        "test'\"'\"'test",
        "テスト🦀'\"'\"'",
        "テスト'\"'\"'テスト",
    ];
    let ex = ex_lines.map(|v| format!("'{}'", v)).join("\n") + "\n";

    cmd.write_stdin(input_lines);
    cmd.assert().success().stdout(predicate::eq(ex.as_bytes()));
    Ok(())
}

#[test]
fn joined_by_null() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("xquo")?;
    let lines = ["test", "test\ntest", "テスト🦀\n", "テスト\u{8}テスト"];
    let input_lines = lines.join("\0");
    let ex_lines = [
        "test",
        "test'$'\\n''test",
        "テスト🦀'$'\\n''",
        "テスト'$'\\b''テスト",
    ];
    let ex = ex_lines.map(|v| format!("'{}'", v)).join("\0") + "\0";

    cmd.write_stdin(input_lines).args(&["-o", "null"]);
    cmd.assert().success().stdout(predicate::eq(ex.as_bytes()));
    Ok(())
}

#[test]
fn disable_escape_chars() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("xquo")?;
    let lines = ["test", "test\ntest", "テスト🦀\n", "テスト\u{8}テスト"];
    let input_lines = lines.join("\0");
    let ex_lines = ["test", "test\ntest", "テスト🦀\n", "テスト\u{8}テスト"];
    let ex = ex_lines.map(|v| format!("'{}'", v)).join("\n") + "\n";

    cmd.write_stdin(input_lines).args(&["-n"]);
    cmd.assert().success().stdout(predicate::eq(ex.as_bytes()));
    Ok(())
}

#[test]
fn disable_escape_chars_and_joined_by_null() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::cargo_bin("xquo")?;
    let lines = ["test", "test\ntest", "テスト🦀\n", "テスト\u{8}テスト"];
    let input_lines = lines.join("\0");
    let ex_lines = ["test", "test\ntest", "テスト🦀\n", "テスト\u{8}テスト"];
    let ex = ex_lines.map(|v| format!("'{}'", v)).join("\0") + "\0";

    cmd.write_stdin(input_lines).args(&["-n", "-o", "null"]);
    cmd.assert().success().stdout(predicate::eq(ex.as_bytes()));
    Ok(())
}

#[test]
fn use_large_data_that_is_shuffle_lines_in_parallel_mode() -> Result<(), Box<dyn std::error::Error>>
{
    let mut cmd = Command::cargo_bin("xquo")?;
    let mut lines = Vec::<String>::new();
    for i in 0..10000 {
        lines.push(format!("{:04}", i));
    }
    let input_lines = lines.join("\0");
    let mut ex_lines = Vec::<String>::new();
    for v in lines {
        ex_lines.push(format!("'{}'", v));
    }
    // let ex = ex_lines.join("\n") + "\n";
    // ソートすると空行が先頭に移動されるので。
    let ex = format!("\n{}", ex_lines.join("\n"));

    cmd.write_stdin(input_lines).args(["-w", "2", "-b", "10"]);

    cmd.assert().success();

    let a = cmd.assert();
    let mut out_lines: Vec<String> = from_utf8(&a.get_output().stdout)
        .unwrap()
        .split('\n')
        .map(|v| v.to_string())
        .collect();
    out_lines.sort();

    assert_eq!(ex, out_lines.join("\n"));
    Ok(())
}

//#[test]
// fn file_doesnt_exist() -> Result<(), Box<dyn std::error::Error>> {
//     let mut cmd = Command::cargo_bin("xquo")?;
//
//     cmd.assert().failure().stderr(predicate::str::contains(
//         "The following required arguments were not provided",
//     ));
//
//     Ok(())
// }
