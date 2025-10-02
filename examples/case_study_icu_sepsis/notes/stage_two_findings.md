Stage Two: Feature Engineering

Observations and Notes

1. Column 8: EtCO2 was accidentally char encoded but contains double / float data and thus had to be converted into F64
2. Patient ID correlates strongly with the target (sepsis) because of the denormalized data records and thus had to be excluded.
3. The parquet data loading and conversion into CausalTensor<Option<f64>> is really fast.

Compile time:
time cargo build --release --bin case_study_icu_sepsis

Finished `release` profile [optimized] target(s) in 40.13s
real	0m40.209s

time cargo build --release --bin case_study_icu_sepsis --features parallel

Finished `release` profile [optimized] target(s) in 1m 13s
real	1m13.964s

Run
time cargo run --release --bin case_study_icu_sepsis
real	3m15.109s

time cargo run --release --bin case_study_icu_sepsis --features parallel
real	0m41.839s

Results:

## Full Data (Sepsis and Non-Sepsis cases)

Run second stage!
Explicitly excluded column Patient_ID (index 42).
Original number of columns: 43
New number of columns after filtering: 42
Target column index: 41
Selected features and their normalized scores (CDL):
- Feature: ICULOS (index: 40), Importance Score: 1.0000
- Feature: HospAdmTime (index: 39), Importance Score: 0.5562
- Feature: Unit2 (index: 38), Importance Score: 0.3997
- Feature: Unit1 (index: 37), Importance Score: 0.3278
- Feature: Gender (index: 36), Importance Score: 0.2515
- Feature: Age (index: 35), Importance Score: 0.2206

- Feature: Platelets (index: 34), Importance Score: 0.1787
- Feature: Fibrinogen (index: 33), Importance Score: 0.1627
- Feature: WBC (index: 32), Importance Score: 0.1389
- Feature: PTT (index: 31), Importance Score: 0.1287
- Feature: Hgb (index: 30), Importance Score: 0.1077
- Feature: TroponinI (index: 28), Importance Score: 0.0996
- Feature: Hct (index: 29), Importance Score: 0.0951

- Feature: Bilirubin_total (index: 27), Importance Score: 0.0888
- Feature: Phosphate (index: 25), Importance Score: 0.0820
- Feature: Potassium (index: 26), Importance Score: 0.0807
- Feature: Magnesium (index: 24), Importance Score: 0.0765
- Feature: Lactate (index: 23), Importance Score: 0.0731
- Feature: Glucose (index: 22), Importance Score: 0.0664
- Feature: Bilirubin_direct (index: 21), Importance Score: 0.0650
- Feature: Chloride (index: 19), Importance Score: 0.0605
- Feature: Creatinine (index: 20), Importance Score: 0.0589
- Feature: Calcium (index: 18), Importance Score: 0.0547
- Feature: Alkalinephos (index: 17), Importance Score: 0.0531
- Feature: BUN (index: 16), Importance Score: 0.0509
- Feature: AST (index: 15), Importance Score: 0.0490
- Feature: SaO2 (index: 14), Importance Score: 0.0481
- Feature: PaCO2 (index: 13), Importance Score: 0.0443
- Feature: FiO2 (index: 11), Importance Score: 0.0421
- Feature: HCO3 (index: 10), Importance Score: 0.0415
- Feature: Resp (index: 7), Importance Score: 0.0393
- Feature: pH (index: 12), Importance Score: 0.0385
- Feature: EtCO2 (index: 8), Importance Score: 0.0382
- Feature: BaseExcess (index: 9), Importance Score: 0.0377
- Feature: DBP (index: 6), Importance Score: 0.0368
- Feature: SBP (index: 4), Importance Score: 0.0348
- Feature: Temp (index: 3), Importance Score: 0.0334
- Feature: MAP (index: 5), Importance Score: 0.0325
- Feature: HR (index: 1), Importance Score: 0.0313

## Sepsis Only

Run second stage!
Explicitly excluded column Patient_ID (index 42).
Original number of columns: 43
New number of columns after filtering: 42
Target column index: 41
Selected features and their normalized scores (CDL):
- Feature: ICULOS (Index: 40), Importance Score: 1.0000
- Feature: HospAdmTime (Index: 39), Importance Score: 0.3398
- Feature: Unit2 (Index: 38), Importance Score: 0.1632
- Feature: Age (Index: 35), Importance Score: 0.0886
- Feature: Unit1 (Index: 37), Importance Score: 0.0782
- Feature: Gender (Index: 36), Importance Score: 0.0723

- Feature: Platelets (Index: 34), Importance Score: 0.0309
- Feature: Phosphate (Index: 25), Importance Score: 0.0254
- Feature: Resp (Index: 7), Importance Score: 0.0248
- Feature: HR (Index: 1), Importance Score: 0.0232
- Feature: Bilirubin_direct (Index: 21), Importance Score: 0.0221
- Feature: WBC (Index: 32), Importance Score: 0.0215
- Feature: PaCO2 (Index: 13), Importance Score: 0.0210

- Feature: EtCO2 (Index: 8), Importance Score: 0.0208
- Feature: Potassium (Index: 26), Importance Score: 0.0200
- Feature: Fibrinogen (Index: 33), Importance Score: 0.0202
- Feature: O2Sat (Index: 2), Importance Score: 0.0198
- Feature: Alkalinephos (Index: 17), Importance Score: 0.0195
- Feature: Bilirubin_total (Index: 27), Importance Score: 0.0190
- Feature: Hour (Index: 0), Importance Score: 0.0186
- Feature: BaseExcess (Index: 9), Importance Score: 0.0186
- Feature: Lactate (Index: 23), Importance Score: 0.0187
- Feature: Hgb (Index: 30), Importance Score: 0.0184
- Feature: pH (Index: 12), Importance Score: 0.0183
- Feature: Creatinine (Index: 20), Importance Score: 0.0181
- Feature: DBP (Index: 6), Importance Score: 0.0181
- Feature: Hct (Index: 29), Importance Score: 0.0176
- Feature: HCO3 (Index: 10), Importance Score: 0.0175
- Feature: TroponinI (Index: 28), Importance Score: 0.0173
- Feature: Chloride (Index: 19), Importance Score: 0.0171
- Feature: SBP (Index: 4), Importance Score: 0.0172
- Feature: FiO2 (Index: 11), Importance Score: 0.0170
- Feature: Glucose (Index: 22), Importance Score: 0.0170
- Feature: BUN (Index: 16), Importance Score: 0.0165
- Feature: Temp (Index: 3), Importance Score: 0.0164
- Feature: SaO2 (Index: 14), Importance Score: 0.0160
- Feature: MAP (Index: 5), Importance Score: 0.0160
- Feature: AST (Index: 15), Importance Score: 0.0157
- Feature: Calcium (Index: 18), Importance Score: 0.0152


## Non-Sepsis Only

Run second stage!
Explicitly excluded column Patient_ID (index 42).
Original number of columns: 43
New number of columns after filtering: 42
Target column index: 41
Selected features and their normalized scores (CDL):
- Feature: ICULOS (Index: 40), Importance Score: 1.0000
- Feature: HospAdmTime (Index: 39), Importance Score: 0.5360
- Feature: Unit2 (Index: 38), Importance Score: 0.4064
- Feature: Unit1 (Index: 37), Importance Score: 0.3303
- Feature: Gender (Index: 36), Importance Score: 0.2379
- Feature: Age (Index: 35), Importance Score: 0.2131

- Feature: Platelets (Index: 34), Importance Score: 0.1622
- Feature: Fibrinogen (Index: 33), Importance Score: 0.1507
- Feature: WBC (Index: 32), Importance Score: 0.1351
- Feature: PTT (Index: 31), Importance Score: 0.1212
- Feature: Hgb (Index: 30), Importance Score: 0.1057
- Feature: Hct (Index: 29), Importance Score: 0.0957

- Feature: TroponinI (Index: 28), Importance Score: 0.0874
- Feature: Bilirubin_total (Index: 27), Importance Score: 0.0782
- Feature: Potassium (Index: 26), Importance Score: 0.0739
- Feature: Phosphate (Index: 25), Importance Score: 0.0697
- Feature: Magnesium (Index: 24), Importance Score: 0.0670
- Feature: Lactate (Index: 23), Importance Score: 0.0636
- Feature: Glucose (Index: 22), Importance Score: 0.0587
- Feature: Bilirubin_direct (Index: 21), Importance Score: 0.0561
- Feature: Creatinine (Index: 20), Importance Score: 0.0533
- Feature: Calcium (Index: 18), Importance Score: 0.0491
- Feature: Chloride (Index: 19), Importance Score: 0.0468
- Feature: Alkalinephos (Index: 17), Importance Score: 0.0456
- Feature: BUN (Index: 16), Importance Score: 0.0418
- Feature: AST (Index: 15), Importance Score: 0.0400
- Feature: SaO2 (Index: 14), Importance Score: 0.0383
- Feature: PaCO2 (Index: 13), Importance Score: 0.0371
- Feature: pH (Index: 12), Importance Score: 0.0353
- Feature: FiO2 (Index: 11), Importance Score: 0.0324
- Feature: EtCO2 (Index: 8), Importance Score: 0.0314
- Feature: Resp (Index: 7), Importance Score: 0.0306
- Feature: HCO3 (Index: 10), Importance Score: 0.0300
- Feature: BaseExcess (Index: 9), Importance Score: 0.0298
- Feature: DBP (Index: 6), Importance Score: 0.0290
- Feature: MAP (Index: 5), Importance Score: 0.0285
- Feature: SBP (Index: 4), Importance Score: 0.0267
- Feature: HR (Index: 1), Importance Score: 0.0248
- Feature: Temp (Index: 3), Importance Score: 0.0245

Findings:

The selected features can be grouped into several categories, all of which are
known to be associated with sepsis:

1. Demographics and Administrative Data
* Top Features: ICULOS, HospAdmTime, Unit1/Unit2, Gender, Age.
* Analysis: It's not surprising that ICULOS (ICU Length of Stay) is the most
  important feature, as it is directly related to the patient's overall outcome.
  HospAdmTime can indicate the severity of illness upon ICU admission.
  Demographics like Age and Gender, and the type of ICU (Unit1/Unit2), are common
  risk factors in many medical prediction models.

2. Laboratory Values Indicating Organ Dysfunction
* Hematology (Blood): Platelets, Fibrinogen, WBC, PTT, Hgb, Hct.
    * Analysis: Sepsis often leads to disseminated intravascular coagulation
      (DIC) and other blood disorders. Therefore, markers for coagulation
      (Fibrinogen, PTT), inflammation (WBC), and platelet count are highly
      relevant.
* Liver Function: Bilirubin_total, Bilirubin_direct, AST, Alkalinephos.
    * Analysis: Liver dysfunction is a common complication of sepsis.
* Kidney Function: Creatinine, BUN.
    * Analysis: Acute kidney injury is another serious complication of sepsis.
* Cardiac Marker: TroponinI.
    * Analysis: Elevated troponin can indicate cardiac muscle damage, which can
      be caused by sepsis.

3. Metabolic and Respiratory Indicators
* Metabolic Markers: Lactate, Glucose, BaseExcess, HCO3.
    * Analysis: Elevated Lactate is a key indicator of tissue hypoperfusion and
      is a critical component in the definition of septic shock.
* Respiratory Markers: SaO2, PaCO2, FiO2, pH.
    * Analysis: Respiratory failure (ARDS) is a common and severe complication
      of sepsis. These markers reflect the patient's respiratory status.
* Electrolytes: Phosphate, Potassium, Magnesium, Chloride, Calcium.
    * Analysis: Electrolyte imbalances are common in critically ill patients and
      can be exacerbated by sepsis.

Analysis of the Results

1. Combined Dataset: The top 6 features are all demographic or administrative
   data (ICULOS, HospAdmTime, Unit1/2, Gender, Age). The first clinical
   measurements (Platelets, Fibrinogen, etc.) appear further down the list with
   significantly lower importance scores.

2. Non-Sepsis Only Dataset: The results are remarkably similar to the combined
   dataset. The same top 6 demographic/administrative features dominate, and the
   clinical variables have comparable, relatively low scores.

3. Sepsis-Only Dataset: This is where things get interesting.
    * The same demographic/administrative features are still at the top, but
      their importance scores are drastically lower (e.g., HospAdmTime drops
      from ~0.55 to ~0.34, and Age from ~0.22 to ~0.09).
    * More importantly, a much wider range of clinical variables now appear in
      the list with non-trivial importance scores (e.g., Platelets, Phosphate,
      Resp, HR, etc.). In the combined and non-sepsis datasets, many of these
      clinical features had scores so low they didn't even make the list.

The Problem

The strong predictive power of the demographic and administrative features in the combined dataset is masking the importance of the clinical variables.

Here's a breakdown of why this is happening:

* Confounding Variables: The demographic and administrative data (ICULOS,
  HospAdmTime, etc.) are likely strong confounders. They are correlated with
  both the clinical measurements and the outcome (sepsis). For example, a
  patient who has been in the ICU for a long time (ICULOS) is more likely to
  have both more clinical measurements taken and a higher chance of developing
  sepsis.
* Data Imbalance: With a 93% to 7% ratio, the model is heavily biased towards the
  non-sepsis cases. The features that are good at predicting "not sepsis" will
  dominate the feature selection process. Since the non-sepsis group is so large,
  the model learns that the demographic data is a very good predictor for the
  majority of the data.
* Masking Effect: Because the MRMR algorithm is trying to find a balance between
  relevance to the target and redundancy with other features, the strong,
  universally present demographic features get picked first. Once they are in the
  model, they "explain away" a lot of the variance, leaving less for the
  clinical variables to contribute, thus lowering their apparent importance.

What Happens When You Separate the Datasets?

* Non-Sepsis: When you run MRMR on only the non-sepsis data, the situation is
  largely the same as the combined set. The demographic features are still the
  best predictors for this large, relatively homogeneous group.
* 
* Sepsis: When you isolate the sepsis cases, you remove the overwhelming
  influence of the non-sepsis group. In this context, the model is forced to look
  for the subtle patterns within the sepsis patients. This is where the clinical
  variables (HR, Resp, WBC, etc.) become much more important, as they are the
  indicators that change as the condition progresses. The demographic data is
  still relevant, but its predictive power is diminished relative to the
  now-crucial clinical measurements.