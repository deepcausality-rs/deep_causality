# Quickstart: Causal Discovery DSL

This document provides a quick overview of how to use the Causal Discovery DSL, including primary user stories and acceptance scenarios.

## Primary User Story

A user wants to perform causal discovery on their data, from loading to formatting results, using a streamlined, composable DSL.

## Acceptance Scenarios (Integration Test Scenarios)

These scenarios will serve as the basis for integration tests to validate the end-to-end functionality of the DSL.

1.  **Given** a CSV file with data, **When** the user defines a CQD process to load, feature select, causal discover, analyze, and format, **Then** the process runs successfully and prints formatted causal insights.
2.  **Given** a Parquet file with data, **When** the user defines a CQD process to load, feature select, causal discover, analyze, and format, **Then** the process runs successfully and prints formatted causal insights.
3.  **Given** a CQD process with a `feat_select` step using mRMR, **When** the process is executed, **Then** mRMR is applied to the data before causal discovery.
4.  **Given** a CQD process with a `causal_discovery` step using SURD, **When** the process is executed, **Then** SURD is applied to the data.
5.  **Given** SURD results, **When** the `analyze` step is executed, **Then** recommendations for converting SURD results to Causaloids are generated.
6.  **Given** analyzed results, **When** the `finalize` step is executed with a console printer, **Then** the results are printed to the console in a human-readable format.

## Contract Tests (Placeholder)

Contract tests will be generated based on the defined traits and `CQD` method signatures to ensure adherence to the API contracts. These tests will initially fail as no implementation exists.
