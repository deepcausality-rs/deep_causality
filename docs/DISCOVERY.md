# DeepCausality Discovery: Causal Discovery Language (CDL)

`deep_causality_discovery` provides a type-safe pipeline for inferring causal structures from observational data. It implements the **Causal Discovery Language (CDL)**, a fluent API that guides users through the discovery process while enforcing correct sequencing at compile time.

---

## 🏗️ The Problem: Causal Discovery

Causal Discovery is the process of analyzing data to reconstruct the underlying causal graph (DAG). This is complex because:
1.  Data is often messy (missing values, noise).
2.  Not all variables are relevant (feature selection).
3.  Algorithms are sensitive to data quality and hyperparameters.
4.  The process involves multiple distinct stages that must happen in order.

---

## 🧩 The Solution: The CDL Pipeline

The **Causal Discovery Language (CDL)** uses the **Typestate Pattern** to model the discovery pipeline. This ensures you cannot, for example, run causal discovery before cleaning your data.

### The Pipeline Stages

| Stage | Type State | Description |
|-------|------------|-------------|
| **1. Initialize** | `NoData` | Configure pipeline settings. |
| **2. Load** | `WithData` | Load raw data from CSV or Parquet into a Causal Tensor. |
| **3. Clean** | `WithCleanedData` | Handle missing values and invalid entries. |
| **4. Select** | `WithFeatures` | (Optional) Use MRMR to select relevant features. |
| **5. Discover** | `WithCausalResults` | Run the SURD algorithm to find causal links. |
| **6. Analyze** | `WithAnalysis` | Analyze the results (metrics, stability). |
| **7. Report** | `Finalized` | Generate a comprehensive PDF/JSON report. |

### Example Usage

```rust
let report = CDL::new()
    .config(my_config)
    .load_csv("data.csv")?       // Returns CDL<WithData>
    .clean_data(my_cleaner)?     // Returns CDL<WithCleanedData>
    .select_features(my_mrmr)?   // Returns CDL<WithFeatures>
    .discover_causality(surd)?   // Returns CDL<WithCausalResults>
    .analyze_results(analyzer)?  // Returns CDL<WithAnalysis>
    .compile_report()?;          // Returns CdlReport
```

---

## 📊 Key Algorithms

### MRMR (Minimum Redundancy Maximum Relevance)
Used for **Feature Selection**. It identifies variables that are highly correlated with the target (Relevance) but not correlated with each other (Redundancy). This ensures a compact and efficient causal model.

### SURD (Structural Unit Representation of Dependency)
The core **Causal Discovery Algorithm**. It analyzes the data tensor to infer directional dependencies between variables, effectively constructing the edges of the causal graph.

---

## 📈 Data Handling

The pipeline relies on `deep_causality_tensor` for efficient data storage and manipulation.
*   **Imputation**: Can handle missing values via mean, median, or custom strategies.
*   **Discretization**: Can convert continuous data into discrete interaction buckets.
*   **Normalization**: Ensures data is on a comparable scale.

---

## 📄 The Output: CDL Report

The process culminates in a `CdlReport` containing:
*   **Metadata**: Dataset stats, configuration.
*   **Causal Graph**: The discovered nodes and edges.
*   **Metrics**: Confidence scores, p-values (if applicable).
*   **Analysis**: Interpretations of the findings.

---

## Summary

`deep_causality_discovery` turns the complex, multi-step process of causal discovery into a safe, linear, and robust workflow. It bridges the gap between raw data files and the initialized `deep_causality_ethos` or `deep_causality_core` models.
