// Copyright (c) "2023" . Marvin Hansen <marvin.hansen@gmail.com> All rights reserved.

use deep_causality::errors::BuildError;

fn parse_data(input: i32) -> Result<i32, BuildError> {
    match input {
        0 => Ok(0),
        x => Err(BuildError(
            format!("unexpected number {}", x),
        )),
    }
}


#[test]
fn test_build_error() {
    let result = parse_data(1);
    let build_error = result.unwrap_err();
    assert_eq!(build_error.to_string(), format!("BuildError: unexpected number {}", 1));
}