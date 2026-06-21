<div align="center">

# Disentangling Latent Risk Pathways<br>via Bayesian Hypergraph Inference

**Official implementation · Yale University**

[![ICML 2026 Oral](https://img.shields.io/badge/ICML-2026%20Oral-00356B)](https://icml.cc/)
[![License: MIT](https://img.shields.io/badge/License-MIT-A8556E)](LICENSE)

[**🌐 Project page**](https://naomi-ding.github.io/BHPI/) &nbsp;·&nbsp; [**📄 Paper (arXiv)**](https://arxiv.org/abs/2606.07677) &nbsp;·&nbsp; [**📌 BibTeX**](#citation)

</div>

---

## 🧩 Overview

BHPI reframes multi-disease modeling as inferring a **latent hypergraph**: diseases group into overlapping *pathways* (hyperedges), and each risk factor acts on **pathways rather than individual diseases**. A disease's per–risk-factor effect is **composed from the pathways it belongs to**:

```math
\beta_{j,v} = d_v^{-1} \sum_{e} H_{v,e} \mu_{j,e}
```

A **repulsion prior** keeps the discovered pathways parsimonious and identifiable, and a **structured variational inference** scheme (Pólya–Gamma augmentation + CAVI) preserves the existence → membership → effect logic for calibrated posterior uncertainty over both the disease groupings and the risk-factor effects.

## 📁 Repository structure

```
BHPI/
├── BHPI.m              # core algorithm: repulsion-aware coordinate-ascent VI
├── simulate_design.m   # entry point: synthetic experiments + structural recovery
├── helper/             # synthetic data generation, hypergraph init, repulsion utilities
└── docs/               # project page (served via GitHub Pages)
```

## 🚀 Getting started

**Requirements:** MATLAB (R2023a+) with the Statistics and Machine Learning Toolbox.

Reproduce the synthetic structure-recovery experiments:

```matlab
simulate_design
```

This simulates data from a known latent hypergraph, fits BHPI, and reports structural recovery (incidence `H` and effect `γ`/`μ`) alongside predictive AUC against the baselines.

## 🛠 Usage

Initialize the variational parameters, fit the model, then predict and evaluate — as in [`simulate_design.m`](simulate_design.m):

```matlab
% 1. Initialize variational parameters (NNMF initialization recommended)
[initials] = cavi_initialization(seed_init, initial_method, E_hat, X_train, Y_train, []);

% 2. Fit the BHPI model
model = BHPI(X_train, Y_train, E_hat, max_iter, ...
             seed_init, initials, omega_repulsion, staged, ...
             fix_z, z_constraint, sigma2_alpha, ...
             warmup_iters, batch_size, t0, weights, tol, verbose);

% 3. Predict on held-out data
eta_val  = X_val * model.beta + model.alpha_mean;
prob_val = 1 ./ (1 + exp(-eta_val));

% 4. Score per-disease AUROC
AUROC = NaN(1, V);
for v = 1:V
    [~, ~, ~, AUROC(v)] = perfcurve(Y_val(:, v), prob_val(:, v), 1);
end
mean_auroc = mean(AUROC);
```

**Key parameters**

| Argument | Meaning |
| :--- | :--- |
| `E_hat` | Upper bound on the number of latent hyperedges; the model self-regularizes to fewer. |
| `omega_repulsion` | Repulsion strength; `> 0` disentangles redundant pathways. |
| `initials` | Starting values for the variational parameters (from `cavi_initialization`). |
| `model.beta` | Learned disease-specific risk-factor effects. |

See the header of [`BHPI.m`](BHPI.m) for the full argument list (`staged`, `fix_z`, `warmup_iters`, `batch_size`, …).

## ⏱️ Runtime & complexity

| Phase | Per-iteration complexity | UK Biobank ($N \approx 277\text{K}$) |
| :--- | :--- | :--- |
| **Training** | $\mathcal{O}(N \cdot E \cdot (P + V))$ | ~74 min · ~28 GB peak |
| **Inference** | efficient matrix ops | $< 0.1$ ms / sample |

> Measured on **4 × Intel Xeon 6342 cores, 60 GB RAM**; inference latency is on par with logistic regression.

## 🗄️ Data availability

The synthetic experiments are fully reproducible from this repository; the paper's real-data results use the **UK Biobank**, which requires approved access and cannot be redistributed here.

<a id="citation"></a>

## ✒️ Citation

```bibtex
@inproceedings{ding2026bhpi,
  title     = {Disentangling Latent Risk Pathways via Bayesian Hypergraph Inference},
  author    = {Ding, Shengxian and Gao, Haonan and Liu, Pangpang and Tian, Xinyuan and Zhao, Yize},
  booktitle = {Proceedings of the 43rd International Conference on Machine Learning (ICML)},
  series    = {Proceedings of Machine Learning Research},
  publisher = {PMLR},
  year      = {2026},
  eprint    = {2606.07677},
  archivePrefix = {arXiv}
}
```

## 📜 License

Released under the [MIT License](LICENSE).
