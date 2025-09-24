## Sepsis Challenge 2019 Dataset Schema

This table provides a comprehensive overview of the clinical time series data used in the PhysioNet/Computing in
Cardiology Challenge 2019 Sepsis Prediction Task. The dataset contains 40 clinical variables per hourly time window for
each patient, along with patient demographics and a sepsis label.

| Column Name        | Description                                                                                                                      | Category    |
|:-------------------|:---------------------------------------------------------------------------------------------------------------------------------|:------------|
| `Hour`             | The elapsed time in hours since ICU admission for a given patient.                                                               | Time Index  |
| `HR`               | Heart rate (beats per minute)                                                                                                    | Vital Sign  |
| `O2Sat`            | Pulse oximetry (%)                                                                                                               | Vital Sign  |
| `Temp`             | Temperature (deg C)                                                                                                              | Vital Sign  |
| `SBP`              | Systolic BP (mm Hg)                                                                                                              | Vital Sign  |
| `MAP`              | Mean arterial pressure (mm Hg)                                                                                                   | Vital Sign  |
| `DBP`              | Diastolic BP (mm Hg)                                                                                                             | Vital Sign  |
| `Resp`             | Respiration rate (breaths per minute)                                                                                            | Vital Sign  |
| `EtCO2`            | End tidal carbon dioxide (mm Hg)                                                                                                 | Vital Sign  |
| `BaseExcess`       | Excess bicarbonate (mmol/L)                                                                                                      | Lab Value   |
| `HCO3`             | Bicarbonate (mmol/L)                                                                                                             | Lab Value   |
| `FiO2`             | Fraction of inspired oxygen (%)                                                                                                  | Lab Value   |
| `pH`               | pH                                                                                                                               | Lab Value   |
| `PaCO2`            | Partial pressure of carbon dioxide from arterial blood (mm Hg)                                                                   | Lab Value   |
| `SaO2`             | Oxygen saturation from arterial blood (%)                                                                                        | Lab Value   |
| `AST`              | Aspartate transaminase (IU/L)                                                                                                    | Lab Value   |
| `BUN`              | Blood urea nitrogen (mg/dL)                                                                                                      | Lab Value   |
| `Alkalinephos`     | Alkaline phosphatase (IU/L)                                                                                                      | Lab Value   |
| `Calcium`          | Calcium (mg/dL)                                                                                                                  | Lab Value   |
| `Chloride`         | Chloride (mmol/L)                                                                                                                | Lab Value   |
| `Creatinine`       | Creatinine (mg/dL)                                                                                                               | Lab Value   |
| `Bilirubin_direct` | Direct bilirubin (mg/dL)                                                                                                         | Lab Value   |
| `Glucose`          | Serum glucose (mg/dL)                                                                                                            | Lab Value   |
| `Lactate`          | Lactic acid (mg/dL)                                                                                                              | Lab Value   |
| `Magnesium`        | Magnesium (mmol/dL)                                                                                                              | Lab Value   |
| `Phosphate`        | Phosphate (mg/dL)                                                                                                                | Lab Value   |
| `Potassium`        | Potassium (mmol/L)                                                                                                               | Lab Value   |
| `Bilirubin_total`  | Total bilirubin (mg/dL)                                                                                                          | Lab Value   |
| `TroponinI`        | Troponin I (ng/mL)                                                                                                               | Lab Value   |
| `Hct`              | Hematocrit (%)                                                                                                                   | Lab Value   |
| `Hgb`              | Hemoglobin (g/dL)                                                                                                                | Lab Value   |
| `PTT`              | Partial thromboplastin time (seconds)                                                                                            | Lab Value   |
| `WBC`              | Leukocyte count (count/L)                                                                                                        | Lab Value   |
| `Fibrinogen`       | Fibrinogen concentration (mg/dL)                                                                                                 | Lab Value   |
| `Platelets`        | Platelet count (count/mL)                                                                                                        | Lab Value   |
| `Age`              | Age (years)                                                                                                                      | Demographic |
| `Gender`           | Female (0) or male (1)                                                                                                           | Demographic |
| `Unit1`            | Administrative identifier for ICU unit (MICU); false (0) or true (1)                                                             | Demographic |
| `Unit2`            | Administrative identifier for ICU unit (SICU); false (0) or true (1)                                                             | Demographic |
| `HospAdmTime`      | Time between hospital and ICU admission (hours since ICU admission)                                                              | Demographic |
| `ICULOS`           | ICU length of stay (hours since ICU admission)                                                                                   | Demographic |
| `SepsisLabel`      | **Target Variable:** For septic patients, `1` if `t ≥ t_sepsis - 6` and `0` if `t < t_sepsis - 6`. For non-septic patients, `0`. | Outcome     |
| `Patient_ID`       | Unique identifier for each patient.                                                                                              | Identifier  |
