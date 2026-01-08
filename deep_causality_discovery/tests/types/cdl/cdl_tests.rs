/*
 * SPDX-License-Identifier: MIT
 * Copyright (c) 2023 - 2026. The DeepCausality Authors and Contributors. All Rights Reserved.
 */

use deep_causality_algorithms::feature_selection::mrmr::MrmrResult;
use deep_causality_algorithms::surd::MaxOrder;
use deep_causality_discovery::CdlBuilder;
use std::io::Write;
use tempfile::NamedTempFile;

// Helper to create a dummy CSV file
fn create_test_csv_file(content: &str) -> NamedTempFile {
    let mut file = NamedTempFile::new().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    // Rename to .csv so extension detection works
    let path = file.path().to_owned();
    let new_path = path.with_extension("csv");
    std::fs::rename(&path, &new_path).unwrap();
    // Re-open/create tempfile wrapper to ensure cleanup, but we renamed it.
    // Tempfile cleanup might fail or miss renaming.
    // Easier: use Builder suffix.

    let mut file = tempfile::Builder::new().suffix(".csv").tempfile().unwrap();
    file.write_all(content.as_bytes()).unwrap();
    file
}

#[test]
fn test_cdl_pipeline_success() {
    // 3 columns, 4 rows (plus header)
    let content = "c1,c2,c3\n1.0,2.0,3.0\n4.0,5.0,6.0\n7.0,8.0,9.0\n10.0,11.0,12.0";
    let file = create_test_csv_file(content);
    let path = file.path().to_str().unwrap().to_string();

    let effect = CdlBuilder::build()
        // Load, target=2
        .bind(|cdl| cdl.load_data(&path, 2, vec![]))
        // Feature select (mock closure)
        .bind(|cdl| {
            cdl.feature_select(|_tensor| {
                // Return dummy result: select col 0 and 1
                // MrmrResult construction
                Ok(MrmrResult::new(vec![(0, 0.5), (1, 0.3)]))
            })
        })
        // Causal Discovery (mock closure)
        .bind(|cdl| {
            cdl.causal_discovery(|_tensor| {
                // Return dummy SurdResult
                // Mocking SurdResult might be hard if fields are private.
                // Using default/empty construction if available or via minimal valid call.
                // Actually, main.rs calls surd_states_cdl directly.
                // Here we can return a constructed Result.
                // If SurdResult has no public constructor, we might need to rely on a real call or an unsafe workaround?
                // Wait, SurdResult IS constructible in algorithms crate?
                // In main.rs update plan, we used `surd_states_cdl`.
                // Let's use `surd_states_cdl` on a small tensor if possible, or assume we can construct it.
                // `deep_causality_algorithms::surd::SurdResult` fields valid?
                // Actually, let's just use `Err` to verify flow or return a minimal valid object if we can.
                // Or use the real `surd_states_cdl`! It depends on `deep_causality_algorithms` which is available.

                // For now, let's try to pass a trivial tensor to real algorithm or mocked result.
                // If we simply return Ok(SurdResult::default()), verify if Default is derived.
                // Assuming Default is not derived for SurdResult.
                // We'll try to use a real call on the small tensor passed in.
                deep_causality_algorithms::surd::surd_states_cdl(_tensor, MaxOrder::Max)
                    .map_err(Into::into)
            })
        })
        // Analyze
        .bind(|cdl| cdl.analyze())
        // Finalize
        .bind(|cdl| cdl.finalize());

    if let Err(e) = &effect.inner {
        println!("Error: {:?}", e);
    }
    assert!(effect.inner.is_ok());

    let report = effect.inner.unwrap();
    assert_eq!(report.records_processed, 4);
    assert_eq!(report.dataset_path, path);
}

#[test]
fn test_cdl_load_data_error() {
    let effect = CdlBuilder::build().bind(|cdl| cdl.load_data("non_existent_file.csv", 0, vec![]));

    assert!(effect.inner.is_err());
    // match specific error type if needed
}

#[test]
fn test_cdl_records_count_propagation() {
    let content = "a,b\n1,1\n2,2\n3,3";
    let file = create_test_csv_file(content);
    let path = file.path().to_str().unwrap();

    let pipe = CdlBuilder::build().bind(|cdl| cdl.load_data(path, 1, vec![]));

    // Check intermediate state if possible.
    // CDL struct fields are public now?
    // WithData struct definition in mod.rs: `pub struct WithData { ... }`

    match pipe.inner {
        Ok(cdl) => {
            // cdl is CDL<WithData>
            assert_eq!(cdl.state.records_count, 3);
        }
        Err(e) => panic!("Failed to load: {:?}", e),
    }
}
