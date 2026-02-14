# PlanRefact.md — Plan de Refactorisation Exhaustif

> **Projet** : FibCalc-rs (15K lignes Rust, 7 crates, 690 tests)
> **Date** : 2026-02-14
> **Base** : Audit complet du code + `CLAUDE.md`
> **État initial** : 0 warnings clippy, 690 tests verts, 1 ignoré

---

## Suivi des tâches

> **Consigne** : Mettre à jour ce tableau à chaque tâche complétée avec succès. Cocher la case, inscrire la date et le commit associé.

| Tâche | Description | Statut | Date | Commit |
|-------|-------------|--------|------|--------|
| 1.1 | Audit `#![allow(dead_code)]` fibcalc-core | [x] | 2026-02-14 | pending |
| 1.2 | Audit `#![allow(dead_code)]` fibcalc-bigfft | [x] | 2026-02-14 | pending |
| 1.3 | Retirer `_strassen_threshold` | [x] | 2026-02-14 | pending |
| 1.4 | Dead code `errors.rs` / `version.rs` | [x] | 2026-02-14 | pending |
| 1.5 | Résoudre frameworks vs inline | [x] | 2026-02-14 | pending |
| 1.6 | Audit `CLIProgressReporter` inutilisé | [x] | 2026-02-14 | pending |
| 2.1 | Créer `ObjectPool<T>` générique | [x] | 2026-02-14 | pending |
| 2.2 | Helper thread-local pool | [x] | 2026-02-14 | pending |
| 2.3 | Migrer `CalculationStatePool` | [x] | 2026-02-14 | pending |
| 2.4 | Migrer `MatrixStatePool` | [x] | 2026-02-14 | pending |
| 3.1 | Default impl `DoublingStepExecutor` | [x] | 2026-02-14 | pending |
| 4.1 | Optimiser `normalize()` sans BigUint | [x] | 2026-02-14 | pending |
| 4.2 | Optimiser `shift_left()` sans BigUint | [x] | 2026-02-14 | pending |
| 4.3 | Optimiser `shift_right()` sans BigUint | [x] | 2026-02-14 | pending |
| 4.4 | Clones hot-path butterfly FFT | [x] | 2026-02-14 | pending |
| 4.5 | Évaluer `fermat_mul()` (optionnel) | [x] | 2026-02-14 | évalué: hot path confirmé, optimisation reportée (complexité élevée) |
| 4.6 | Renommer `clear_preserving_capacity()` | [x] | 2026-02-14 | pending |
| 5.1 | Extraire `check_memory_budget` | [x] | 2026-02-14 | pending |
| 5.2 | Éliminer duplication `execute_cli_logic` | [x] | 2026-02-14 | pending |
| 5.3 | Extraire setup calculateurs | [x] | 2026-02-14 | pending |
| 5.4 | Propager `FibError` dans orchestrateur | [x] | 2026-02-14 | pending |
| 6.1 | String → `&'static str` bridge TUI | [x] | 2026-02-14 | pending |
| 6.2 | Déduplication render `model.rs` | [x] | 2026-02-14 | pending |
| 6.3 | Audit `TUIProgressReporter` | [x] | 2026-02-14 | pending |
| 6.4 | Constante `LOG2_PHI` | [x] | 2026-02-14 | pending |
| 6.5 | Audit `ColorTheme` / `SparklineBuffer` | [x] | 2026-02-14 | pending |
| 7.1 | `CancellationToken` AtomicU64 → AtomicBool | [x] | 2026-02-14 | pending |
| 7.2 | Retirer `MatrixState::temp` | [x] | 2026-02-14 | pending |
| 7.3 | Déduplication `adjust()` | [x] | 2026-02-14 | pending |
| 7.4 | `FibIterator::from_index` O(log n) | [x] | 2026-02-14 | pending |
| 8.1 | FFT bench réel `microbench.rs` | [x] | 2026-02-14 | pending |
| 8.2 | Médiane `runner.rs` | [x] | 2026-02-14 | pending |
| 9.1 | Retirer `#[allow(...)]` inutiles | [x] | 2026-02-14 | audité: tous nécessaires |
| 9.2 | Tests pool générique | [x] | 2026-02-14 | pending |
| 9.3 | Benchmark non-régression | [x] | 2026-02-14 | pending |
| 9.4 | Vérification finale | [x] | 2026-02-14 | 668 tests, 0 warnings, fmt ok |

**Progression** : 36 / 36 tâches complétées (100%)

---

## Table des matières

1. [Vue d'ensemble](#1-vue-densemble)
2. [Epic 1 — Élimination du code mort et nettoyage](#epic-1--élimination-du-code-mort-et-nettoyage)
3. [Epic 2 — Extraction du pool d'objets générique](#epic-2--extraction-du-pool-dobjets-générique)
4. [Epic 3 — Déduplication des stratégies de multiplication](#epic-3--déduplication-des-stratégies-de-multiplication)
5. [Epic 4 — Performance Fermat/FFT](#epic-4--performance-fermatfft)
6. [Epic 5 — Refactorisation de l'orchestration app.rs](#epic-5--refactorisation-de-lorchestration-apprs)
7. [Epic 6 — Nettoyage TUI et CLI](#epic-6--nettoyage-tui-et-cli)
8. [Epic 7 — Micro-optimisations et nettoyage fibcalc-core](#epic-7--micro-optimisations-et-nettoyage-fibcalc-core)
9. [Epic 8 — Corrections calibration](#epic-8--corrections-calibration)
10. [Epic 9 — Qualité et couverture de tests](#epic-9--qualité-et-couverture-de-tests)
11. [Graphe de dépendances](#graphe-de-dépendances)
12. [Ordre d'exécution recommandé](#ordre-dexécution-recommandé)

---

## 1. Vue d'ensemble

### Métriques actuelles

| Métrique | Valeur |
|---|---|
| Lignes de code (Rust) | ~14 930 |
| Crates | 7 |
| Tests | 690 (1 ignoré) |
| Warnings clippy (pedantic) | 0 |
| `#![allow(dead_code)]` crate-level | 2 crates |
| Champs inutilisés (`_strassen_threshold`, `MatrixState::temp`) | 2 |
| `CancellationToken` utilise `AtomicU64` pour un bool | 1 |
| Fonctions mortes (`handle_error`, `version`, `CLIProgressReporter`) | 3 |
| `bench_fft()` identique à `bench_karatsuba()` (calibration factice) | 1 |
| Erreurs `FibError` → `String` perdant l'information de type | 1 |
| Médiane floor-only dans `runner.rs` (pair → biaisée) | 1 |
| Patterns de pool dupliqués | 2 (~120 lignes dupliquées) |
| `execute_doubling_step` dupliqué | 4 implémentations identiques |
| `adjust()` blocs dupliqués dans threshold manager | 6 blocs (~72 lignes) |
| Clones `FermatNum` dans la boucle butterfly FFT (hot path) | 2 par itération |
| Conversions BigUint dans les chemins chauds FFT | 5 fonctions |
| `FibIterator::from_index` O(n) évitable | 1 |

### Principes directeurs (issus de CLAUDE.md)

- **Changements chirurgicaux** : chaque ligne modifiée trace à un objectif de refactorisation
- **Simplicité d'abord** : réduire la complexité, pas en ajouter
- **Vérification systématique** : `cargo test --workspace` + `cargo clippy -- -W clippy::pedantic` après chaque tâche
- **Pas de surconception** : pas de nouvelles abstractions sauf si elles éliminent de la duplication mesurable

---

## Epic 1 — Élimination du code mort et nettoyage

**Motivation** : Deux crates utilisent `#![allow(dead_code)]` crate-level, masquant potentiellement du code réellement mort. Un champ `_strassen_threshold` est stocké mais jamais lu.

**Impact** : Réduction du bruit, meilleure visibilité du code réellement utilisé, réduction binaire.

### Tâche 1.1 — Auditer `#![allow(dead_code)]` dans `fibcalc-core`

- **Fichier** : `crates/fibcalc-core/src/lib.rs:5`
- **Action** : Retirer `#![allow(dead_code)]`. Compiler. Identifier les modules/fonctions réellement morts.
- **Modules suspects** : `arena.rs`, `common.rs`, `generator.rs`, `generator_iterative.rs`, `fft_wrappers.rs`, `doubling_framework.rs`, `matrix_framework.rs`
- **Décision** : Pour chaque item mort :
  - S'il est de l'infrastructure pour une fonctionnalité future (PRD), ajouter `#[allow(dead_code)]` localement avec un commentaire `// TODO: used by Phase X task Y`
  - S'il est remplacé (ex: `doubling_framework` remplacé par l'implémentation inline dans `fastdoubling.rs`), le retirer
- **Vérification** : `cargo test --workspace && cargo clippy -- -W clippy::pedantic`
- **Dépendances** : Aucune
- **Parallélisable avec** : Tâche 1.2

### Tâche 1.2 — Auditer `#![allow(dead_code)]` dans `fibcalc-bigfft`

- **Fichier** : `crates/fibcalc-bigfft/src/lib.rs:5`
- **Action** : Même approche que 1.1. Tous les modules sont `pub(crate)`, donc potentiellement morts si non appelés.
- **Vérification** : `cargo test --workspace && cargo clippy -- -W clippy::pedantic`
- **Dépendances** : Aucune
- **Parallélisable avec** : Tâche 1.1

### Tâche 1.3 — Retirer `_strassen_threshold` d'`AdaptiveStrategy`

- **Fichier** : `crates/fibcalc-core/src/strategy.rs:170-178`
- **Problème** : Le champ `_strassen_threshold` est stocké mais jamais lu. Le préfixe `_` confirme que c'est intentionnellement ignoré, mais c'est du code mort structurel.
- **Action** :
  - Si le Strassen est prévu dans le PRD : ajouter un `// TODO: Phase X` et garder
  - Sinon : retirer le champ, simplifier le constructeur
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune
- **Parallélisable avec** : 1.1, 1.2

### Tâche 1.4 — Retirer le code mort dans `errors.rs` et `version.rs`

- **Fichiers** :
  - `crates/fibcalc/src/errors.rs:9` — `handle_error(err: &FibError) -> i32`
  - `crates/fibcalc/src/version.rs:6` — `version()`, `full_version()`
- **Problème** : Ces fonctions publiques ne sont appelées nulle part dans le code de production. Elles n'apparaissent que dans leurs propres tests unitaires. `main.rs` utilise `anyhow::Result` directement, pas `handle_error`.
- **Action** :
  - Vérifier qu'aucun consommateur externe n'utilise ces fonctions (le crate `fibcalc` est un binaire, pas une bibliothèque partagée)
  - Si confirmé inutilisé : retirer les fonctions et leurs tests
  - Alternativement : si `handle_error` est prévu pour une future API, ajouter `#[allow(dead_code)]` localement avec un commentaire `// TODO`
- **Vérification** : `cargo test --workspace && cargo clippy -- -W clippy::pedantic`
- **Dépendances** : Aucune
- **Parallélisable avec** : 1.1, 1.2, 1.3

### Tâche 1.5 — Résoudre les frameworks vs implémentations inline

- **Fichiers** :
  - `crates/fibcalc-core/src/doubling_framework.rs` (~128 lignes)
  - `crates/fibcalc-core/src/matrix_framework.rs` (~66 lignes)
- **Problème** : Ces modules génériques dupliquent la logique déjà inline dans `fastdoubling.rs` et `matrix.rs`. Les versions inline sont supérieures (réutilisation des buffers via pool). Les frameworks allouent à chaque appel.
- **Action** :
  - Vérifier que rien n'utilise ces frameworks en production (seulement des tests internes)
  - Si confirmé inutilisé : retirer les modules et leurs tests
  - Si utilisé par l'algo FFT-based : refactorer pour que l'algo FFT utilise directement la stratégie
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Tâche 1.1 (l'audit de dead_code révélera les usages)

### Tâche 1.6 — Auditer `CLIProgressReporter` inutilisé dans le binaire

- **Fichier** : `crates/fibcalc-cli/src/presenter.rs:14`
- **Problème** : `CLIProgressReporter` implémente `ProgressReporter` mais n'est jamais instancié dans `app.rs` ni dans `main.rs`. Seul `CLIResultPresenter` est utilisé. Le reporter est testé mais jamais appelé en production.
- **Action** :
  - Si prévu pour une future intégration (PRD) : documenter avec un `// TODO`
  - Sinon : retirer `CLIProgressReporter` et ses tests
  - Note : cette tâche est liée à 6.3 (audit `TUIProgressReporter`)
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune

---

## Epic 2 — Extraction du pool d'objets générique

**Motivation** : `fastdoubling.rs` et `matrix.rs` contiennent chacun ~60 lignes de code de pool quasi-identiques (struct XxxPool + thread_local + acquire/release). C'est de la duplication structurelle.

**Impact** : ~120 lignes de code dupliqué → ~40 lignes de module générique + 2 instantiations.

### Tâche 2.1 — Créer `ObjectPool<T>` générique

- **Nouveau fichier** : `crates/fibcalc-core/src/pool.rs`
- **Contenu** :
  ```rust
  pub struct ObjectPool<T> {
      pool: Mutex<Vec<T>>,
      max_size: usize,
  }
  ```
  Avec `acquire(factory: impl FnOnce() -> T, reset: impl FnOnce(&mut T))` et `release(item: T)`.
- **Trait requis** : `T: Send` pour la sécurité thread
- **Vérification** : Tests unitaires pour le pool générique
- **Dépendances** : Aucune

### Tâche 2.2 — Créer le macro/helper thread-local pool

- **Fichier** : `crates/fibcalc-core/src/pool.rs` (extension de 2.1)
- **Action** : Fournir un pattern réutilisable pour les thread-local pools :
  ```rust
  pub fn tl_acquire<T>(pool: &RefCell<Vec<T>>, max: usize, factory: fn() -> T, reset: fn(&mut T)) -> T
  pub fn tl_release<T>(pool: &RefCell<Vec<T>>, max: usize, item: T)
  ```
- **Vérification** : Tests unitaires
- **Dépendances** : Tâche 2.1

### Tâche 2.3 — Migrer `CalculationStatePool` vers `ObjectPool`

- **Fichier** : `crates/fibcalc-core/src/fastdoubling.rs`
- **Action** :
  - Remplacer `CalculationStatePool` par `ObjectPool<CalculationState>`
  - Remplacer `tl_acquire_state` / `tl_release_state` par les helpers génériques
  - Garder le `thread_local!` car c'est critique pour la performance
- **Vérification** : `cargo test --workspace` — les tests existants du pool doivent passer
- **Dépendances** : Tâche 2.2

### Tâche 2.4 — Migrer `MatrixStatePool` vers `ObjectPool`

- **Fichier** : `crates/fibcalc-core/src/matrix.rs`
- **Action** : Même migration que 2.3 pour `MatrixStatePool` / `MatrixState`
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Tâche 2.2
- **Parallélisable avec** : Tâche 2.3

---

## Epic 3 — Déduplication des stratégies de multiplication

**Motivation** : `execute_doubling_step` est copié-collé identiquement dans 3 des 4 stratégies (Karatsuba, FFT, Adaptive). Seul `ParallelKaratsuba` diffère.

**Impact** : ~45 lignes de code dupliqué → implémentation par défaut dans le trait.

### Tâche 3.1 — Ajouter une implémentation par défaut à `DoublingStepExecutor`

- **Fichier** : `crates/fibcalc-core/src/strategy.rs`
- **Action** :
  - Ajouter une implémentation par défaut de `execute_doubling_step` dans le trait `DoublingStepExecutor` qui utilise `self.multiply()` et `self.square()` :
    ```rust
    fn execute_doubling_step(&self, fk: &BigUint, fk1: &BigUint) -> (BigUint, BigUint) {
        let t = (fk1 << 1u32) - fk;
        let f2k = self.multiply(fk, &t);
        let f2k1 = self.square(fk) + self.square(fk1);
        (f2k, f2k1)
    }
    ```
  - Retirer les implémentations identiques de `KaratsubaStrategy`, `FFTOnlyStrategy`, `AdaptiveStrategy`
  - Garder l'override dans `ParallelKaratsubaStrategy` (elle a la logique parallèle)
- **Vérification** :
  - `cargo test --workspace` — le test `all_strategies_agree_on_doubling` valide l'équivalence
  - Vérifier que le benchmark ne régresse pas
- **Dépendances** : Aucune

---

## Epic 4 — Performance Fermat/FFT

**Motivation** : Le crate `fibcalc-bigfft` utilise des conversions `BigUint` dans les chemins chauds de `FermatNum`, ce qui crée des allocations heap inutiles dans les boucles FFT.

**Impact** : Réduction des allocations dans les opérations FFT pour les très grands nombres (>500K bits).

### Tâche 4.1 — Optimiser `FermatNum::normalize()` sans BigUint

- **Fichier** : `crates/fibcalc-bigfft/src/fermat.rs:61-65`
- **Problème** : `normalize()` convertit en `BigUint`, fait un modulo, reconvertit. Pour `2^shift + 1`, la réduction est une opération simple sur les limbs.
- **Action** : Implémenter la réduction modulaire directement sur les limbs u64 :
  - Extraire les bits au-dessus de `shift`
  - Soustraire cette valeur des bits bas (car `2^shift ≡ -1 mod (2^shift+1)`)
  - Boucler si nécessaire
- **Vérification** :
  - Tests existants : `fermat_normalize`, `fermat_add_wraps`, `fermat_sub_wraps`
  - Ajouter un test de round-trip pour des valeurs > modulus
- **Dépendances** : Aucune

### Tâche 4.2 — Optimiser `FermatNum::shift_left()` sans BigUint

- **Fichier** : `crates/fibcalc-bigfft/src/fermat.rs:230-238`
- **Problème** : `shift_left` convertit en `BigUint`, shift, modulo, reconvertit. Le shift sur limbs u64 est une opération directe (déplacement de mots + shift intra-mot).
- **Action** : Implémenter le shift directement sur les limbs, puis appeler `normalize()` (version optimisée de 4.1)
- **Vérification** : Tests `fermat_shift_left`, `fermat_shift_left_wraps`
- **Dépendances** : Tâche 4.1

### Tâche 4.3 — Optimiser `FermatNum::shift_right()` sans BigUint

- **Fichier** : `crates/fibcalc-bigfft/src/fermat.rs:241-253`
- **Problème** : Même pattern que `shift_left`.
- **Action** : Réutiliser le shift optimisé de 4.2 (puisque `shift_right(k)` = `shift_left(2*shift - k)`)
- **Vérification** : Test `fermat_shift_right_inverse_of_left`
- **Dépendances** : Tâche 4.2

### Tâche 4.4 — Éliminer les clones hot-path dans le butterfly FFT

- **Fichier** : `crates/fibcalc-bigfft/src/fft_core.rs:29-34`
- **Problème** : La boucle butterfly interne de `fft_forward()` clone deux `FermatNum` par itération :
  ```rust
  let mut t = data[start + j + half].clone(); // clone Vec<u64>
  let u = data[start + j].clone();            // clone Vec<u64>
  ```
  Pour une FFT de taille `n=1024` avec shift=2048, cela produit ~2M+ allocations par forward transform. Chaque `FermatNum` contient un `Vec<u64>` (limbs).
- **Action** :
  - Utiliser `split_at_mut` pour obtenir deux slices mutables sans clone
  - Ou allouer un buffer de travail unique réutilisé à chaque itération
  - Ou utiliser `std::mem::swap` + arithmétique in-place pour éviter les allocations
  - Note : le crate interdit `unsafe_code`, donc `split_at_mut` est l'option correcte
- **Vérification** :
  - `cargo test -p fibcalc-bigfft` — tests de round-trip FFT existants
  - `cargo bench` — régression impossible, amélioration attendue
- **Dépendances** : Aucune
- **Impact** : Potentiel 10-20% d'amélioration sur les calculs FFT (nombres > 500K bits)
- **Risque** : Moyen — l'arithmétique butterfly est sensible aux indices

### Tâche 4.5 — Optimiser `FermatNum::fermat_mul()` (optionnel, complexité élevée)

- **Fichier** : `crates/fibcalc-bigfft/src/fermat.rs:219-227`
- **Problème** : Convertit en `BigUint` pour la multiplication. La multiplication modulaire sur Fermat est implémentable via NTT, mais c'est complexe.
- **Action** : Évaluer si ce chemin est réellement chaud (utilisé dans la boucle FFT ou seulement pour le setup). Si non chaud, documenter comme optimisation future.
- **Vérification** : Benchmark avant/après
- **Dépendances** : Tâches 4.1-4.3
- **Risque** : Élevé — multiplication modulaire efficace est non-triviale

### Tâche 4.6 — Renommer `clear_preserving_capacity()` dans `pool.rs`

- **Fichier** : `crates/fibcalc-bigfft/src/pool.rs:56`
- **Problème** : La fonction `clear_preserving_capacity(value: &mut BigUint)` fait `*value = BigUint::ZERO`, ce qui ne préserve PAS la capacité (num-bigint n'expose pas son `Vec` interne). Le commentaire l'admet déjà.
- **Action** : Renommer en `reset_value()` ou `clear_value()` pour refléter le comportement réel.
- **Vérification** : `cargo test -p fibcalc-bigfft`
- **Dépendances** : Aucune
- **Risque** : Nul — renommage interne uniquement

---

## Epic 5 — Refactorisation de l'orchestration app.rs

**Motivation** : `app.rs` contient une duplication test : `execute_cli_logic()` (~50 lignes) reproduit `run_cli()` sans le handler Ctrl+C. Les blocs de mémoire check sont aussi dupliqués entre `run_cli` et `run_tui`.

### Tâche 5.1 — Extraire la validation mémoire en helper

- **Fichier** : `crates/fibcalc/src/app.rs`
- **Problème** : Le bloc mémoire check (lignes 74-82 et 159-166) est identique dans `run_cli` et `run_tui`.
- **Action** : Extraire :
  ```rust
  fn check_memory_budget(n: u64, opts: &Options) -> Result<()> { ... }
  ```
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune

### Tâche 5.2 — Éliminer la duplication `execute_cli_logic` dans les tests

- **Fichier** : `crates/fibcalc/src/app.rs:304-355`
- **Problème** : `execute_cli_logic` duplique `run_cli` à 95%. La seule différence est le handler ctrlc.
- **Action** :
  - Rendre `run_cli` testable en acceptant un paramètre optionnel pour le handler ctrlc, ou
  - Extraire la logique core de `run_cli` dans une fonction interne qui ne touche pas ctrlc, puis appeler cette fonction depuis les tests
- **Vérification** : Les 20+ tests de `run_cli_*` doivent continuer à passer
- **Dépendances** : Tâche 5.1

### Tâche 5.3 — Extraire le setup calculateur en helper

- **Fichier** : `crates/fibcalc/src/app.rs`
- **Problème** : Le bloc factory + get_calculators + cancel + ctrlc est dupliqué entre `run_cli` et `run_tui`.
- **Action** : Extraire :
  ```rust
  fn setup_calculators(config: &AppConfig) -> Result<(Vec<Arc<dyn Calculator>>, CancellationToken)>
  ```
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Tâche 5.1

### Tâche 5.4 — Propager `FibError` au lieu de `String` dans l'orchestrateur

- **Fichier** : `crates/fibcalc-orchestration/src/orchestrator.rs:47`
- **Problème** : `result.map_err(|e| e.to_string())` convertit un `FibError` typé en `String`, perdant l'information de type. Le `CalculationResult::outcome` est `Result<BigUint, String>`.
- **Action** :
  - Changer `CalculationResult::outcome` de `Result<BigUint, String>` à `Result<BigUint, FibError>` dans `interfaces.rs`
  - Retirer le `.map_err(|e| e.to_string())` dans `orchestrator.rs`
  - Mettre à jour les consommateurs : `app.rs`, `presenter.rs` (CLI et TUI)
- **Vérification** : `cargo test --workspace` — les tests E2E couvrent les chemins d'erreur
- **Dépendances** : Aucune
- **Impact** : Permet un matching d'erreurs plus précis (ex: distinguer `Cancelled` de `Timeout`)

---

## Epic 6 — Nettoyage TUI et CLI

**Motivation** : Petites améliorations de qualité dans les crates UI.

### Tâche 6.1 — Réduire les allocations String dans le bridge TUI

- **Fichier** : `crates/fibcalc-tui/src/bridge.rs:69`
- **Problème** : `update.algorithm.to_string()` est appelé à chaque mise à jour de progression. `algorithm` est un `&'static str`, mais `TuiMessage::Progress` attend un `String`.
- **Action** :
  - Changer `TuiMessage::Progress { algorithm: String }` en `algorithm: &'static str` si possible
  - Ou utiliser `Cow<'static, str>` pour éviter l'allocation
- **Impact** : Réduit les allocations dans la boucle de progression hot
- **Vérification** : `cargo test --workspace` — les tests TUI couvrent bien ces chemins
- **Dépendances** : Aucune

### Tâche 6.2 — Éliminer la duplication render dans `model.rs`

- **Fichier** : `crates/fibcalc-tui/src/model.rs:411-444`
- **Problème** : Les blocs `show_logs == true` et `show_logs == false` dans `render()` dupliquent les appels `render_metrics` et `render_sparkline`.
- **Action** : Extraire le rendu metrics+sparkline en helper, puis brancher sur le layout
- **Vérification** : Tests render existants (`render_with_show_logs_true`, `render_with_show_logs_false`)
- **Dépendances** : Aucune

### Tâche 6.3 — Nettoyer `TUIProgressReporter` potentiellement inutilisé

- **Fichier** : `crates/fibcalc-tui/src/bridge.rs:16-39`
- **Problème** : `TUIProgressReporter` implémente `ProgressReporter` (trait d'orchestration), mais le code TUI dans `app.rs:run_tui` utilise `TuiBridgeObserver` (trait `ProgressObserver` de core). Vérifier si `TUIProgressReporter` est utilisé quelque part.
- **Action** :
  - Si non utilisé : retirer (+ ses tests)
  - Si utilisé : documenter la distinction entre les deux patterns
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune

### Tâche 6.4 — Nommer la constante magique `0.694` (log2(phi))

- **Fichier** : `crates/fibcalc-tui/src/model.rs:217`
- **Problème** : Le calcul de throughput utilise `self.n_value as f64 * 0.694` sans nommer la constante. `0.694` est `log2(phi) = log2((1+sqrt(5))/2)`, qui estime le nombre de bits de F(n).
- **Action** : Définir `const LOG2_PHI: f64 = std::f64::consts::LOG2_E * (1.0_f64 + 5.0_f64.sqrt()).ln() / 2.0;` ou la valeur littérale `0.694_271_662` dans `fibcalc-core/src/constants.rs` et l'importer.
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune

### Tâche 6.5 — Auditer `ColorTheme` et `SparklineBuffer` inutilisés

- **Fichiers** :
  - `crates/fibcalc-tui/src/styles.rs` — `ColorTheme` struct avec 8 attributs de couleur + méthodes
  - `crates/fibcalc-tui/src/sparkline.rs` — `SparklineBuffer` wrapper autour de `VecDeque`
- **Problème** :
  - `ColorTheme` est défini mais les fonctions de rendu utilisent des couleurs inline directement
  - `SparklineBuffer` est défini mais `TuiApp` utilise `VecDeque<f64>` directement pour `sparkline_data`
- **Action** :
  - Pour chaque : soit l'adopter partout, soit le retirer comme code mort
  - Préférence : retirer si inutilisé (principe de simplicité CLAUDE.md)
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune

---

## Epic 7 — Micro-optimisations et nettoyage fibcalc-core

**Motivation** : L'audit approfondi de `fibcalc-core` a révélé plusieurs problèmes mineurs mais cumulatifs : type atomique surdimensionné, champ inutilisé, logique dupliquée dans le gestionnaire de seuils, et itérateur O(n) évitable.

### Tâche 7.1 — `CancellationToken` : AtomicU64 → AtomicBool

- **Fichier** : `crates/fibcalc-core/src/progress.rs:104-106`
- **Problème** : `CancellationToken::cancelled` utilise `Arc<AtomicU64>` pour stocker un booléen (0 ou 1). `AtomicBool` est sémantiquement correct et plus compact.
- **Action** : Remplacer `AtomicU64` par `AtomicBool`, adapter `is_cancelled()` (`load` retourne `bool` directement) et `cancel()` (`store(true, ...)`).
- **Vérification** : `cargo test --workspace` — les tests existants (`cancellation_token_*`) valident le comportement
- **Dépendances** : Aucune
- **Risque** : Faible — changement interne, API publique inchangée

### Tâche 7.2 — Retirer `MatrixState::temp` inutilisé

- **Fichier** : `crates/fibcalc-core/src/matrix_types.rs:91`
- **Problème** : Le champ `temp: Matrix` est initialisé dans `new()` mais jamais lu ni écrit après construction. Il n'est même pas reset dans `reset()`. Code mort structurel.
- **Action** : Retirer le champ `temp` de `MatrixState` et de son constructeur.
- **Vérification** : `cargo test --workspace`
- **Dépendances** : Aucune

### Tâche 7.3 — Déduplication de `DynamicThresholdManager::adjust()`

- **Fichier** : `crates/fibcalc-core/src/dynamic_threshold.rs:67-138`
- **Problème** : La méthode `adjust()` contient 6 blocs quasi-identiques (positive/négative × FFT/parallel/strassen). Chaque bloc fait : check hysteresis → compute factor → apply → clamp floor → record.
- **Action** : Extraire une méthode helper :
  ```rust
  fn adjust_threshold(&mut self, name: &str, current: &mut usize, benefit: f64, floor: usize)
  ```
  Appeler 3 fois (FFT, parallel, strassen) au lieu de 6 blocs inline.
- **Vérification** : Les 10+ tests de `dynamic_threshold` couvrent tous les chemins.
- **Dépendances** : Aucune

### Tâche 7.4 — `FibIterator::from_index` : documenter la limitation O(n)

- **Fichier** : `crates/fibcalc-core/src/iterator.rs:34-42`
- **Problème** : `from_index(n)` boucle `n` fois en O(n) pour atteindre F(n). Pour n > 10K, c'est prohibitif. Le commentaire dit « A more efficient version could use fast doubling to jump to n ».
- **Action** :
  - Option A (minimal) : Ajouter un `#[doc]` warning sur la complexité O(n)
  - Option B (recommandé) : Utiliser `fibonacci(n)` et `fibonacci(n+1)` pour initialiser l'état en O(log n)
- **Vérification** : Test existant + nouveau test avec n=10000
- **Dépendances** : Aucune

---

## Epic 8 — Corrections calibration

**Motivation** : Le crate `fibcalc-calibration` contient un benchmarking FFT factice et un calcul de médiane biaisé, ce qui rend l'auto-calibration des seuils FFT non fiable.

### Tâche 8.1 — Intégrer le vrai benchmarking FFT dans `microbench.rs`

- **Fichier** : `crates/fibcalc-calibration/src/microbench.rs:23-29`
- **Problème** : `bench_fft()` est identique à `bench_karatsuba()` — les deux font `&a * &b` (multiplication `num-bigint` / Karatsuba). Le commentaire ligne 21 dit : _"Currently uses the same underlying num-bigint multiply; will route to actual FFT once fibcalc-bigfft is integrated."_ Résultat : `find_fft_crossover()` compare Karatsuba vs Karatsuba et ne trouve jamais de crossover réel.
- **Action** :
  - Importer `fibcalc_bigfft` dans `fibcalc-calibration` (ajouter la dépendance dans `Cargo.toml`)
  - Remplacer le corps de `bench_fft()` par un appel à la multiplication FFT réelle
  - Adapter `bench_fft_detailed()` de la même façon
  - Retirer le commentaire `will route to actual FFT`
- **Vérification** :
  - `cargo test -p fibcalc-calibration` — les tests existants passent
  - `find_fft_crossover(&[512, 1024, 2048, 4096, 8192])` devrait montrer un crossover réel à ~500K bits
- **Dépendances** : Aucune
- **Impact** : Rend l'auto-calibration des seuils FFT fonctionnelle

### Tâche 8.2 — Corriger le calcul de médiane dans `runner.rs`

- **Fichier** : `crates/fibcalc-calibration/src/runner.rs:41`
- **Problème** : `let median = durations[durations.len() / 2];` prend l'élément supérieur pour les tableaux de taille paire. Pour 10 éléments (indices 0-9), il prend l'indice 5, ignorant l'indice 4. La vraie médiane statistique est la moyenne des deux éléments centraux.
- **Action** : Corriger pour les tailles paires :
  ```rust
  let median = if durations.len() % 2 == 1 {
      durations[durations.len() / 2]
  } else {
      let mid = durations.len() / 2;
      (durations[mid - 1] + durations[mid]) / 2
  };
  ```
- **Vérification** :
  - Ajouter un test avec un nombre pair d'itérations
  - `cargo test -p fibcalc-calibration`
- **Dépendances** : Aucune
- **Risque** : Faible — le biais actuel est minime pour les benchmarks (< 1%)

---

## Epic 9 — Qualité et couverture de tests

**Motivation** : Consolidation de la qualité après les refactorisations.

### Tâche 9.1 — Retirer les `#[allow(...)]` inutiles

- **Action** : Rechercher tous les `#[allow(clippy::...)]` dans le code et vérifier s'ils sont encore nécessaires après les refactorisations.
- **Fichiers** : Tous les crates
- **Cibles particulières** :
  - `#[allow(clippy::too_many_lines)]` sur `handle_message` (model.rs:128) — la fonction fait ~108 lignes avec le match, ce qui est raisonnable pour un message handler
  - `#[allow(clippy::unused_self)]` sur `execute_doubling_loop` (fastdoubling.rs:169)
- **Vérification** : `cargo clippy -- -W clippy::pedantic`
- **Dépendances** : Epics 1-8 (exécuter en dernier)

### Tâche 9.2 — Ajouter des tests pour le pool générique

- **Fichier** : `crates/fibcalc-core/src/pool.rs`
- **Action** : Tests unitaires complets :
  - Acquire/release basique
  - Taille max respectée
  - Thread-local acquire/release
  - Multi-thread safety
- **Dépendances** : Epic 2

### Tâche 9.3 — Benchmark de non-régression

- **Action** : Exécuter `cargo bench` avant et après chaque epic pour s'assurer qu'aucune régression de performance n'est introduite.
- **Cibles** :
  - Fast Doubling (petits et grands n)
  - Matrix Exponentiation
  - FFT multiplication (si les optimisations Fermat sont appliquées)
- **Vérification** : Rapport Criterion avant/après
- **Dépendances** : Se fait en continu

### Tâche 9.4 — Vérification finale

- **Action** :
  - `cargo test --workspace` — 690+ tests verts
  - `cargo clippy -- -W clippy::pedantic` — 0 warnings
  - `cargo fmt --check` — formatage conforme
  - `cargo audit` — pas de vulnérabilités
  - `cargo deny check` — licences conformes
- **Dépendances** : Toutes les tâches précédentes

---

## Graphe de dépendances

```
Epic 1 (Code mort)              Epic 3 (Stratégies)
  ├─ 1.1 ──┐                      └─ 3.1
  ├─ 1.2   ├─ 1.5
  ├─ 1.3 ──┘
  ├─ 1.4 (errors/version)
  └─ 1.6 (CLIProgressReporter)

Epic 2 (Pool générique)          Epic 4 (Fermat/FFT)
  └─ 2.1                           ├─ 4.1
       └─ 2.2                      │    └─ 4.2
            ├─ 2.3                 │         └─ 4.3
            └─ 2.4                │              └─ 4.5 (optionnel)
                                   ├─ 4.4 (clones butterfly, indépendant)
                                   └─ 4.6 (rename, indépendant)

Epic 5 (App.rs + Orchestration)  Epic 6 (TUI/CLI)
  └─ 5.1                           ├─ 6.1
       ├─ 5.2                      ├─ 6.2
       └─ 5.3                      ├─ 6.3
  └─ 5.4 (FibError propagation)   ├─ 6.4
                                   └─ 6.5

Epic 7 (Micro-optim core)       Epic 8 (Calibration)
  ├─ 7.1 (AtomicBool)              ├─ 8.1 (FFT bench réel)
  ├─ 7.2 (MatrixState temp)        └─ 8.2 (médiane)
  ├─ 7.3 (adjust() dedup)
  └─ 7.4 (FibIterator)          Epic 9 (Qualité)
                                    └─ 9.1 (après Epics 1-8)
                                    └─ 9.2 (après Epic 2)
                                    └─ 9.3 (continu)
                                    └─ 9.4 (dernier)
```

**Aucune dépendance inter-epic** sauf :
- Epic 2 (pool) → si Epic 1 confirme que les frameworks sont morts, la migration pool est plus simple
- Epic 9.1 → après tous les autres epics
- Epics 7 et 8 sont entièrement parallélisables avec les Epics 2-6

---

## Ordre d'exécution recommandé

### Phase A — Nettoyage fondamental (parallélisable)

| # | Tâche | Epic | Peut être parallélisée avec |
|---|-------|------|-----------------------------|
| 1 | 1.1 Audit dead_code fibcalc-core | 1 | 1.2-1.4, 1.6, 3.1, 5.4, 6.x, 7.x, 8.x |
| 2 | 1.2 Audit dead_code fibcalc-bigfft | 1 | 1.1, 1.3-1.4, 1.6, 3.1, 5.4, 6.x, 7.x, 8.x |
| 3 | 1.3 Retirer _strassen_threshold | 1 | 1.1, 1.2, 1.4, 1.6, 3.1, 5.4, 6.x, 7.x, 8.x |
| 4 | 1.4 Dead code errors.rs/version.rs | 1 | 1.1-1.3, 1.6, 3.1, 5.4, 6.x, 7.x, 8.x |
| 5 | 1.6 Audit CLIProgressReporter | 1 | 1.1-1.4, 3.1, 5.4, 6.x, 7.x, 8.x |
| 6 | 3.1 Default impl DoublingStepExecutor | 3 | 1.x, 5.4, 6.x, 7.x, 8.x |
| 7 | 5.4 FibError propagation orchestrateur | 5 | 1.x, 3.1, 6.x, 7.x, 8.x |
| 8 | 6.1 String → &'static str bridge | 6 | 1.x, 3.1, 5.4, 6.2-6.5, 7.x, 8.x |
| 9 | 6.2 Déduplication render model.rs | 6 | 1.x, 3.1, 5.4, 6.1, 6.3-6.5, 7.x, 8.x |
| 10 | 6.3 Audit TUIProgressReporter | 6 | 1.x, 3.1, 5.4, 6.1, 6.2, 6.4, 6.5, 7.x, 8.x |
| 11 | 6.4 Constante LOG2_PHI | 6 | 1.x, 3.1, 5.4, 6.1-6.3, 6.5, 7.x, 8.x |
| 12 | 6.5 Audit ColorTheme/SparklineBuffer | 6 | 1.x, 3.1, 5.4, 6.1-6.4, 7.x, 8.x |
| 13 | 7.1 CancellationToken AtomicBool | 7 | 1.x, 3.1, 5.4, 6.x, 7.2-7.4, 8.x |
| 14 | 7.2 Retirer MatrixState::temp | 7 | 1.x, 3.1, 5.4, 6.x, 7.1, 7.3, 7.4, 8.x |
| 15 | 7.3 Déduplication adjust() | 7 | 1.x, 3.1, 5.4, 6.x, 7.1, 7.2, 7.4, 8.x |
| 16 | 7.4 FibIterator::from_index O(log n) | 7 | 1.x, 3.1, 5.4, 6.x, 7.1-7.3, 8.x |
| 17 | 4.4 Clones hot-path butterfly FFT | 4 | 1.x, 3.1, 5.4, 6.x, 7.x, 4.6, 8.x |
| 18 | 4.6 Rename clear_preserving_capacity | 4 | 1.x, 3.1, 5.4, 6.x, 7.x, 4.4, 8.x |
| 19 | 8.1 FFT bench réel microbench.rs | 8 | 1.x, 3.1, 5.4, 6.x, 7.x, 8.2 |
| 20 | 8.2 Médiane runner.rs | 8 | 1.x, 3.1, 5.4, 6.x, 7.x, 8.1 |

### Phase B — Refactorisations structurelles (séquentielles dans l'epic)

| # | Tâche | Epic | Dépend de |
|---|-------|------|-----------|
| 21 | 1.5 Résoudre frameworks vs inline | 1 | 1.1 |
| 22 | 2.1 Créer ObjectPool<T> | 2 | — |
| 23 | 2.2 Helper thread-local pool | 2 | 2.1 |
| 24 | 2.3 Migrer CalculationStatePool | 2 | 2.2 |
| 25 | 2.4 Migrer MatrixStatePool | 2 | 2.2 |
| 26 | 5.1 Extraire check_memory_budget | 5 | — |
| 27 | 5.2 Éliminer execute_cli_logic | 5 | 5.1 |
| 28 | 5.3 Extraire setup_calculateurs | 5 | 5.1 |

**Note** : Les tâches 22-25 (Epic 2) et 26-28 (Epic 5) sont parallélisables entre elles.

### Phase C — Optimisations performance (séquentielles)

| # | Tâche | Epic | Dépend de |
|---|-------|------|-----------|
| 29 | 4.1 Optimiser normalize() | 4 | — |
| 30 | 4.2 Optimiser shift_left() | 4 | 4.1 |
| 31 | 4.3 Optimiser shift_right() | 4 | 4.2 |
| 32 | 4.5 Évaluer fermat_mul() | 4 | 4.1-4.3 |

### Phase D — Vérification finale

| # | Tâche | Epic | Dépend de |
|---|-------|------|-----------|
| 33 | 9.1 Retirer #[allow] inutiles | 9 | Phases A-C |
| 34 | 9.2 Tests pool générique | 9 | Epic 2 |
| 35 | 9.3 Benchmark non-régression | 9 | Continu |
| 36 | 9.4 Vérification finale | 9 | Tout |

---

## Résumé des gains attendus

| Métrique | Avant | Après (estimé) |
|---|---|---|
| Code dupliqué (pool) | ~120 lignes | ~0 (+ 40 lignes module générique) |
| Code dupliqué (doubling step) | ~45 lignes | ~0 (default impl) |
| Code dupliqué (adjust() threshold) | ~72 lignes (6 blocs) | ~30 lignes (3 appels helper) |
| Code dupliqué (app.rs) | ~80 lignes | ~0 (helpers extraits) |
| `#![allow(dead_code)]` crate-level | 2 | 0 |
| Clones `FermatNum` dans butterfly FFT | 2 par itération | 0 (split_at_mut) |
| Allocations BigUint dans FFT hot path | 5 fonctions | 0-1 |
| Champs inutilisés | 2 (`_strassen_threshold`, `MatrixState::temp`) | 0 |
| Types surdimensionnés | 1 (`AtomicU64` pour bool) | 0 |
| Frameworks dupliquant le code inline | 2 (~194 lignes) | 0 |
| `FibIterator::from_index` complexité | O(n) | O(log n) |
| Fonctions mortes (`handle_error`, `version`, `CLIProgressReporter`) | 3 | 0 |
| Calibration FFT factice (bench_fft = bench_karatsuba) | 1 | 0 (vrai bench FFT) |
| Erreurs `FibError` → `String` (perte d'info type) | 1 | 0 (`FibError` propagé) |
| Médiane biaisée (floor-only pour tailles paires) | 1 | 0 (vraie médiane) |

**Total** : **9 Epics, 36 tâches**, ~400-500 lignes de code éliminées ou consolidées, calibration FFT fonctionnelle, performance FFT améliorée, 0 régression test/clippy.
