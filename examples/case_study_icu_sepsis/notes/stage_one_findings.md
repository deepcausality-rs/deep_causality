# Findings of the first stage


* Total unique Patient_IDs: 40336
* Total number of data records for all patients: 1552210
* Average number of data records per patient: 38
* Total unique patients with SepsisLabel = 1: 2932
* Percentage of patients with SepsisLabel = 1: 7.27%

**The Problem:** You have 40,336 individual mini-time series, each potentially having a unique causal fingerprint. Analyzing each time series in isolation for discovery is computationally prohibitive and prone to overfitting on sparse data.

A 7.27% prevalence rate for sepsis is a highly imbalanced dataset. This is a well-known nightmare for conventional
statistical and machine learning methods

In a sepsis context, predicting "no sepsis" 92.73% of the time, when 7.27% of patients actually have sepsis,
means you are missing over 7% of sepsis cases. This is an unacceptably high rate of false negatives, leading to
preventable deaths.

