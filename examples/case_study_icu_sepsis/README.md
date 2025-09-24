# Case Study: ICU Sepsis Prediction

This crate serves as a practical case study within the `deep_causality` Rust monorepo, demonstrating the application of the computational causality library to a real-world medical dataset. It focuses on the preliminary data analysis stage for predicting sepsis in Intensive Care Unit (ICU) patients.

## Dataset

The case study utilizes a subset of the **PhysioNet Computing in Cardiology Challenge 2019 - Prediction of Sepsis** dataset. The original CSV data has been converted into a more efficient Parquet format for this example.

*   **Source:** [PhysioNet Computing in Cardiology Challenge 2019](https://www.kaggle.com/datasets/salikhussaini49/prediction-of_sepsis/data)
*   **Conversion:** CSV to Parquet via [tablab.app](https://www.tablab.app/csv/to/parquet)
*   **License:** The dataset is made available under the [ODC Open Database License (ODbL)](./data/doc/LICENSE.txt).

## Data Schema

The dataset comprises clinical time series data, including 40 clinical variables per hourly time window, patient demographics, and a sepsis label. Key columns include:

*   `Hour`: Elapsed time since ICU admission.
*   `HR`, `O2Sat`, `Temp`, `SBP`, `MAP`, `DBP`, `Resp`, `EtCO2`: Vital signs.
*   Various lab values (`BaseExcess`, `HCO3`, `pH`, `PaCO2`, `AST`, `BUN`, `Glucose`, `Lactate`, etc.).
*   Demographic information (`Age`, `Gender`, `Unit1`, `Unit2`, `HospAdmTime`, `ICULOS`).
*   `SepsisLabel`: The target variable for sepsis prediction.
*   `Patient_ID`: Unique identifier for each patient.

For a comprehensive list and descriptions of all columns, please refer to [`./notes/data.md`](./notes/data.md).

## Preliminary Data Analysis (First Stage Findings)

The initial analysis of the dataset reveals the following characteristics:

*   **Total unique Patient_IDs:** 40,336
*   **Total number of data records for all patients:** 1,552,210
*   **Average number of data records per patient:** 38

**The Problem:** The dataset consists of 40,336 individual mini-time series, each potentially possessing a unique causal fingerprint. Analyzing each of these time series in isolation for causal discovery is computationally prohibitive and carries a high risk of overfitting due to the sparse nature of individual patient data. This highlights the need for advanced causal reasoning techniques that can handle such complex, multi-patient, and multi-variable time-series data efficiently.

## Running the Example

To run the preliminary data analysis:

```bash
cargo run --bin case_study_icu_sepsis
```

This will execute the `first_stage` module, which loads the Parquet data, performs basic statistical analysis, and prints records for a specific `Patient_ID` (currently hardcoded to `42` for demonstration purposes).

## Code Structure

*   `src/main.rs`: The entry point of the application, which calls the `first_stage` module.
*   `src/first_stage.rs`: Contains the logic for loading the Parquet dataset, performing initial data analysis, and extracting specific patient records.

## License

The code in this crate is licensed under the **MIT License**.
The dataset used in this case study is licensed under the **ODC Open Database License (ODbL)**.
