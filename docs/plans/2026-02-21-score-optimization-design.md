# Design : Optimisation du Score Académique (92 → 97+)

**Date** : 2026-02-21
**Objectif** : Maximiser la note académique en ciblant les 3 catégories pénalisées
**Approche** : 4 agents Claude Code parallèles, séparation par couche architecturale

## Contexte

Évaluation académique actuelle : **92/100** (Grade A+).

| Catégorie | Note | Pénalité identifiée |
|-----------|------|---------------------|
| Architecture & Modularité | 24/25 | Couplage implicite avec GMP |
| Complexité Algorithmique | 25/25 | — |
| Fiabilité & Tests | 20/20 | — |
| Performance & Mémoire | 18/20 | Pas de core affinity, allocateurs non câblés |
| Documentation & Outillage | 5/10 | Friction installation (GMP), docs incomplètes |

## Impact projeté

| Catégorie | Avant | Après | Delta |
|-----------|-------|-------|-------|
| Architecture & Modularité | 24/25 | 25/25 | +1 |
| Performance & Mémoire | 18/20 | 20/20 | +2 |
| Documentation & Outillage | 5/10 | 9/10 | +4 |
| **Total** | **92/100** | **97/100** | **+5** |

## Architecture des agents

```
Temps ──────────────────────────────────────────────►
        Phase 1 (parallèle)              Phase 2 (séquentiel)
┌─────────────────────────────┐    ┌──────────────────────┐
│ Agent 1: Core/Perf          │    │                      │
│ Agent 2: Portabilité        │───►│ Agent 4: Validation  │
│ Agent 3: Documentation      │    │                      │
└─────────────────────────────┘    └──────────────────────┘
```

Chaque agent Phase 1 travaille sur un git worktree isolé. Agent 4 intervient après merge.

---

## Agent 1 — Core/Performance

**Branche** : `feat/core-perf`
**Crates** : `fibcalc-bigfft`, `fibcalc-core`, `fibcalc-tui`

### Tâche 1.1 — Câbler FFTBumpAllocator dans le chemin FFT

- **Fichiers** : `crates/fibcalc-bigfft/src/bump.rs`, `crates/fibcalc-bigfft/src/fft.rs`
- **Action** : Remplacer les allocations `vec![0u64; len]` dans le FFT par `bump.alloc_slice(len)`. Appeler `bump.reset()` entre les étapes FFT.
- **Contrainte** : Pas d'`unsafe`. L'API `bumpalo` est entièrement safe.
- **Vérification** : `cargo test -p fibcalc-bigfft` + benchmark criterion avant/après.

### Tâche 1.2 — Câbler BigIntPool dans les multiplications FFT

- **Fichiers** : `crates/fibcalc-bigfft/src/pool.rs`, `crates/fibcalc-bigfft/src/mul.rs`
- **Action** : Utiliser `pool.acquire(bit_len)` / `pool.release(bigint)` dans `mul()` et `sqr()` au lieu d'allouer de nouveaux `BigUint` à chaque appel.
- **Contrainte** : Retirer les `#[allow(dead_code)]` une fois câblé. Ne pas modifier la logique du pool lui-même (annotation existante "do not modify pool.rs").
- **Vérification** : Golden tests passent. Pool stats (hits/misses) > 0 dans un test dédié.

### Tâche 1.3 — Core Affinity TUI vs Compute

- **Fichiers** : `crates/fibcalc-tui/src/app.rs`, `crates/fibcalc-tui/Cargo.toml`
- **Action** : Ajouter `core_affinity` comme dépendance. Dans `run_tui()` :
  - Épingler le thread metrics + TUI sur le core 0
  - Épingler le thread compute sur les cores 1..N
  - Configurer `rayon::ThreadPoolBuilder` avec `start_handler` pour affinité dans le pool
- **Contrainte** : Fallback gracieux si `core_affinity::get_core_ids()` retourne `None`.
- **Vérification** : `cargo test -p fibcalc-tui`, test manuel TUI avec `n=1000000`.

### Tâche 1.4 — Connecter FFTMemoryEstimate au budget mémoire

- **Fichiers** : `crates/fibcalc-bigfft/src/memory_est.rs`, `crates/fibcalc-core/src/memory_budget.rs`
- **Action** : Exposer `estimate_fft_memory` dans l'API publique de `fibcalc-bigfft`. L'intégrer dans `MemoryEstimate::estimate(n)` quand n dépasse le seuil FFT.
- **Contrainte** : `fibcalc-core` dépend déjà de `fibcalc-bigfft`, dépendance valide.
- **Vérification** : Test unitaire comparant l'estimation avant/après pour `n=10_000_000`.

---

## Agent 2 — Portabilité

**Branche** : `feat/portability`
**Crates** : `fibcalc-core`, `fibcalc`, workspace `Cargo.toml`

### Tâche 2.1 — Nettoyer la déclaration workspace de rug

- **Fichier** : `Cargo.toml` (racine workspace)
- **Action** : Marquer `rug` comme `optional = true` au niveau workspace.
- **Vérification** : `cargo tree` ne montre pas `rug` sans `--features gmp`.

### Tâche 2.2 — Compléter le stub calculator_gmp.rs

- **Fichier** : `crates/fibcalc-core/src/calculator_gmp.rs`
- **Action** : Implémenter `GmpCalculator` (trait `CoreCalculator`, Fast Doubling avec `rug::Integer`), enregistrer dans `DefaultFactory` sous `#[cfg(feature = "gmp")]`.
- **Contrainte** : Compile uniquement sous `#[cfg(feature = "gmp")]`. Respecter `unsafe_code = "forbid"`.
- **Vérification** : `cargo test -p fibcalc-core --features gmp` + `cargo test -p fibcalc-core` (sans GMP).

### Tâche 2.3 — CI check dual-build

- **Fichier** : `.github/workflows/` (nouveau ou existant)
- **Action** : Matrice CI testant pure-Rust et GMP séparément.
- **Vérification** : Les deux jobs passent.

### Tâche 2.4 — Feature flag documentation

- **Fichiers** : `crates/fibcalc-core/src/lib.rs`, `README.md`
- **Action** : `#[doc(cfg(feature = "gmp"))]` sur le module GMP + section installation dans README.
- **Vérification** : `cargo doc --workspace` avec badge conditionnel.

---

## Agent 3 — Documentation

**Branche** : `feat/documentation`
**Crates** : Aucun code Rust. Uniquement docs et commentaires.

### Tâche 3.1 — Réécrire le README

- **Fichier** : `README.md`
- **Action** : Quick Start 3 lignes, chemins d'installation, exemples CLI, diagramme architecture, tableau benchmarks.
- **Contrainte** : Référencer `docs/` pour les détails, pas de duplication.
- **Vérification** : Build + run en < 2 min en suivant le README.

### Tâche 3.2 — Guide d'installation multi-plateforme

- **Fichier** : `docs/INSTALLATION.md` (nouveau)
- **Action** : Windows, Linux, macOS, Docker. Troubleshooting des 3 erreurs courantes.
- **Vérification** : Chaque commande testable.

### Tâche 3.3 — Couverture rustdoc

- **Fichiers** : `lib.rs` de `fibcalc-core`, `fibcalc-bigfft`, `fibcalc-orchestration`
- **Action** : Module-level docs, exemples sur traits publics, `#![warn(missing_docs)]`.
- **Contrainte** : Commentaires uniquement, aucune modification de code.
- **Vérification** : `cargo doc --workspace --no-deps` sans warnings.

### Tâche 3.4 — CHANGELOG et metadata Cargo

- **Fichiers** : `docs/CHANGELOG.md`, tous les `Cargo.toml`
- **Action** : Entrée changelog, metadata `description`/`repository`/`keywords`/`categories`.
- **Vérification** : `cargo package --list -p fibcalc` sans warnings metadata.

---

## Agent 4 — Validation & Intégration

**Branche** : `feat/validation` (basée sur merge des 3 branches)
**Portée** : Tout le workspace. Séquentiel après Phase 1.

### Tâche 4.1 — Benchmarks avant/après

- Capturer baselines sur `main`, re-lancer après merge.
- Rapport comparatif : n=1000, n=100_000, n=1_000_000.
- **Livrable** : `docs/BENCHMARK_REPORT.md`

### Tâche 4.2 — Suite de tests complète

Exécuter séquentiellement :
1. `cargo test --workspace`
2. `cargo test --workspace --features gmp`
3. `cargo test --workspace -- --ignored`
4. `cargo clippy --workspace -- -D warnings`
5. `cargo clippy --workspace --features gmp -- -D warnings`
6. `cargo doc --workspace --no-deps`

**Critère** : Les 6 commandes passent exit code 0.

### Tâche 4.3 — Vérification des allocateurs câblés

- Test d'intégration : `fib(1_000_000)` via FFT, vérifier pool stats hits > 0 et bump reset count > 0.
- **Fichier** : `tests/allocator_integration.rs`

### Tâche 4.4 — Vérification core affinity

- Test fallback gracieux quand `core_affinity` retourne `None`.
- Test single-core sans panic.
- **Fichier** : `crates/fibcalc-tui/src/app.rs` (module test existant)

### Tâche 4.5 — Code review croisé

Vérifier : pas de `clone()` inutiles, pas d'`unsafe`, `deny.toml` à jour, `#[allow(dead_code)]` retirés correctement, feature gates corrects.

### Tâche 4.6 — Scoring final

- Réévaluer selon la grille académique.
- **Livrable** : `docs/SCORING_SELF_ASSESSMENT.md`

---

## Contraintes transversales

- **`unsafe_code = "forbid"`** — aucune exception, tous agents.
- **Clippy pedantic** — zéro warning, tous agents.
- **Golden tests** — source de vérité pour l'exactitude des résultats.
- **Frontières de crates** — respecter le graphe de dépendances existant.
- **Git worktrees** — chaque agent Phase 1 travaille dans un worktree isolé pour éviter les conflits.
