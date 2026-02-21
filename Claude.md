# CLAUDE.md — FibRust (FibCalc-rs)

Calculateur Fibonacci haute performance en Rust. Prototype académique exploitant le système de types, la sûreté mémoire et les abstractions zero-cost de Rust pour le calcul numérique.

## Projet

- **Workspace** : 7 crates Cargo
- **Rust** : 1.80+ (MSRV), édition 2021
- **Licence** : Apache 2.0
- **Taille** : ~14 000 lignes, 669+ tests, 96.1% couverture
- **Unsafe** : `unsafe_code = "forbid"` au niveau workspace

## Architecture (4 couches, 7 crates)

```
crates/
  fibcalc/              # Binaire : CLI parsing, dispatch, signal handling (clap)
  fibcalc-core/         # CŒUR : algorithmes, traits, stratégies, observers, registre
  fibcalc-bigfft/       # Multiplication FFT, nombres de Fermat, cache, allocateur
  fibcalc-orchestration/ # Exécution parallèle, sélection calculateur, analyse
  fibcalc-calibration/  # Auto-tuning adaptatif, micro-benchmarks, profils
  fibcalc-cli/          # Présentation CLI, barres de progression, complétion shell
  fibcalc-tui/          # Dashboard TUI interactif (ratatui, architecture Elm MVU)
tests/
  golden.rs             # Tests d'intégration golden file
  testdata/             # Données de référence
fuzz/                   # Cibles libfuzzer
```

## Stack technique

| Domaine | Technologie |
|---------|-------------|
| Big integers | `num-bigint` (défaut) / `rug` (GMP, optionnel) |
| Parallélisme | `rayon` (work-stealing), `crossbeam` (channels) |
| CLI | `clap` (derive mode) + `clap_complete` |
| TUI | `ratatui` 0.29 + `crossterm` 0.28 |
| Allocation | `bumpalo` (arena), pools thread-local |
| Synchro | `parking_lot` 0.12 |
| Logging | `tracing` + `tracing-subscriber` |
| Erreurs | `thiserror` (lib) / `anyhow` (bin) |
| Sérialisation | `serde` + `serde_json` |

## Algorithmes

1. **Fast Doubling** — O(log n), overhead minimal
2. **Matrix Exponentiation** — O(log n), opérations matricielles
3. **FFT-Based** — O(n log n) pour nombres massifs
4. **Cross-validation** : les 3 algorithmes tournent en parallèle, résultats comparés

## Profil de build

```toml
[profile.release]
lto = true
codegen-units = 1
strip = true
opt-level = 3
panic = "abort"
overflow-checks = true
```

Rustflags : `-C target-cpu=native`

## Testing

- **Golden file tests** : `tests/golden.rs` contre `fibonacci_golden.json`
- **Property-based** : `proptest` dans `fibcalc-core/tests/properties.rs`
- **E2E** : `assert_cmd` + `predicates` dans `fibcalc/tests/e2e.rs`
- **Benchmarks** : `criterion` 0.5 dans `fibcalc-core/benches/fibonacci.rs`
- **Fuzzing** : libfuzzer dans `fuzz/`

## Linting (Clippy pedantic)

```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
```

Exceptions autorisées : `module_name_repetitions`, `must_use_candidate`, `missing_errors_doc`, `missing_panics_doc`, `similar_names`, `struct_excessive_bools`, `items_after_statements`, `unused_self`, `if_not_else`, `redundant_else`.

## Directives pour Claude

1. **Zero unsafe** : `unsafe_code = "forbid"` — aucune exception. Trouver des alternatives safe.
2. **Ownership** : Respecter les patterns de transfert de propriété existants (`pointer stealing`, arena allocation).
3. **Tests obligatoires** : `cargo test --workspace` doit passer. Golden tests = source de vérité.
4. **Clippy pedantic** : `cargo clippy --workspace -- -D warnings` doit passer.
5. **Séparation des crates** : Respecter les frontières de dépendance. `fibcalc-core` ne dépend pas de `fibcalc-cli`.
6. **Traits** : Utiliser les traits existants (`FibCalculator`, `ProgressObserver`, `CalculatorFactory`). Ne pas créer de nouveaux traits sans justification.
7. **Performance** : Ne pas introduire de `clone()` inutiles, d'allocations dans les hot paths, ou de `Box<dyn>` là où les génériques suffisent.
8. **Modifications chirurgicales** : Codebase mature — pas de refactoring sans demande explicite.
