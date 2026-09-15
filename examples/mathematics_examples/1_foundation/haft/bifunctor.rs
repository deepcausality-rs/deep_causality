/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_haft::{Bifunctor, ResultUnboundWitness};

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
    let err = api_response_err.unwrap_err();
    assert_eq!(err.code, 404);
    assert_eq!(err.message, "User 99 not found");
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
