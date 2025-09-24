# Causal Methods for Early Sepsis Detection

## Data

Data Records:
* Total unique Patient_IDs: 40336
* Total number of data records for all patients: 1552210
* Average number of data records per patient: 38
* Total unique patients with SepsisLabel = 1: 2932
* Percentage of patients with SepsisLabel = 1: 7.27%

### Problem: Highly Imbalanced data.

In a sepsis context, predicting "no sepsis" 92.73% of the time, when 7.27% of patients actually have sepsis,
means you are missing over 7% of sepsis cases. This is an unacceptably high rate of false negatives, leading to
preventable deaths.

## DeepCausality's Context-First Causality

1. Focus on Causal Mechanism, Not Statistical Correlation:

* EPP Way: Instead of trying to find statistical patterns that distinguish the 7.27% from the 92.73% (which is hard due
  to imbalance), DeepCausality focuses on discovering the underlying causal mechanisms that lead to sepsis. It's not
  learning a correlational boundary; it's learning the process of becoming septic.
* Value: mRMR + SURD will focus on identifying biomarkers that are causally relevant to sepsis progression, regardless
  of their prevalence. It's looking for the 'signal' of sepsis, not the 'noise' of healthy patients.

2. No Distributional Assumptions:

* EPP Way: Your CausalTensor and SURD algorithms are built to handle arbitrary, non-parametric probability
  distributions. They do not assume Gaussianity or other specific statistical shapes that are easily broken by
  imbalance.
* Value: This means the causal insights derived by SURD (unique, redundant, synergistic influences) are robust and
  valid, even on severely imbalanced data. You are not making statistical assumptions that the data cannot support.

3. Context-First for Personalized Triggers:

* EPP Way: Each patient's journey is modeled as a dynamic Context hypergraph. The CausaloidGraph and its Causaloids are
  triggered by individual patient states, not population-level averages. A Causaloid looking for "Lactate rising
  rapidly" doesn't care if sepsis is rare in the general population; it cares if this patient's lactate is rising.
* Value: This enables truly personalized, early detection. The rarity of sepsis in the overall population does not blind
  the model to the clear causal signals in an individual patient.