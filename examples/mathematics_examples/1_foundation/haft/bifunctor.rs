/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

//! # `Bifunctor`: two channels, mapped in one step
//!
//! Some containers carry two independent type parameters, and both sides often need transforming
//! at the same boundary. `Bifunctor::bimap` takes one function per side and does it in one step.
//!
//! ```text
//! ResultUnboundWitness   Result<A, B>   the success channel and the error channel
//! Tuple2Witness          (A, B)         the payload and whatever travels beside it
//! ```
//!
//! Both are `HKT2Unbound` witnesses: the two parameters are unrelated and neither carries a bound,
//! so `bimap` can send each side anywhere. The setting is an API boundary, where a domain type
//! becomes a DTO, a domain error becomes an HTTP error, and the timing that rode along with them
//! becomes a header.

use deep_causality_haft::{Bifunctor, ResultUnboundWitness, Tuple2Witness};

// ============================================================================
// Domain: API Response Handling
// ============================================================================

fn main() {
    print_header();

    // ------------------------------------------------------------------------
    // Bifunctor: Dual-Track Processing
    //
    // ENGINEERING VALUE:
    // In many systems (especially Web APIs), you have types with TWO generic parameters:
    // Result<T, E>, Either<L, R>, Tuple<A, B>.
    //
    // You often need to transform BOTH sides simultaneously:
    // - Success: Domain Object -> DTO (Data Transfer Object)
    // - Error: Domain Error -> API Error (HTTP Code + Message)
    //
    // Bifunctor (`bimap`) allows you to do this in a single, declarative step.
    // ------------------------------------------------------------------------
    // Scenario 1: Successful Operation
    let success_result: Result<DomainUser, DomainError> = Ok(DomainUser {
        id: 42,
        username: "alice".to_string(),
        email: "alice@example.com".to_string(),
    });

    let original_success = success_result.clone();

    // Transformation Logic
    let to_dto = |u: DomainUser| UserDto {
        id: u.id.to_string(),
        display_name: u.username.to_uppercase(),
    };

    let to_api_error = |e: DomainError| match e {
        DomainError::UserNotFound(id) => ApiError {
            code: 404,
            message: format!("User {} not found", id),
        },
        DomainError::PermissionDenied => ApiError {
            code: 403,
            message: "Access denied".to_string(),
        },
        DomainError::DatabaseError(_) => ApiError {
            code: 500,
            message: "Internal server error".to_string(),
        },
    };

    // Apply bimap: Transform T -> T' AND E -> E'
    let api_response_ok: Result<UserDto, ApiError> =
        ResultUnboundWitness::bimap(success_result, to_dto, to_api_error);

    print_success(&original_success, &api_response_ok);
    assert_eq!(
        api_response_ok
            .as_ref()
            .map(|dto| dto.display_name.as_str()),
        Ok("ALICE")
    );

    // Scenario 2: Failed Operation. The same transformation logic is reused.
    let error_result: Result<DomainUser, DomainError> = Err(DomainError::UserNotFound(99));
    let original_error = error_result.clone();
    let api_response_err: Result<UserDto, ApiError> =
        ResultUnboundWitness::bimap(error_result, to_dto, to_api_error);

    print_failure(&original_error, &api_response_err);
    match api_response_err {
        Err(ref err) => {
            assert_eq!(err.code, 404);
            assert_eq!(err.message, "User 99 not found");
        }
        Ok(ref dto) => panic!("the error channel carried a payload: {dto:?}"),
    }

    // ------------------------------------------------------------------------
    // Scenario 3: the same move on a pair.
    //
    // `Tuple2Witness` is the other `HKT2Unbound` witness. A handler returns its payload beside
    // the time it took, and both sides cross the API boundary at once: the payload becomes a DTO
    // and the timing becomes the header value it is reported as.
    // ------------------------------------------------------------------------
    let handler_output: (DomainUser, u32) = (
        DomainUser {
            id: 7,
            username: "bob".to_string(),
            email: "bob@example.com".to_string(),
        },
        18,
    );
    let original_pair = handler_output.clone();

    let to_header = |millis: u32| format!("{millis}ms");
    let (dto, timing): (UserDto, String) = Tuple2Witness::bimap(handler_output, to_dto, to_header);

    print_pair(&original_pair, &dto, &timing);
    assert_eq!(dto.display_name, "BOB");
    assert_eq!(timing, "18ms");
}

// -----------------------------------------------------------------------------------------
// Printing
// -----------------------------------------------------------------------------------------

fn print_header() {
    println!("=== DeepCausality HKT: Bifunctor Pattern ===\n");
    println!("--- API Response Normalization ---");
}

fn print_success(original: &Result<DomainUser, DomainError>, response: &Result<UserDto, ApiError>) {
    println!("Original Success: {original:?}");
    println!("API Response (OK): {response:#?}");
}

fn print_failure(original: &Result<DomainUser, DomainError>, response: &Result<UserDto, ApiError>) {
    println!("\nOriginal Error:   {original:?}");
    println!("API Response (Err): {response:#?}");
}

fn print_pair(original: &(DomainUser, u32), dto: &UserDto, timing: &str) {
    println!("\n--- The same move on a pair, through Tuple2Witness ---");
    println!("Handler output:   ({:?}, {} ms)", original.0, original.1);
    println!("API Response:     {dto:?}");
    println!("Timing header:    {timing}");
}

#[derive(Debug, Clone, PartialEq)]
struct DomainUser {
    id: u32,
    username: String,
    email: String,
}

#[derive(Debug, Clone, PartialEq)]
struct UserDto {
    id: String,
    display_name: String,
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
enum DomainError {
    UserNotFound(u32),
    DatabaseError(String),
    PermissionDenied,
}

#[derive(Debug, Clone, PartialEq)]
struct ApiError {
    code: u16,
    message: String,
}
