# PRD : Portage de FibGo vers Rust — Document de Spécification Exhaustif

**Version** : 1.0
**Date** : 2026-02-10
**Auteur** : Généré par Claude Code (équipe de 5 agents parallèles)
**Statut** : Complet — 78 tâches réalisées sur 7 phases

---

## Résumé Exécutif

Ce document constitue le Product Requirements Document (PRD) exhaustif pour le portage du projet **FibGo** — un calculateur Fibonacci haute performance écrit en Go — vers **Rust**. Le projet Go comporte 102 fichiers source, 85 fichiers test, 38 documents, et 17 packages.

Ce PRD couvre l'intégralité des aspects du portage :

1. **Fondations & Analyse des exigences** (12 tâches) — Vision, critères de succès, guide idiomatique Go→Rust, évaluation des dépendances, registre de risques, plan de validation croisée, baselines de performance, exigences non fonctionnelles, cross-compilation, licences, timeline, spécification CLAUDE.md
2. **Spécifications algorithmiques détaillées** (18 tâches) — Fast Doubling, Matrix Exponentiation, FFT (Fermat, récursion, polynômes), stratégie adaptative, arithmétique modulaire, preuves de correction
3. **Système Observer & suivi de progression** (10 tâches) — Pattern Observer, Freeze(), modèle géométrique, pré-calcul puissances de 4, observers concrets
4. **Gestion mémoire & concurrence** (12 tâches) — Arena, GC Controller, pools, bump allocator, budget mémoire, sémaphore, collecte d'erreurs, annulation coopérative
5. **Seuils dynamiques & calibration** (8 tâches) — DynamicThresholdManager, hystérésis, métriques, profils de calibration, estimation adaptative, micro-benchmarks
6. **Spécification détaillée du TUI** (10 tâches) — Architecture Elm→ratatui, messages, layout adaptatif, sparklines, Braille, logs scrollables, métriques système
7. **Intégration, tests & finalisation** (8 tâches) — Mapping fichier-par-fichier, contrats de traits, DFD, cas limites, propagation d'erreurs, FFI, tests d'intégration, documentation

---

## Table des Matières

- [Phase 1 : Fondations &amp; Analyse des exigences](#phase-1--fondations--analyse-des-exigences)
- [Phase 2 : Spécifications algorithmiques détaillées](#phase-2--spécifications-algorithmiques-détaillées)
- [Phase 3 : Système Observer &amp; suivi de progression](#phase-3--système-observer--suivi-de-progression)
- [Phase 4 : Gestion mémoire &amp; concurrence](#phase-4--gestion-mémoire--concurrence)
- [Phase 5 : Seuils dynamiques &amp; calibration](#phase-5--seuils-dynamiques--calibration)
- [Phase 6 : Spécification détaillée du TUI](#phase-6--spécification-détaillée-du-tui)
- [Phase 7 : Intégration, tests &amp; finalisation](#phase-7--intégration-tests--finalisation)
- [Résumé &amp; Dépendances critiques](#résumé--dépendances-critiques)

---

# Phase 1 : Fondations & Analyse des exigences

> **Projet** : Portage de FibGo (Go) vers Rust — Calculateur Fibonacci haute performance
> **Version** : 1.0
> **Date** : 2026-02-10

---

## T1.1 — Vision & Critères de succès

### 1.1.1 Vision du projet

FibCalc-rs est le portage complet du calculateur Fibonacci haute performance FibGo (Go 1.25+) vers Rust. L'objectif est de produire un binaire natif offrant des performances égales ou supérieures à la version Go, tout en exploitant les garanties de sécurité mémoire de Rust (absence de data races, pas de garbage collector, ownership explicite) et la richesse de son écosystème (Cargo, crates.io, `#[test]`, `criterion`).

Le projet cible une parité fonctionnelle complète : trois algorithmes (Fast Doubling, Matrix Exponentiation, FFT-Based), mode CLI et TUI interactif, calibration automatique, seuils dynamiques, et support optionnel GMP via la crate `rug`.

### 1.1.2 Objectifs et critères de succès

#### O1 — Parité fonctionnelle complète

| #    | Critère                                                                         | Méthode de vérification                              | Seuil                         |
| ---- | -------------------------------------------------------------------------------- | ------------------------------------------------------ | ----------------------------- |
| O1.1 | Les 3 algorithmes (fast, matrix, fft) produisent des résultats identiques à Go | Validation croisée sur golden files (27 valeurs de N) | 100% des golden tests passent |
| O1.2 | Toutes les options CLI sont supportées (22 flags documentés)                   | Tests E2E sur chaque flag avec sortie comparée        | 22/22 flags opérationnels    |
| O1.3 | Le mode TUI interactif fonctionne avec les 6 raccourcis clavier                  | Test manuel + tests unitaires du modèle Elm           | 6/6 raccourcis fonctionnels   |
| O1.4 | Le mode `--last-digits K` fonctionne pour N arbitrairement grands              | Test avec N=10^10, K=100, comparaison avec Go          | Résultats identiques         |
| O1.5 | Shell completion (bash, zsh, fish, powershell) opérationnelle                   | Génération + validation syntaxique par shell         | 4/4 shells supportés         |

#### O2 — Performance égale ou supérieure

| #    | Critère                                                    | Méthode de vérification                           | Seuil                             |
| ---- | ----------------------------------------------------------- | --------------------------------------------------- | --------------------------------- |
| O2.1 | Fast Doubling F(10M) ≤ temps Go de référence             | Benchmark `criterion` vs baseline Go              | ≤ 2.1s (ref AMD Ryzen 9)         |
| O2.2 | Empreinte mémoire (RSS) ≤ Go pour chaque N testé         | Mesure RSS via `/proc/self/status` ou `sysinfo` | RSS_Rust ≤ RSS_Go × 1.05        |
| O2.3 | Temps de démarrage (cold start) < 50ms                     | Mesure `time` sur 100 exécutions                 | p99 < 50ms                        |
| O2.4 | Aucune régression >5% sur les 18 benchmarks de référence | Suite `criterion` avec comparaison statistique    | Pas de régression >5% (p < 0.05) |

#### O3 — Qualité du code et sécurité mémoire

| #    | Critère                                            | Méthode de vérification                        | Seuil                               |
| ---- | --------------------------------------------------- | ------------------------------------------------ | ----------------------------------- |
| O3.1 | Zéro bloc `unsafe` hors FFI GMP et linkage SIMD  | Audit `cargo geiger`                           | ≤ 5 blocs unsafe, tous documentés |
| O3.2 | Couverture de tests ≥ 75%                          | `cargo tarpaulin`                              | ≥ 75% lignes couvertes             |
| O3.3 | Aucun warning clippy en mode `pedantic`           | `cargo clippy -- -W clippy::pedantic`          | 0 warnings                          |
| O3.4 | Aucune vulnérabilité connue dans les dépendances | `cargo audit`                                  | 0 vulnérabilités                  |
| O3.5 | Complexité cyclomatique < 15 par fonction          | Analyse statique (clippy / cognitive complexity) | Max 15 par fonction                 |

#### O4 — Portabilité multi-plateforme

| #    | Critère                                                           | Méthode de vérification | Seuil                                      |
| ---- | ------------------------------------------------------------------ | ------------------------- | ------------------------------------------ |
| O4.1 | Compilation réussie sur 5 target triples                          | CI matrix (voir T1.9)     | 5/5 targets compilent                      |
| O4.2 | Tests passent sur Linux x86_64, macOS arm64, Windows x86_64        | CI multi-plateforme       | 100% tests passent sur 3 OS                |
| O4.3 | Binaire statiquement lié (pas de dépendance dynamique, hors GMP) | `ldd` / `otool -L`    | 0 dépendances dynamiques (mode pure-Rust) |

#### O5 — Expérience développeur

| #    | Critère                                                      | Méthode de vérification             | Seuil                                   |
| ---- | ------------------------------------------------------------- | ------------------------------------- | --------------------------------------- |
| O5.1 | `cargo build --release` complète en < 120s                 | Mesure sur CI (machine standard)      | < 120s                                  |
| O5.2 | Documentation rustdoc complète pour tous les modules publics | `cargo doc --no-deps` sans warning  | 0 items publics non documentés         |
| O5.3 | CLAUDE.md Rust opérationnel et testé                        | Validation par agent IA (Claude Code) | Agent peut build + test sans aide       |
| O5.4 | Ajout d'un nouvel algorithme en < 30 minutes                  | Mesure de temps avec guide            | Trait `Calculator` + register < 30min |

#### O6 — Maintenabilité et écosystème

| #    | Critère                                                      | Méthode de vérification     | Seuil                    |
| ---- | ------------------------------------------------------------- | ----------------------------- | ------------------------ |
| O6.1 | Structure Cargo workspace avec ≤ 5 crates                    | Inspection `Cargo.toml`     | ≤ 5 crates              |
| O6.2 | Toutes les dépendances ont une licence compatible Apache-2.0 | `cargo deny check licenses` | 0 incompatibilités      |
| O6.3 | CI/CD opérationnelle (build, test, lint, audit, release)     | GitHub Actions workflow       | Pipeline complète verte |

---

## T1.2 — Analyse des lacunes

### 1.2.1 Tableau des lacunes identifiées

Le PRD existant (`PRD-Claude 1.md`, ~1280 lignes) présente 15 lacunes identifiées lors de l'audit comparatif avec le code source Go (102 fichiers, 17 packages).

| #   | Lacune                                                                                                    | Impact                                                                            | Priorité | Section PRD cible | Stratégie de résolution                                 |
| --- | --------------------------------------------------------------------------------------------------------- | --------------------------------------------------------------------------------- | --------- | ----------------- | --------------------------------------------------------- |
| L1  | Pas de mapping fichier-par-fichier Go → Rust                                                             | **Critique** — Un développeur ne sait pas où commencer                   | P0        | T7.1              | Tableau exhaustif 100+ fichiers avec crate destination    |
| L2  | Documentation algorithmique superficielle (pas de pseudocode d'itération bit-à-bit, swaps de pointeurs) | **Critique** — Portage impossible sans compréhension du flux exact        | P0        | T2.1–T2.4        | Pseudocode détaillé + diagrammes de machine à états   |
| L3  | Détails FFT manquants (arithmétique Fermat, récursion, sélection de paramètres, polynômes)          | **Critique** — Le moteur FFT est le composant le plus complexe (~1500 LoC) | P0        | T2.5–T2.9        | Spécification mathématique + pseudocode par fonction    |
| L4  | Système Observer non spécifié (Freeze() lock-free, modèle géométrique, pré-calcul puissances de 4) | **Élevé** — Affecte la progression et l'UX du TUI                        | P1        | T3.1–T3.10       | Diagramme UML + spécification Freeze + formules          |
| L5  | Seuils dynamiques non détaillés (ring buffer, hystérésis, algorithme d'ajustement)                    | **Élevé** — Affecte l'auto-tuning des performances                       | P1        | T5.1–T5.4        | Pseudocode complet + constantes + analyse de stabilité   |
| L6  | Pas de design alternatif Arena/GC Controller pour Rust (pas de GC en Rust)                                | **Élevé** — Le GC Controller Go n'a pas d'équivalent direct             | P1        | T4.1–T4.2        | Analyse RAII + mapping bumpalo + stratégie d'allocation  |
| L7  | Spécification TUI lacunaire (filtrage par génération, programRef, ring buffer sparklines)              | **Élevé** — TUI = composant le plus visible pour l'utilisateur           | P1        | T6.1–T6.10       | Catalogue des 11 types de messages + layout adaptatif     |
| L8  | Mapping idiomatique Go→Rust superficiel (au-delà des correspondances de surface)                        | **Élevé** — Risque de code "Go écrit en Rust"                           | P1        | T1.3              | Guide avec snippets côte-à-côte + notes de performance |
| L9  | Pas de critères d'acceptation par fonctionnalité                                                        | **Moyen** — Impossible de valider la complétion                           | P1        | T1.1              | Matrice O1-O6 avec critères testables                    |
| L10 | Pas d'évaluation comparative des dépendances                                                            | **Moyen** — Risque de mauvais choix (perf, licence, maturité)             | P1        | T1.4              | Matrice décisionnelle multi-critères                    |
| L11 | Pas de plan de validation croisée Go/Rust                                                                | **Moyen** — Impossible de garantir la correction                           | P1        | T1.6              | Protocole N × algo avec golden files + diff automatique  |
| L12 | Pas de registre de risques par composant                                                                  | **Moyen** — Surprises lors du portage                                      | P2        | T1.5              | Registre structuré avec mitigation                       |
| L13 | Pas de diagrammes de flux de données pour Rust                                                           | **Moyen** — Architecture Rust invisible                                    | P2        | T7.3              | 5 DFD avec frontières de crates et ownership             |
| L14 | Pas de spécification CLAUDE.md pour le projet Rust                                                       | **Faible** — Affecte l'outillage IA uniquement                             | P2        | T1.12             | Miroir structurel du CLAUDE.md Go                         |
| L15 | Pas de stratégie de détection de régression de performance                                             | **Moyen** — Régressions silencieuses après refactoring                   | P1        | T1.7              | Baselines criterion + seuils d'alerte statistiques        |

### 1.2.2 Priorisation

- **P0 (bloquant)** : L1, L2, L3 — Sans ces éléments, le portage ne peut pas démarrer
- **P1 (essentiel)** : L4–L11, L15 — Qualité et complétude du portage
- **P2 (souhaitable)** : L12–L14 — Amélioration progressive

### 1.2.3 Couverture par les phases du PRD

```
L1  → Phase 7 (T7.1 Migration Map)
L2  → Phase 2 (T2.1–T2.4 Fast Doubling + Matrix)
L3  → Phase 2 (T2.5–T2.9 FFT)
L4  → Phase 3 (T3.1–T3.10 Observer)
L5  → Phase 5 (T5.1–T5.4 Seuils dynamiques)
L6  → Phase 4 (T4.1–T4.2 Arena/GC)
L7  → Phase 6 (T6.1–T6.10 TUI)
L8  → Phase 1 (T1.3 Guide idiomes)
L9  → Phase 1 (T1.1 Critères de succès)
L10 → Phase 1 (T1.4 Matrice dépendances)
L11 → Phase 1 (T1.6 Validation croisée)
L12 → Phase 1 (T1.5 Registre risques)
L13 → Phase 7 (T7.3 DFD)
L14 → Phase 1 (T1.12 CLAUDE.md Rust)
L15 → Phase 1 (T1.7 Baselines performance)
```

---

## T1.3 — Guide approfondi des idiomes Rust

Ce guide traduit chaque pattern Go utilisé dans FibGo vers son équivalent Rust idiomatique, avec des extraits de code côte-à-côte et des notes de performance.

### 1.3.1 Interfaces → Traits

**Go** : Les interfaces sont satisfaites implicitement (duck typing structurel).

```go
// Go — internal/fibonacci/calculator.go
type Calculator interface {
    Calculate(ctx context.Context, progressChan chan<- ProgressUpdate,
        calcIndex int, n uint64, opts Options) (*big.Int, error)
    Name() string
}
```

**Rust** : Les traits sont implémentés explicitement. L'envoi dynamique (`dyn Trait`) remplace les interfaces Go.

```rust
// Rust — src/fibonacci/calculator.rs
use num_bigint::BigUint;
use tokio_util::sync::CancellationToken;

pub trait Calculator: Send + Sync {
    fn calculate(
        &self,
        cancel: &CancellationToken,
        progress: &dyn ProgressObserver,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError>;

    fn name(&self) -> &str;
}
```

**Notes** :

- `Send + Sync` requis pour le passage entre threads (Go le garantit implicitement)
- `&dyn ProgressObserver` remplace le channel Go pour le reporting de progression
- `Result<T, E>` remplace le pattern `(T, error)` de Go
- Le `context.Context` est remplacé par `CancellationToken` (plus léger que `tokio::Context`)

### 1.3.2 Goroutines → Rayon / Tokio

**Go** : Goroutines légères avec `go func()` et `errgroup` pour la synchronisation.

```go
// Go — internal/fibonacci/common.go
g, ctx := errgroup.WithContext(ctx)
g.Go(func() error {
    result1, err = strategy.Multiply(nil, x, y, opts)
    return err
})
g.Go(func() error {
    result2, err = strategy.Square(nil, z, opts)
    return err
})
err := g.Wait()
```

**Rust** : Pour le calcul parallèle CPU-bound, `rayon` est préféré à `tokio` (qui est conçu pour l'I/O async).

```rust
// Rust — src/fibonacci/common.rs
use rayon::join;

let (res1, res2) = rayon::join(
    || strategy.multiply(x, y, &opts),
    || strategy.square(z, &opts),
);
let result1 = res1?;
let result2 = res2?;
```

**Notes** :

- `rayon::join` est la correspondance directe du fork-join 2 voies
- Pour 3+ tâches parallèles : `rayon::scope` avec `s.spawn`
- Le pool de threads rayon est dimensionné automatiquement (nombre de cœurs)
- Pour les tâches I/O (TUI, réseau) : utiliser `tokio::spawn`

### 1.3.3 Channels → `crossbeam::channel` ou `std::sync::mpsc`

**Go** : Channels typés avec `make(chan T, capacity)`.

```go
// Go — internal/fibonacci/observers.go
type ChannelObserver struct {
    ch        chan<- ProgressUpdate
    calcIndex int
}

func (o *ChannelObserver) OnProgress(calcIndex int, progress float64) {
    select {
    case o.ch <- ProgressUpdate{CalcIndex: calcIndex, Progress: progress}:
    default: // Non-blocking send
    }
}
```

**Rust** : `crossbeam::channel` pour les channels multi-producteur performants.

```rust
// Rust — src/fibonacci/observers.rs
use crossbeam::channel::{Sender, TrySendError};

pub struct ChannelObserver {
    tx: Sender<ProgressUpdate>,
    calc_index: usize,
}

impl ProgressObserver for ChannelObserver {
    fn on_progress(&self, calc_index: usize, progress: f64) {
        // Envoi non-bloquant (équivalent du select/default Go)
        let _ = self.tx.try_send(ProgressUpdate {
            calc_index,
            progress,
        });
    }
}
```

**Notes** :

- `crossbeam::channel::bounded(cap)` ≡ `make(chan T, cap)`
- `crossbeam::channel::unbounded()` ≡ `make(chan T)` avec tampon illimité
- `try_send` ≡ `select { case ch <- v: default: }`
- Alternative : `tokio::sync::mpsc` si dans un contexte async

### 1.3.4 `sync.Pool` → Arena / Pool custom

**Go** : `sync.Pool` pour le recyclage d'objets avec GC.

```go
// Go — internal/fibonacci/common.go
var statePool = sync.Pool{
    New: func() interface{} {
        return &CalculationState{
            FK: new(big.Int), FK1: new(big.Int),
            T1: new(big.Int), T2: new(big.Int), T3: new(big.Int),
        }
    },
}
state := statePool.Get().(*CalculationState)
defer statePool.Put(state)
```

**Rust** : Pas de GC, donc pas besoin de pool pour éviter la pression GC. Deux approches :

```rust
// Approche 1 : Allocation stack/locale (préféré pour les petits états)
let mut state = CalculationState::new();
// Pas besoin de pool — ownership directe, RAII

// Approche 2 : Arena allocator pour les gros BigUint temporaires
use bumpalo::Bump;
let arena = Bump::with_capacity(estimated_size);
// Les allocations dans l'arena sont O(1) et libérées en bloc
```

**Notes** :

- En Rust, `sync.Pool` n'est pas nécessaire car il n'y a pas de GC
- Pour les `BigUint` temporaires volumineux : `bumpalo::Bump` pour l'allocation en bloc
- Pour le cas FFT avec réutilisation intensive : un pool `crossbeam::queue::ArrayQueue<T>` peut être utile
- Le pattern zero-copy (vol de pointeur) se traduit par `std::mem::take` ou `std::mem::replace`

### 1.3.5 `context.Context` → `CancellationToken`

**Go** : `context.Context` pour l'annulation coopérative et les deadlines.

```go
// Go — vérification dans les boucles algorithmiques
for bit := msb - 2; bit >= 0; bit-- {
    select {
    case <-ctx.Done():
        return nil, ctx.Err()
    default:
    }
    // ... calcul ...
}
```

**Rust** : `tokio_util::sync::CancellationToken` ou `Arc<AtomicBool>` pour le cas synchrone.

```rust
// Rust — vérification dans les boucles algorithmiques
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

pub struct CancelFlag(Arc<AtomicBool>);

impl CancelFlag {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Relaxed)
    }
}

for bit in (0..msb - 1).rev() {
    if cancel.is_cancelled() {
        return Err(FibError::Cancelled);
    }
    // ... calcul ...
}
```

**Notes** :

- `Ordering::Relaxed` suffit car l'annulation n'a pas besoin de garanties de visibilité immédiate
- Pour le timeout : `std::time::Instant::now().elapsed() > timeout`
- Pas d'équivalent exact du `context.WithTimeout` — combiner annulation + vérification temporelle

### 1.3.6 Error Handling : `(T, error)` → `Result<T, E>`

**Go** : Pattern `(T, error)` avec vérification manuelle.

```go
// Go — internal/errors/errors.go
type CalculationError struct {
    Algorithm string
    N         uint64
    Cause     error
}

func (e *CalculationError) Error() string {
    return fmt.Sprintf("calculation error for %s at N=%d: %v",
        e.Algorithm, e.N, e.Cause)
}
```

**Rust** : Enum d'erreur avec `thiserror` pour la dérivation automatique.

```rust
// Rust — src/errors.rs
use thiserror::Error;

#[derive(Debug, Error)]
pub enum FibError {
    #[error("calculation error for {algorithm} at N={n}: {source}")]
    Calculation {
        algorithm: String,
        n: u64,
        #[source]
        source: Box<dyn std::error::Error + Send + Sync>,
    },

    #[error("configuration error: {0}")]
    Config(String),

    #[error("operation cancelled")]
    Cancelled,

    #[error("operation timed out after {0:?}")]
    Timeout(std::time::Duration),

    #[error("result mismatch between algorithms")]
    Mismatch,
}

// Codes de sortie
impl FibError {
    pub fn exit_code(&self) -> i32 {
        match self {
            FibError::Calculation { .. } => 1,
            FibError::Timeout(_) => 2,
            FibError::Mismatch => 3,
            FibError::Config(_) => 4,
            FibError::Cancelled => 130,
        }
    }
}
```

**Notes** :

- `?` opérateur remplace `if err != nil { return err }`
- `thiserror` génère les implémentations `Display` et `Error`
- `anyhow` pour les erreurs dans le code applicatif (main, CLI)
- `thiserror` pour les erreurs de bibliothèque (algorithmes, orchestration)

### 1.3.7 Generics : Contrainte pointeur → Bounds de traits

**Go** : Generics avec contrainte pointeur pour les tâches parallèles.

```go
// Go — internal/fibonacci/common.go
type taskResult[T any] struct {
    result T
    err    error
}

func executeTasks[T any, PT interface {
    *T
    Execute(ctx context.Context) error
}](ctx context.Context, tasks []PT) error {
    // ...
}
```

**Rust** : Generics avec bounds de traits.

```rust
// Rust — src/fibonacci/common.rs
pub trait ParallelTask: Send {
    type Output: Send;
    fn execute(&mut self, cancel: &CancelFlag) -> Result<Self::Output, FibError>;
}

pub fn execute_tasks<T: ParallelTask>(
    cancel: &CancelFlag,
    tasks: &mut [T],
) -> Result<Vec<T::Output>, FibError> {
    rayon::scope(|s| {
        // ...
    })
}
```

### 1.3.8 `sync.RWMutex` → `parking_lot::RwLock`

**Go** : Verrous lecteur-écrivain pour le registre de calculateurs.

```go
// Go — internal/fibonacci/registry.go
type DefaultFactory struct {
    mu          sync.RWMutex
    creators    map[string]func() coreCalculator
    calculators map[string]Calculator
}
```

**Rust** : `parking_lot::RwLock` (plus performant que `std::sync::RwLock`).

```rust
// Rust — src/fibonacci/registry.rs
use parking_lot::RwLock;
use std::collections::HashMap;

pub struct DefaultFactory {
    creators: RwLock<HashMap<String, Box<dyn Fn() -> Box<dyn CoreCalculator>>>>,
    calculators: RwLock<HashMap<String, Arc<dyn Calculator>>>,
}
```

### 1.3.9 `big.Int` → `num_bigint::BigUint`

**Go** : `math/big.Int` avec opérations in-place.

```go
// Go
a.Mul(a, b)      // a = a * b (in-place)
a.Add(a, b)      // a = a + b (in-place)
bits := a.BitLen()
```

**Rust** : `num_bigint::BigUint` avec ownership.

```rust
// Rust
use num_bigint::BigUint;

let a = &a * &b;          // Crée un nouveau BigUint
a *= &b;                   // In-place via MulAssign (si mutable)
let bits = a.bits() as usize;
```

**Notes de performance** :

- `num-bigint` utilise Karatsuba internement pour les grands nombres
- Pour les performances FFT, il faudra implémenter notre propre multiplication FFT ou utiliser `rug`
- `rug` (bindings GMP) offre les meilleures performances brutes mais nécessite libgmp (LGPL)
- L'accès aux "limbs" internes de `BigUint` se fait via `to_u64_digits()` / `from_slice()`

### 1.3.10 Bump Allocator FFT → `bumpalo`

**Go** : Allocateur bump custom dans `internal/bigfft/bump.go`.

```go
// Go
type BumpAllocator struct {
    buf    []big.Word
    offset int
}
func (b *BumpAllocator) Alloc(n int) []big.Word {
    // O(1) bump allocation
}
```

**Rust** : `bumpalo::Bump` avec l'API identique.

```rust
// Rust — src/bigfft/bump.rs
use bumpalo::Bump;

let arena = Bump::with_capacity(estimated_bytes);
let slice: &mut [u64] = arena.alloc_slice_fill_default(n);
// O(1) allocation, libéré en bloc à la destruction de `arena`
```

### 1.3.11 `init()` auto-registration → Inventory / linkme

**Go** : Les calculateurs GMP s'auto-enregistrent via `init()`.

```go
// Go — internal/fibonacci/calculator_gmp.go
func init() {
    RegisterCalculator("gmp", func() coreCalculator { return &GMPCalculator{} })
}
```

**Rust** : Utiliser `inventory` ou `linkme` pour l'enregistrement statique.

```rust
// Rust — src/fibonacci/calculator_gmp.rs
#[cfg(feature = "gmp")]
inventory::submit! {
    CalculatorRegistration::new("gmp", || Box::new(GmpCalculator))
}
```

Alternative plus simple : enregistrement explicite dans la factory avec `#[cfg(feature = "gmp")]`.

### 1.3.12 Build Tags → Cargo Features

**Go** : Build tags dans les commentaires de fichier.

```go
//go:build gmp
```

**Rust** : Features Cargo dans `Cargo.toml`.

```toml
# Cargo.toml
[features]
default = []
gmp = ["dep:rug"]

[dependencies]
rug = { version = "1", optional = true }
```

```rust
// Conditionnel en Rust
#[cfg(feature = "gmp")]
mod calculator_gmp;
```

### 1.3.13 `go:linkname` SIMD → `std::arch` / assembleur inline

**Go** : Accès aux routines assembly de `math/big` via `go:linkname`.

```go
// Go — internal/bigfft/arith_decl.go
//go:linkname addVV math/big.addVV
func addVV(z, x, y []big.Word) big.Word
```

**Rust** : Intrinsèques SIMD natifs via `std::arch`.

```rust
// Rust — src/bigfft/arith_amd64.rs
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "x86_64")]
pub unsafe fn add_vv_avx2(z: &mut [u64], x: &[u64], y: &[u64]) -> u64 {
    // AVX2 vectorized addition
    // ...
}

// Fallback portable
pub fn add_vv(z: &mut [u64], x: &[u64], y: &[u64]) -> u64 {
    // Implémentation scalaire
    let mut carry: u64 = 0;
    for i in 0..z.len() {
        let (sum, c1) = x[i].overflowing_add(y[i]);
        let (sum, c2) = sum.overflowing_add(carry);
        z[i] = sum;
        carry = (c1 as u64) + (c2 as u64);
    }
    carry
}
```

### 1.3.14 Tableau récapitulatif des correspondances

| Pattern Go                   | Idiome Rust                                 | Crate(s)                 | Notes perf                              |
| ---------------------------- | ------------------------------------------- | ------------------------ | --------------------------------------- |
| `interface{}`              | `dyn Trait` / `impl Trait`              | —                       | Dispatch dynamique vs statique          |
| `goroutine` + `errgroup` | `rayon::join` / `rayon::scope`          | `rayon`                | Pool de threads work-stealing           |
| `chan T`                   | `crossbeam::channel`                      | `crossbeam`            | Plus performant que `std::sync::mpsc` |
| `sync.Pool`                | Stack allocation /`bumpalo`               | `bumpalo`              | Pas de GC → pas besoin de pool         |
| `context.Context`          | `CancellationToken` / `Arc<AtomicBool>` | `tokio-util`           | Relaxed ordering suffit                 |
| `(T, error)`               | `Result<T, E>`                            | `thiserror`            | Propagation avec `?`                  |
| `sync.RWMutex`             | `parking_lot::RwLock`                     | `parking_lot`          | ~30% plus rapide que std                |
| `big.Int`                  | `BigUint` / `rug::Integer`              | `num-bigint` / `rug` | rug = GMP (LGPL)                        |
| `go:build tag`             | `#[cfg(feature = "x")]`                   | Cargo features           | Compilation conditionnelle              |
| `go:linkname` asm          | `std::arch::x86_64`                       | —                       | Intrinsèques natifs                    |
| `init()` auto-register     | `inventory::submit!`                      | `inventory`            | Enregistrement statique                 |
| `sync.Once`                | `std::sync::OnceLock`                     | —                       | Initialisation paresseuse               |
| `defer`                    | `Drop` trait / `scopeguard`             | `scopeguard`           | RAII automatique                        |
| `select {}` multi-channel  | `crossbeam::select!`                      | `crossbeam`            | Macro de sélection                     |
| `go fmt`                   | `rustfmt`                                 | —                       | Formatage automatique                   |
| `golangci-lint`            | `cargo clippy`                            | —                       | Linting statique                        |

---

## T1.4 — Matrice d'évaluation des dépendances

### 1.4.1 Arithmétique grands nombres

| Critère                        | `num-bigint`                | `rug` (GMP)                       | `ibig`             |
| ------------------------------- | ----------------------------- | ----------------------------------- | -------------------- |
| **Performance Karatsuba** | Bonne (natif Rust)            | Excellente (asm optimisé C/asm)    | Bonne                |
| **Performance FFT mult.** | Absente (Karatsuba max)       | Excellente (GMP FFT natif)          | Partielle            |
| **Sécurité mémoire**   | Pure Rust, 100% safe          | FFI C, blocs `unsafe` requis      | Pure Rust            |
| **Ergonomie API**         | Bonne, traits std             | Très bonne, opérateurs natifs     | Bonne                |
| **Maturité**             | Très mature (>10 ans)        | Très mature (GMP = 1991)           | Jeune (2021)         |
| **Licence**               | MIT/Apache-2.0                | LGPL-3.0 (via GMP)                  | MIT/Apache-2.0       |
| **Cross-compilation**     | Triviale                      | Difficile (nécessite libgmp)       | Triviale             |
| **Taille binaire**        | Faible                        | Élevée (+libgmp)                  | Faible               |
| **Recommandation**        | **Défaut (pure-Rust)** | **Feature optionnelle "gmp"** | Alternative possible |

**Décision** : `num-bigint` par défaut + notre propre FFT multiplication (portage du module `bigfft`). Feature `gmp` optionnelle via `rug` pour les performances maximales.

### 1.4.2 TUI / Interface terminale

| Critère                            | `ratatui`                                 | `cursive`                  | `tui-realm`   |
| ----------------------------------- | ------------------------------------------- | ---------------------------- | --------------- |
| **Architecture**              | Immediate mode (Elm-like)                   | Widget-based                 | Component-based |
| **Compatibilité Bubble Tea** | **Excellente** — même paradigme Elm | Faible — modèle différent | Moyenne         |
| **Sparklines / Graphiques**   | Oui (widgets natifs)                        | Non (plugin nécessaire)     | Oui             |
| **Écosystème**              | Très actif (>8K stars)                     | Mature (>3K stars)           | Petit           |
| **Backend**                   | crossterm (cross-platform)                  | pancurses/crossterm          | crossterm       |
| **Licence**                   | MIT                                         | MIT                          | MIT             |
| **Recommandation**            | **Retenu**                            | Rejeté                      | Rejeté         |

**Décision** : `ratatui` avec backend `crossterm`. La correspondance architecturale avec Bubble Tea (Init/Update/View) facilitera le portage du TUI.

### 1.4.3 CLI / Parsing d'arguments

| Critère                    | `clap` (derive)                | `argh`            | `structopt`         |
| --------------------------- | -------------------------------- | ------------------- | --------------------- |
| **Ergonomie**         | Excellente (dérivation macro)   | Bonne               | Intégré à clap     |
| **Completion shells** | Oui (clap_complete)              | Non                 | Via clap              |
| **Validation**        | Riche (types, ranges, conflicts) | Basique             | Via clap              |
| **Maturité**         | Standard de facto                | Google, minimaliste | Déprécié (→ clap) |
| **Taille binaire**    | ~200KB                           | ~50KB               | N/A                   |
| **Licence**           | MIT/Apache-2.0                   | BSD-3               | MIT/Apache-2.0        |
| **Recommandation**    | **Retenu**                 | Rejeté             | Déprécié           |

**Décision** : `clap` avec dérivation + `clap_complete` pour les shell completions.

### 1.4.4 Concurrence et parallélisme

| Critère                      | `rayon`                    | `crossbeam`                         | `tokio`                           |
| ----------------------------- | ---------------------------- | ------------------------------------- | ----------------------------------- |
| **Modèle**             | Data-parallel, work-stealing | Primitives bas niveau                 | Async I/O runtime                   |
| **Cas d'usage FibCalc** | Multiplication parallèle    | Channels, scoped threads              | Event loop TUI (optionnel)          |
| **Overhead**            | Faible (pool de threads)     | Minimal                               | Élevé (runtime async)             |
| **Recommandation**      | **Calculs CPU-bound**  | **Communication inter-threads** | **Non retenu pour le calcul** |

**Décision** : `rayon` pour le parallélisme computationnel + `crossbeam` pour les channels. Tokio n'est pas nécessaire pour un calculateur CPU-bound synchrone.

### 1.4.5 Allocation mémoire

| Critère                    | `bumpalo`                 | `typed-arena`              | `std alloc`     |
| --------------------------- | --------------------------- | ---------------------------- | ----------------- |
| **Modèle**           | Bump pointer, reset en bloc | Typed arena, drop individuel | General purpose   |
| **Correspondance Go** | `BumpAllocator` (bigfft)  | `CalculationArena`         | `sync.Pool`     |
| **Performance**       | O(1) alloc, excellent cache | O(1) alloc                   | O(log n) alloc    |
| **Recommandation**    | **FFT temporaires**   | **États de calcul**   | **Défaut** |

### 1.4.6 Logging

| Critère                         | `tracing`                    | `log` + `env_logger` | `slog` |
| -------------------------------- | ------------------------------ | ------------------------ | -------- |
| **Structuré**             | Oui (spans + events)           | Basique                  | Oui      |
| **Correspondance zerolog** | Excellente                     | Moyenne                  | Bonne    |
| **Performance**            | Excellente (quand désactivé) | Bonne                    | Bonne    |
| **Recommandation**         | **Retenu**               | Rejeté                  | Rejeté  |

### 1.4.7 Tests et benchmarks

| Crate                           | Rôle                   | Correspondance Go  |
| ------------------------------- | ----------------------- | ------------------ |
| `criterion`                   | Benchmarks statistiques | `go test -bench` |
| `proptest`                    | Property-based testing  | `gopter`         |
| `cargo-fuzz` / `libfuzzer`  | Fuzz testing            | `go test -fuzz`  |
| `insta`                       | Snapshot/golden testing | Golden file tests  |
| `assert_cmd` + `predicates` | Tests E2E CLI           | `test/e2e`       |

### 1.4.8 Autres dépendances

| Crate                      | Rôle                                     | Correspondance Go                 |
| -------------------------- | ----------------------------------------- | --------------------------------- |
| `thiserror`              | Dérivation d'erreurs                     | `internal/errors`               |
| `anyhow`                 | Erreurs contextuelles (binaire)           | `fmt.Errorf` wrapping           |
| `serde` + `serde_json` | Sérialisation JSON (profils calibration) | `encoding/json`                 |
| `parking_lot`            | Mutexes performants                       | `sync.Mutex` / `sync.RWMutex` |
| `sysinfo`                | Métriques système (CPU, RAM)            | `gopsutil/v4`                   |
| `indicatif`              | Spinners et barres de progression CLI     | `briandowns/spinner`            |
| `console`                | Détection NO_COLOR, styles               | `fatih/color`                   |
| `num-traits`             | Traits numériques                        | implicite dans `math/big`       |

---

## T1.5 — Registre de risques

### 1.5.1 Risques par module

| Module                                | Risque                                                                                       | Prob.    | Impact             | Mitigation                                                                                                         |
| ------------------------------------- | -------------------------------------------------------------------------------------------- | -------- | ------------------ | ------------------------------------------------------------------------------------------------------------------ |
| **bigfft (FFT)**                | R1 : La multiplication FFT sur nombres de Fermat n'a pas d'équivalent Rust existant         | Élevée | **Critique** | Portage manuel du module bigfft Go (~1500 LoC). Prioriser ce module en Phase 2.                                    |
| **bigfft (FFT)**                | R2 : Performance de la FFT Rust inférieure à Go (go:linkname vers asm math/big)            | Moyenne  | **Élevé**  | Utiliser `std::arch` intrinsèques SIMD + benchmarks comparatifs dès le portage                                 |
| **bigfft (FFT)**                | R3 : Le cache LRU FFT thread-safe est complexe à porter avec les bons lifetimes             | Moyenne  | **Moyen**    | Utiliser `dashmap` ou `parking_lot::RwLock<LruCache>` pour simplifier                                          |
| **fibonacci (algorithmes)**     | R4 : Les opérations in-place sur BigUint sont moins idiomatiques qu'en Go                   | Moyenne  | **Moyen**    | Utiliser `MulAssign`, `AddAssign` + `std::mem::replace` pour le zero-copy                                    |
| **fibonacci (algorithmes)**     | R5 : Le seuil de basculement FFT peut différer entre Go et Rust                             | Élevée | **Moyen**    | Calibration Rust indépendante dès Phase 5, ne pas copier les seuils Go                                           |
| **fibonacci (Observer)**        | R6 : Le mécanisme Freeze() lock-free est difficile à implémenter sans GC                  | Moyenne  | **Moyen**    | Utiliser `Arc<Vec<Box<dyn Observer>>>` avec snapshot atomique via `ArcSwap`                                    |
| **fibonacci (dynamic thresh.)** | R7 : Le ring buffer avec hystérésis est sensible au timing Rust (différent de Go)         | Faible   | **Faible**   | Les constantes d'hystérésis (15%) sont robustes, ajuster empiriquement                                           |
| **tui**                         | R8 : Le mapping Bubble Tea → ratatui nécessite un redesign du cycle de messages            | Élevée | **Élevé**  | Prototyper le TUI tôt (Phase 6) avec un spike architectural                                                       |
| **tui**                         | R9 : Les sparklines Braille nécessitent un widget custom dans ratatui                       | Faible   | **Faible**   | ratatui a un widget Sparkline natif, adapter si nécessaire                                                        |
| **orchestration**               | R10 : errgroup → rayon::scope a une sémantique légèrement différente (panic vs error)   | Moyenne  | **Moyen**    | Utiliser `std::panic::catch_unwind` + conversion en Result                                                       |
| **config**                      | R11 : Le parsing de flags Go (`flag` package) ne mappe pas 1:1 sur clap                    | Faible   | **Faible**   | clap est plus puissant, adaptation straightforward                                                                 |
| **calibration**                 | R12 : Les micro-benchmarks Rust ont un overhead différent de Go                             | Moyenne  | **Moyen**    | Utiliser `criterion` en mode automatique, ne pas comparer les valeurs absolues cross-langage                     |
| **bigfft (SIMD)**               | R13 : Les routines asm amd64 via go:linkname n'ont pas d'équivalent direct                  | Moyenne  | **Élevé**  | Implémenter via `std::arch::x86_64` intrinsèques ou laisser LLVM auto-vectoriser                               |
| **gmp (feature)**               | R14 : La crate `rug` (GMP bindings) est LGPL, ce qui peut être incompatible               | Faible   | **Élevé**  | Documenter la contrainte LGPL. Feature optionnelle uniquement. Alternative :`gmp-mpfr-sys` avec linking statique |
| **global**                      | R15 : La taille du portage (~15K LoC Go → ~20K LoC Rust estimé) crée un risque de dérive | Moyenne  | **Élevé**  | Migration incrémentale phase par phase avec validation croisée à chaque étape                                  |

### 1.5.2 Matrice Risque × Impact

```
Impact ↑
Critique │  R1
Élevé    │  R2  R8  R13  R14  R15
Moyen    │  R3  R4  R5  R6  R10  R12
Faible   │  R7  R9  R11
         └────────────────────────→ Probabilité
           Faible   Moyenne   Élevée
```

### 1.5.3 Risques prioritaires (Top 5)

1. **R1** (bigfft FFT) — Mitigation : démarrer le portage FFT en Sprint 1 de la Phase 2
2. **R8** (TUI redesign) — Mitigation : spike architectural ratatui avant le portage complet
3. **R15** (taille du portage) — Mitigation : plan incrémental avec jalons de validation
4. **R2** (perf FFT) — Mitigation : benchmarks comparatifs dès les premiers fichiers portés
5. **R14** (licence GMP) — Mitigation : feature optionnelle, pure-Rust par défaut

---

## T1.6 — Plan de validation croisée Go/Rust

### 1.6.1 Principe

Chaque algorithme Rust doit produire des résultats **bit-à-bit identiques** à la version Go pour un ensemble exhaustif de valeurs de N. La validation s'appuie sur trois mécanismes complémentaires :

1. **Golden files** — Valeurs pré-calculées stockées en JSON
2. **Exécution parallèle** — Exécution Go + Rust avec comparaison automatique
3. **Identités mathématiques** — Vérification indépendante des propriétés algébriques

### 1.6.2 Matrice de validation

| N           | Fast Doubling                  | Matrix Exp.    | FFT-Based      | Résultat (digits) | Type de validation            |
| ----------- | ------------------------------ | -------------- | -------------- | ------------------ | ----------------------------- |
| 0           | F(0) = 0                       | F(0) = 0       | F(0) = 0       | 1                  | Golden file                   |
| 1           | F(1) = 1                       | F(1) = 1       | F(1) = 1       | 1                  | Golden file                   |
| 92          | Near max u64                   | Near max u64   | Near max u64   | 19                 | Golden file (frontière u64)  |
| 93          | Max u64 (12200160415121876738) | Idem           | Idem           | 20                 | Golden file (seuil fast path) |
| 94          | Premier BigInt                 | Premier BigInt | Premier BigInt | 20                 | Golden file (overflow u64)    |
| 1 000       | Golden                         | Golden         | Golden         | 209                | Golden file                   |
| 10 000      | Golden                         | Golden         | Golden         | 2 090              | Golden file + identités      |
| 100 000     | Golden                         | Golden         | Golden         | 20 899             | Golden file + identités      |
| 1 000 000   | Exécution Go                  | Exécution Go  | Exécution Go  | 208 988            | Diff Go/Rust + Cassini        |
| 10 000 000  | Exécution Go                  | Exécution Go  | Exécution Go  | 2 089 877          | Diff Go/Rust + Cassini        |
| 100 000 000 | Exécution Go                  | Exécution Go  | Exécution Go  | 20 898 764         | Diff Go/Rust (dernier)        |

**Total** : 11 valeurs de N × 3 algorithmes = **33 points de validation**

### 1.6.3 Protocole de validation

#### Étape 1 : Golden files (automatique, CI)

```bash
# Générer les golden files depuis Go
cd fibgo && go run ./cmd/generate-golden/ > golden.json

# Copier dans le projet Rust
cp golden.json fibcalc-rs/tests/testdata/fibonacci_golden.json

# Exécuter les tests Rust contre les golden files
cd fibcalc-rs && cargo test golden
```

Le fichier `fibonacci_golden.json` contient des paires `{ "n": uint64, "result": "decimal_string" }`.

#### Étape 2 : Diff Go/Rust pour les grandes valeurs

```bash
#!/bin/bash
# validate_cross.sh
for N in 1000000 10000000 100000000; do
    for ALGO in fast matrix fft; do
        GO_RESULT=$(cd fibgo && go run ./cmd/fibcalc -n $N -algo $ALGO -c -q)
        RS_RESULT=$(cd fibcalc-rs && cargo run --release -- -n $N -algo $ALGO -c -q)
        if [ "$GO_RESULT" != "$RS_RESULT" ]; then
            echo "MISMATCH: N=$N algo=$ALGO"
            exit 1
        fi
    done
done
echo "All cross-validation passed"
```

#### Étape 3 : Identités mathématiques (test indépendant)

Tests property-based avec `proptest` vérifiant :

1. **Identité de Cassini** : `F(n-1) × F(n+1) - F(n)² = (-1)^n`
2. **Identité de doublement** : `F(2n) = F(n) × (2×F(n+1) - F(n))`
3. **Identité de d'Ocagne** : `|F(m)×F(n+1) - F(m+1)×F(n)| = F(n-m)` pour n > m

```rust
// Rust — tests/property_tests.rs
use proptest::prelude::*;

proptest! {
    #[test]
    fn cassini_identity(n in 2u64..25000) {
        let f_n_minus_1 = fib(n - 1);
        let f_n = fib(n);
        let f_n_plus_1 = fib(n + 1);
        let lhs = &f_n_minus_1 * &f_n_plus_1 - &f_n * &f_n;
        let expected = if n % 2 == 0 { -1i64 } else { 1i64 };
        prop_assert_eq!(lhs, BigInt::from(expected));
    }
}
```

#### Étape 4 : Fuzz testing croisé

5 cibles de fuzz correspondant aux 5 cibles Go :

| Cible Fuzz                         | Stratégie                                   | Limite N                  |
| ---------------------------------- | -------------------------------------------- | ------------------------- |
| `fuzz_fast_doubling_consistency` | Cross-valide Fast Doubling vs Matrix         | n ≤ 50 000               |
| `fuzz_fft_consistency`           | Cross-valide FFT vs Fast Doubling            | n ≤ 20 000               |
| `fuzz_fibonacci_identities`      | Vérifie identités de doublement + d'Ocagne | n ≤ 10 000               |
| `fuzz_progress_monotonicity`     | Progression monotone croissante              | n 10..20 000              |
| `fuzz_fast_doubling_mod`         | Validation modular Fast Doubling             | n ≤ 100 000, mod ≤ 10^9 |

### 1.6.4 Critères de réussite

- 100% des golden tests passent (33/33)
- Diff Go/Rust identique pour N ∈ {1M, 10M, 100M} × 3 algos
- Propriétés de Cassini, doublement et d'Ocagne vérifiées sur 100 000 valeurs aléatoires
- Fuzz testing 30 minutes sans crash ni divergence

---

## T1.7 — Baselines de performance et détection de régression

### 1.7.1 Configuration de référence

Deux machines de référence documentées dans le projet Go :

**Machine A (README)** : Intel Core Ultra 9 275HX, 24 cores
**Machine B (PERFORMANCE.md)** : AMD Ryzen 9 5900X, 12 cores, 32 GB DDR4-3600, Linux 6.1, Go 1.25.0

### 1.7.2 Baselines Go (Machine B — AMD Ryzen 9 5900X)

| N           | Fast Doubling | Matrix Exp. | FFT-Based | Digits     |
| ----------- | ------------- | ----------- | --------- | ---------- |
| 1 000       | 15 µs        | 18 µs      | 45 µs    | 209        |
| 10 000      | 180 µs       | 220 µs     | 350 µs   | 2 090      |
| 100 000     | 3.2 ms        | 4.1 ms      | 5.8 ms    | 20 899     |
| 1 000 000   | 85 ms         | 110 ms      | 95 ms     | 208 988    |
| 10 000 000  | 2.1 s         | 2.8 s       | 2.3 s     | 2 089 877  |
| 100 000 000 | 45 s          | 62 s        | 48 s      | 20 898 764 |
| 250 000 000 | 3 m 12 s      | 4 m 25 s    | 3 m 28 s  | 52 246 909 |

### 1.7.3 Baselines mémoire Go (estimées)

| N             | Mémoire estimée (peak RSS) |
| ------------- | ---------------------------- |
| 10 000 000    | ~120 MB                      |
| 100 000 000   | ~1.2 GB                      |
| 1 000 000 000 | ~12 GB                       |

### 1.7.4 Seuils de régression Rust

**Règle** : Aucune régression >5% n'est acceptable sur une même machine avec la même charge.

| Métrique           | Seuil d'alerte (jaune) | Seuil d'échec (rouge) |
| ------------------- | ---------------------- | ---------------------- |
| Temps d'exécution  | +3%                    | +5%                    |
| Mémoire peak RSS   | +5%                    | +10%                   |
| Allocations totales | +10%                   | +25%                   |
| Temps de démarrage | +10ms                  | +20ms                  |

### 1.7.5 Stratégie de détection

1. **Benchmarks `criterion`** avec sauvegarde des résultats de base dans `target/criterion/`
2. **CI avec comparaison** : chaque PR exécute les benchmarks et compare avec la baseline
3. **Script `bench-compare.sh`** : exécute Go et Rust dos-à-dos sur la même machine
4. **Alertes** : les régressions >3% génèrent un commentaire de PR automatique

```toml
# Cargo.toml — benchmarks
[[bench]]
name = "fibonacci_bench"
harness = false

[dev-dependencies]
criterion = { version = "0.5", features = ["html_reports"] }
```

```rust
// benches/fibonacci_bench.rs
use criterion::{criterion_group, criterion_main, Criterion, BenchmarkId};

fn benchmark_fast_doubling(c: &mut Criterion) {
    let mut group = c.benchmark_group("FastDoubling");
    for n in [1_000, 10_000, 100_000, 1_000_000, 10_000_000] {
        group.bench_with_input(BenchmarkId::new("N", n), &n, |b, &n| {
            b.iter(|| fast_doubling(n));
        });
    }
    group.finish();
}

criterion_group!(benches, benchmark_fast_doubling);
criterion_main!(benches);
```

---

## T1.8 — Exigences non fonctionnelles (NFR)

### 1.8.1 Performance

| #      | Exigence                                       | Cible                  | Méthode de mesure                                      |
| ------ | ---------------------------------------------- | ---------------------- | ------------------------------------------------------- |
| NFR-P1 | Temps de démarrage (cold start, aucun calcul) | < 50 ms (p99)          | `hyperfine --warmup 3 --runs 100 ./fibcalc-rs --help` |
| NFR-P2 | Latence premier résultat pour N ≤ 93         | < 1 ms                 | Benchmark unitaire                                      |
| NFR-P3 | Débit de calcul F(10M) Fast Doubling          | ≤ 2.1 s (ref Ryzen 9) | `criterion`                                           |
| NFR-P4 | Surcoût du TUI vs CLI (même calcul)          | < 5%                   | Comparaison temps CLI vs TUI                            |

### 1.8.2 Mémoire

| #      | Exigence                                            | Cible                   | Méthode de mesure               |
| ------ | --------------------------------------------------- | ----------------------- | -------------------------------- |
| NFR-M1 | RSS au démarrage (aucun calcul)                    | < 10 MB                 | `/proc/self/status` VmRSS      |
| NFR-M2 | RSS pour F(10M) Fast Doubling                       | ≤ 130 MB               | Mesure runtime                   |
| NFR-M3 | Mode `--last-digits K` mémoire                   | O(K), indépendant de N | Test avec N=10^12, K=100         |
| NFR-M4 | Pas de fuite mémoire sur 1000 calculs consécutifs | RSS stable (±5%)       | Test en boucle avec `valgrind` |

### 1.8.3 Taille du binaire

| #      | Exigence                              | Cible  | Méthode de mesure                               |
| ------ | ------------------------------------- | ------ | ------------------------------------------------ |
| NFR-B1 | Binaire stripped (release, pure-Rust) | < 5 MB | `strip fibcalc-rs && ls -la`                   |
| NFR-B2 | Binaire avec feature GMP              | < 8 MB | Idem avec `--features gmp`                     |
| NFR-B3 | Temps de compilation incrémentale    | < 15 s | `cargo build` après modification d'un fichier |

### 1.8.4 Fiabilité

| #      | Exigence                                             | Cible                                | Méthode de mesure                    |
| ------ | ---------------------------------------------------- | ------------------------------------ | ------------------------------------- |
| NFR-R1 | Zéro panic en fonctionnement normal                 | 0 panics                             | Tests E2E + fuzz testing              |
| NFR-R2 | Timeout configurable respecté                       | Arrêt dans les 100ms après timeout | Test avec `--timeout 1s` sur N=10^9 |
| NFR-R3 | Ctrl+C arrête proprement le calcul                  | Exit code 130, pas de corruption     | Test signal SIGINT                    |
| NFR-R4 | Résultats déterministes (même N, même résultat) | Identique sur 100 exécutions        | Script de comparaison                 |

### 1.8.5 Portabilité

| #      | Exigence                                       | Cible                  | Méthode de mesure          |
| ------ | ---------------------------------------------- | ---------------------- | --------------------------- |
| NFR-X1 | Compilation sur 5 target triples               | 5/5 réussis           | CI matrix                   |
| NFR-X2 | Pas de dépendance système (hors feature GMP) | 0 libs dynamiques      | `ldd` / `otool -L`      |
| NFR-X3 | Support NO_COLOR standard                      | Couleurs désactivées | `NO_COLOR=1 ./fibcalc-rs` |

### 1.8.6 Sécurité

| #      | Exigence                                          | Cible                                      | Méthode de mesure                 |
| ------ | ------------------------------------------------- | ------------------------------------------ | ---------------------------------- |
| NFR-S1 | Aucune vulnérabilité connue dans les deps       | 0 advisories                               | `cargo audit`                    |
| NFR-S2 | Blocs `unsafe` minimaux et documentés          | ≤ 5, tous avec commentaire `// SAFETY:` | `cargo geiger` + audit manuel    |
| NFR-S3 | Pas d'injection de commande via les arguments CLI | Validation stricte                         | Tests de fuzzing sur les arguments |

### 1.8.7 Observabilité

| #      | Exigence                                               | Cible                           | Méthode de mesure              |
| ------ | ------------------------------------------------------ | ------------------------------- | ------------------------------- |
| NFR-O1 | Logging structuré configurable (niveaux TRACE→ERROR) | 5 niveaux                       | `RUST_LOG=trace ./fibcalc-rs` |
| NFR-O2 | Métriques de progression (bits/s, digits/s, steps/s)  | Affichage en mode `--details` | Test de sortie                  |
| NFR-O3 | Profil de calibration sérialisable en JSON            | Lecture/écriture JSON          | Test serde round-trip           |

### 1.8.8 Ergonomie

| #      | Exigence                                               | Cible                       | Méthode de mesure                   |
| ------ | ------------------------------------------------------ | --------------------------- | ------------------------------------ |
| NFR-E1 | Messages d'erreur clairs avec suggestion de correction | Toutes les erreurs config   | Test E2E des messages d'erreur       |
| NFR-E2 | Shell completion pour 4 shells                         | bash, zsh, fish, powershell | Génération + validation syntaxique |
| NFR-E3 | `--help` lisible et complet                          | Tous les flags documentés  | Comparaison avec la doc              |

---

## T1.9 — Matrice de cross-compilation

### 1.9.1 Targets supportés

| #  | Target Triple                 | OS                    | Arch   | SIMD         | Priorité |
| -- | ----------------------------- | --------------------- | ------ | ------------ | --------- |
| T1 | `x86_64-unknown-linux-gnu`  | Linux                 | x86_64 | AVX2/AVX-512 | P0        |
| T2 | `x86_64-unknown-linux-musl` | Linux (statique)      | x86_64 | AVX2/AVX-512 | P1        |
| T3 | `x86_64-pc-windows-msvc`    | Windows               | x86_64 | AVX2/AVX-512 | P1        |
| T4 | `x86_64-apple-darwin`       | macOS                 | x86_64 | AVX2         | P1        |
| T5 | `aarch64-apple-darwin`      | macOS (Apple Silicon) | ARM64  | NEON         | P1        |

### 1.9.2 Instructions de build par target

#### T1 : `x86_64-unknown-linux-gnu` (target principal)

```bash
# Installation toolchain
rustup default stable
rustup target add x86_64-unknown-linux-gnu

# Build
cargo build --release --target x86_64-unknown-linux-gnu

# Build avec feature GMP
sudo apt-get install libgmp-dev
cargo build --release --target x86_64-unknown-linux-gnu --features gmp

# Vérification
ldd target/x86_64-unknown-linux-gnu/release/fibcalc-rs
# → uniquement libc, libm, libpthread (+ libgmp si feature gmp)
```

**Problèmes connus** :

- Nécessite glibc ≥ 2.17 (CentOS 7+)
- Pour SIMD AVX-512 : ajouter `RUSTFLAGS="-C target-cpu=native"`

#### T2 : `x86_64-unknown-linux-musl` (binaire statique)

```bash
# Installation toolchain musl
rustup target add x86_64-unknown-linux-musl
sudo apt-get install musl-tools

# Build statique
cargo build --release --target x86_64-unknown-linux-musl

# Vérification (aucune dépendance dynamique)
ldd target/x86_64-unknown-linux-musl/release/fibcalc-rs
# → "not a dynamic executable" (succès)
```

**Problèmes connus** :

- Feature GMP non supportée avec musl (libgmp nécessite glibc)
- Taille binaire légèrement plus grande (~10%)

#### T3 : `x86_64-pc-windows-msvc`

```bash
# Depuis Windows avec MSVC Build Tools installé
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc

# Feature GMP (nécessite MSYS2 + mingw-w64-x86_64-gmp)
# Alternative : compiler avec x86_64-pc-windows-gnu
```

**Problèmes connus** :

- GMP difficile à compiler avec MSVC → préférer le target `gnu` ou WSL
- Les tests TUI nécessitent un terminal réel (pas PowerShell ISE)

#### T4 : `x86_64-apple-darwin`

```bash
# Depuis macOS Intel ou avec cross-compilation
rustup target add x86_64-apple-darwin
cargo build --release --target x86_64-apple-darwin

# Feature GMP
brew install gmp
cargo build --release --target x86_64-apple-darwin --features gmp
```

**Problèmes connus** :

- AVX-512 non disponible sur aucun Mac Intel
- Cross-compilation depuis Linux nécessite `osxcross`

#### T5 : `aarch64-apple-darwin` (Apple Silicon)

```bash
# Depuis macOS Apple Silicon
rustup target add aarch64-apple-darwin
cargo build --release --target aarch64-apple-darwin

# Feature GMP
brew install gmp
cargo build --release --target aarch64-apple-darwin --features gmp
```

**Problèmes connus** :

- Pas d'AVX2/AVX-512 → utiliser NEON intrinsèques ou scalar fallback
- Les opérations SIMD doivent avoir un fallback `#[cfg(not(target_arch = "x86_64"))]`
- Cross-compilation depuis Linux x86_64 nécessite Xcode SDKs

### 1.9.3 CI Matrix

```yaml
# .github/workflows/ci.yml (extrait)
strategy:
  matrix:
    include:
      - os: ubuntu-latest
        target: x86_64-unknown-linux-gnu
      - os: ubuntu-latest
        target: x86_64-unknown-linux-musl
      - os: windows-latest
        target: x86_64-pc-windows-msvc
      - os: macos-latest
        target: x86_64-apple-darwin
      - os: macos-latest
        target: aarch64-apple-darwin
```

---

## T1.10 — Compatibilité des licences

### 1.10.1 Licence du projet source

FibGo est sous **Apache License 2.0**, une licence permissive compatible avec la majorité des licences open source.

### 1.10.2 Matrice de licences des dépendances Rust

| Crate                  | Licence          | Compatible Apache-2.0  | Notes                                                                                       |
| ---------------------- | ---------------- | ---------------------- | ------------------------------------------------------------------------------------------- |
| `num-bigint`         | MIT / Apache-2.0 | Oui                    | Dual-license, choix libre                                                                   |
| `num-traits`         | MIT / Apache-2.0 | Oui                    | Dual-license                                                                                |
| `rug`                | LGPL-3.0+        | **Conditionnel** | Via GMP (LGPL). OK si linking dynamique. Problématique pour binaire statique redistribué. |
| `gmp-mpfr-sys`       | LGPL-3.0+        | **Conditionnel** | Même contrainte que `rug`                                                                |
| `ratatui`            | MIT              | Oui                    |                                                                                             |
| `crossterm`          | MIT              | Oui                    |                                                                                             |
| `clap`               | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `clap_complete`      | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `rayon`              | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `crossbeam`          | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `bumpalo`            | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `parking_lot`        | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `thiserror`          | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `anyhow`             | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `serde`              | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `serde_json`         | MIT / Apache-2.0 | Oui                    |                                                                                             |
| `tracing`            | MIT              | Oui                    |                                                                                             |
| `tracing-subscriber` | MIT              | Oui                    |                                                                                             |
| `criterion`          | MIT / Apache-2.0 | Oui                    | dev-dependency uniquement                                                                   |
| `proptest`           | MIT / Apache-2.0 | Oui                    | dev-dependency uniquement                                                                   |
| `sysinfo`            | MIT              | Oui                    |                                                                                             |
| `indicatif`          | MIT              | Oui                    |                                                                                             |
| `console`            | MIT              | Oui                    |                                                                                             |

### 1.10.3 Analyse de la problématique GMP / LGPL

**Constat** : La crate `rug` (bindings Rust vers GMP/MPFR/MPC) hérite de la licence LGPL-3.0+ de GMP.

**Implications** :

- **Linking dynamique** : Compatible avec Apache-2.0. L'utilisateur peut remplacer la bibliothèque GMP.
- **Linking statique** : Zone grise LGPL. Nécessite de fournir le code objet permettant la re-liaison.
- **Redistribution binaire** : Contraintes LGPL à respecter (sources GMP, instructions de re-liaison).

**Recommandation** :

1. Le binaire par défaut (`cargo build --release`) n'inclut **pas** GMP → 100% Apache-2.0
2. La feature `gmp` est **optionnelle** et documentée avec les contraintes LGPL
3. Pour les distributions binaires avec GMP : fournir les instructions de re-liaison LGPL
4. Alternative : `ibig` (MIT/Apache-2.0) si les performances sont suffisantes

### 1.10.4 Configuration `cargo-deny`

```toml
# deny.toml
[licenses]
unlicensed = "deny"
allow = ["MIT", "Apache-2.0", "BSD-2-Clause", "BSD-3-Clause", "ISC", "Unicode-DFS-2016"]
deny = ["GPL-2.0", "GPL-3.0", "AGPL-3.0"]

# LGPL autorisé uniquement pour la feature gmp
exceptions = [
    { allow = ["LGPL-3.0"], name = "gmp-mpfr-sys" },
    { allow = ["LGPL-3.0"], name = "rug" },
]

[licenses.private]
ignore = false
```

---

## T1.11 — Timeline de migration raffinée

### 1.11.1 Estimation de l'effort

| Package Go                                      | LoC Go (approx.)  | Complexité portage                   | LoC Rust (estimé) | Durée estimée           |
| ----------------------------------------------- | ----------------- | ------------------------------------- | ------------------ | ------------------------- |
| `internal/fibonacci` (algorithmes)            | ~3 500            | Élevée (BigInt, FFT, parallélisme) | ~4 500             | 3-4 semaines              |
| `internal/bigfft`                             | ~2 500            | Très élevée (FFT Fermat, SIMD)     | ~3 000             | 3-4 semaines              |
| `internal/orchestration`                      | ~500              | Moyenne (errgroup → rayon)           | ~500               | 1 semaine                 |
| `internal/cli`                                | ~1 200            | Faible (reformatage)                  | ~1 000             | 1 semaine                 |
| `internal/tui`                                | ~1 800            | Élevée (Bubble Tea → ratatui)      | ~2 200             | 2-3 semaines              |
| `internal/calibration`                        | ~800              | Moyenne                               | ~900               | 1 semaine                 |
| `internal/config`                             | ~400              | Faible (flag → clap)                 | ~300               | 0.5 semaine               |
| `internal/app`                                | ~300              | Faible                                | ~250               | 0.5 semaine               |
| `internal/errors`                             | ~200              | Faible (→ thiserror)                 | ~150               | 0.5 semaine               |
| Support (`format`, `metrics`, `ui`, etc.) | ~600              | Faible                                | ~500               | 1 semaine                 |
| Tests                                           | ~5 000            | Moyenne (réécriture)                | ~5 500             | 3-4 semaines              |
| **Total**                                 | **~16 800** |                                       | **~18 800**  | **~18-22 semaines** |

### 1.11.2 Phases et timeline

```
Phase A : Infrastructure Cargo (2 semaines)
├── Cargo workspace setup
├── Types de base (FibError, Options, structures)
├── CLAUDE.md Rust
└── CI/CD initial (build, test, lint, audit)

Phase B : Arithmétique (4 semaines)                    ← Chemin critique
├── Portage bigfft (FFT Fermat, bump allocator)
├── Portage arithmétique SIMD (scalar + x86_64)
├── Cache LRU FFT
└── Benchmarks comparatifs Go/Rust

Phase C : Algorithmes Fibonacci (3 semaines)           ← Chemin critique
├── Fast Doubling + DoublingFramework
├── Matrix Exponentiation + MatrixFramework
├── FFT-Based Calculator
├── Strategies (Adaptive, FFT-Only, Karatsuba)
└── Golden file validation

Phase D : Orchestration & Observer (2 semaines)
├── ProgressSubject / Observer pattern
├── Orchestrateur parallèle (rayon)
├── Comparaison inter-algorithmes
└── Validation croisée Go/Rust complète

Phase E : CLI (2 semaines)
├── Parsing clap + env vars
├── Formatage de sortie + spinner
├── Shell completion
└── Tests E2E

Phase F : TUI (3 semaines)
├── Spike architectural ratatui
├── Modèle Elm (Init/Update/View)
├── Panels (header, logs, metrics, chart, footer)
├── Bridge ProgressReporter/ResultPresenter
└── Tests de rendu

Phase G : Calibration & Polish (2 semaines)
├── Système de calibration
├── Seuils dynamiques
├── Documentation (rustdoc, README, docs/)
└── Release binaires multi-plateforme
```

### 1.11.3 Graphe de dépendances

```
Phase A ──→ Phase B ──→ Phase C ──→ Phase D ──→ Phase E
                                       ↓
                                    Phase F
                                       ↓
                                    Phase G

Chemin critique : A → B → C → D → G
Durée chemin critique : 2 + 4 + 3 + 2 + 2 = 13 semaines

Avec parallélisation E∥F après D :
Durée totale : 2 + 4 + 3 + 2 + max(2, 3) + 2 = 16 semaines
```

### 1.11.4 Jalons de validation

| Jalon                                   | Semaine | Critère de réussite                                   |
| --------------------------------------- | ------- | ------------------------------------------------------- |
| M1 : FFT multiplication opérationnelle | S6      | `mulFFT` et `sqrFFT` produisent résultats corrects |
| M2 : Fast Doubling F(1M) correct        | S9      | Golden test + comparaison Go                            |
| M3 : 3 algorithmes + orchestration      | S11     | Validation croisée complète N ≤ 10M                  |
| M4 : CLI fonctionnel                    | S13     | Toutes les flags opérationnelles, E2E pass             |
| M5 : TUI fonctionnel                    | S14     | Dashboard interactif avec 5 panels                      |
| M6 : Release candidate                  | S16     | Tous les NFR satisfaits, benchmarks verts               |

---

## T1.12 — Spécification CLAUDE.md pour le projet Rust

Le fichier `CLAUDE.md` suivant sera placé à la racine du projet Rust pour guider Claude Code.

---

### Contenu du CLAUDE.md Rust

```markdown
# CLAUDE.md — FibCalc-rs

Ce fichier fournit les instructions pour Claude Code lors du travail sur ce dépôt.

## Commandes Build & Test

# Commandes Cargo essentielles
cargo build --release                                    # Build release
cargo test                                               # Tous les tests
cargo test -- --nocapture                                # Tests avec sortie stdout
cargo test --lib                                         # Tests unitaires uniquement
cargo test --test golden                                 # Tests golden files
cargo test --test e2e                                    # Tests end-to-end
cargo test -p fibcalc-bigfft                             # Tests d'un crate spécifique
cargo bench                                              # Benchmarks criterion
cargo bench -- "FastDoubling"                            # Benchmark spécifique
cargo clippy -- -W clippy::pedantic                      # Lint strict
cargo fmt --check                                        # Vérifier le formatage
cargo audit                                              # Audit sécurité des deps
cargo deny check                                         # Vérification licences
cargo tarpaulin --out html                               # Couverture de code
cargo fuzz run fuzz_fast_doubling -- -max_total_time=30  # Fuzz testing

# Build avec features
cargo build --release --features gmp                     # Avec support GMP (LGPL)

## Vue d'ensemble de l'architecture

**Crate** : `fibcalc-rs` (Rust 2024 edition, MSRV 1.80+)

Calculateur Fibonacci haute performance avec modes CLI et TUI. Quatre couches :

    Point d'entrée (src/main.rs)
        ↓
    Orchestration (src/orchestration/)  — exécution parallèle, agrégation résultats
        ↓
    Métier (src/fibonacci/, src/bigfft/)  — algorithmes, multiplication FFT
        ↓
    Présentation (src/cli/, src/tui/)  — sortie CLI ou dashboard TUI

### Traits clés

**Calculator** (src/fibonacci/calculator.rs) : Trait public consommé par l'orchestration.
Méthodes : `calculate()`, `name()`.

**CoreCalculator** (src/fibonacci/calculator.rs) : Trait interne pour les implémentations
d'algorithmes. Méthodes : `calculate_core()`, `name()`. Encapsulé par `FibCalculator`
(décorateur) qui ajoute le fast path n ≤ 93 et le reporting de progression.

**Multiplier** (src/fibonacci/strategy.rs) : Interface étroite pour multiply/square.
Étendu par `DoublingStepExecutor` pour les pas optimisés.

**ProgressObserver** (src/fibonacci/observer.rs) : Pattern Observer pour les mises à jour
de progression. `Freeze()` crée des snapshots lock-free pour les boucles chaudes.

### Structure Cargo workspace

    fibcalc-rs/
    ├── Cargo.toml              # Workspace root
    ├── crates/
    │   ├── fibcalc/            # Binaire principal
    │   ├── fibcalc-core/       # Algorithmes Fibonacci + orchestration
    │   ├── fibcalc-bigfft/     # Multiplication FFT
    │   ├── fibcalc-tui/        # Dashboard TUI ratatui
    │   └── fibcalc-cli/        # Sortie CLI, spinners, completion
    ├── tests/                  # Tests d'intégration
    │   ├── golden.rs
    │   └── e2e.rs
    ├── benches/                # Benchmarks criterion
    │   └── fibonacci_bench.rs
    └── fuzz/                   # Cibles de fuzz testing
        └── fuzz_targets/

## Conventions de code

**Imports** : Grouper comme (1) std, (2) crates externes, (3) crates workspace.

**Gestion d'erreurs** : `thiserror` pour les erreurs de bibliothèque, `anyhow` dans main.
Enum `FibError` avec variantes : Calculation, Config, Cancelled, Timeout, Mismatch.

**Concurrence** : `rayon` pour le parallélisme CPU-bound. `crossbeam::channel` pour
la communication. Sémaphore via `rayon::ThreadPool` avec taille limitée.

**Tests** : Table-driven avec `#[test]`. >75% couverture cible. Golden files dans
`tests/testdata/fibonacci_golden.json`. Fuzz targets dans `fuzz/fuzz_targets/`.
Property-based via `proptest`. Benchmarks via `criterion`.

**Linting** : `cargo clippy -- -W clippy::pedantic`. Complexité cyclomatique < 15.

**Commits** : Conventional Commits — `feat`, `fix`, `docs`, `refactor`, `perf`, `test`, `chore`.

## Patterns clés

- **Decorator** : `FibCalculator` encapsule `CoreCalculator`
- **Factory + Registry** : `DefaultFactory` avec création paresseuse et cache
- **Strategy + ISP** : `Multiplier` (étroit) et `DoublingStepExecutor` (large)
- **Observer** : `ProgressSubject`/`ProgressObserver` avec `Freeze()` lock-free
- **Arena** : `bumpalo::Bump` pour les temporaires FFT
- **Zero-copy** : `std::mem::take` / `std::mem::replace` pour le retour de résultat

## Features Cargo

- `default` : Pure Rust, pas de dépendance externe
- `gmp` : Support GMP via `rug` (LGPL, nécessite libgmp)
- `simd` : Optimisations SIMD explicites (auto-détection par défaut)

## Configuration

Flags CLI > Variables d'environnement (`FIBCALC_*`) > Calibration adaptative > Défauts statiques.

Seuils par défaut : ParallelThreshold=4096 bits, FFTThreshold=500K bits, StrassenThreshold=3072 bits.

## Dépendances clés

| Crate | Rôle |
|-------|------|
| `num-bigint` | Arithmétique grands nombres |
| `rayon` | Parallélisme work-stealing |
| `crossbeam` | Channels et primitives concurrentes |
| `ratatui` + `crossterm` | TUI interactif |
| `clap` + `clap_complete` | Parsing CLI + shell completion |
| `bumpalo` | Bump allocator pour FFT |
| `parking_lot` | Mutexes performants |
| `tracing` | Logging structuré |
| `thiserror` | Dérivation d'erreurs |
| `serde` + `serde_json` | Sérialisation (profils calibration) |
| `criterion` | Benchmarks (dev) |
| `proptest` | Property-based testing (dev) |
```

---

## Annexe A — Correspondance des constantes Go → Rust

| Constante Go                 | Valeur           | Fichier source    | Équivalent Rust                                    |
| ---------------------------- | ---------------- | ----------------- | --------------------------------------------------- |
| `DefaultParallelThreshold` | 4 096 bits       | `constants.go`  | `const DEFAULT_PARALLEL_THRESHOLD: usize = 4096`  |
| `DefaultFFTThreshold`      | 500 000 bits     | `constants.go`  | `const DEFAULT_FFT_THRESHOLD: usize = 500_000`    |
| `DefaultStrassenThreshold` | 3 072 bits       | `constants.go`  | `const DEFAULT_STRASSEN_THRESHOLD: usize = 3072`  |
| `ParallelFFTThreshold`     | 5 000 000 bits   | `constants.go`  | `const PARALLEL_FFT_THRESHOLD: usize = 5_000_000` |
| `CalibrationN`             | 10 000 000       | `constants.go`  | `const CALIBRATION_N: u64 = 10_000_000`           |
| `ProgressReportThreshold`  | 0.01 (1%)        | `constants.go`  | `const PROGRESS_REPORT_THRESHOLD: f64 = 0.01`     |
| `MaxFibUint64`             | 93               | `calculator.go` | `const MAX_FIB_U64: u64 = 93`                     |
| `MaxPooledBitLen`          | 100 000 000 bits | `common.go`     | `const MAX_POOLED_BIT_LEN: usize = 100_000_000`   |

## Annexe B — Correspondance des codes de sortie

| Code Go | Constante Go          | Signification                | Code Rust                 |
| ------- | --------------------- | ---------------------------- | ------------------------- |
| 0       | `ExitSuccess`       | Succès                      | `std::process::exit(0)` |
| 1       | `ExitErrorGeneric`  | Erreur générique           | `FibError::Calculation` |
| 2       | `ExitErrorTimeout`  | Timeout                      | `FibError::Timeout`     |
| 3       | `ExitErrorMismatch` | Désaccord entre algorithmes | `FibError::Mismatch`    |
| 4       | `ExitErrorConfig`   | Erreur de configuration      | `FibError::Config`      |
| 130     | `ExitErrorCanceled` | Annulation (Ctrl+C)          | `FibError::Cancelled`   |

## Annexe C — Correspondance des flags CLI

| Flag Go                 | Type     | Défaut     | Flag Rust (clap)                | Notes                                          |
| ----------------------- | -------- | ----------- | ------------------------------- | ---------------------------------------------- |
| `-n`                  | uint64   | 100 000 000 | `-n <N>`                      | `#[arg(short, default_value = "100000000")]` |
| `-algo`               | string   | "all"       | `--algo <ALGO>`               | `#[arg(long, default_value = "all")]`        |
| `-calculate` / `-c` | bool     | false       | `--calculate` / `-c`        | `#[arg(short, long)]`                        |
| `-verbose` / `-v`   | bool     | false       | `--verbose` / `-v`          | `#[arg(short, long)]`                        |
| `-details` / `-d`   | bool     | false       | `--details` / `-d`          | `#[arg(short, long)]`                        |
| `-output` / `-o`    | string   | ""          | `--output <PATH>`             | `#[arg(short, long)]`                        |
| `-quiet` / `-q`     | bool     | false       | `--quiet` / `-q`            | `#[arg(short, long)]`                        |
| `-calibrate`          | bool     | false       | `--calibrate`                 | `#[arg(long)]`                               |
| `-auto-calibrate`     | bool     | false       | `--auto-calibrate`            | `#[arg(long)]`                               |
| `-timeout`            | duration | 5m          | `--timeout <DURATION>`        | Parse custom (e.g., "5m", "1h")                |
| `-threshold`          | int      | 0           | `--threshold <BITS>`          | `#[arg(long, default_value = "0")]`          |
| `-fft-threshold`      | int      | 0           | `--fft-threshold <BITS>`      | `#[arg(long, default_value = "0")]`          |
| `-strassen-threshold` | int      | 0           | `--strassen-threshold <BITS>` | `#[arg(long, default_value = "0")]`          |
| `-tui`                | bool     | false       | `--tui`                       | `#[arg(long)]`                               |
| `-completion`         | string   | ""          | `--completion <SHELL>`        | `#[arg(long, value_enum)]`                   |
| `--version` / `-V`  | bool     | false       | `--version` / `-V`          | Automatique avec clap                          |
| `--last-digits`       | int      | 0           | `--last-digits <K>`           | `#[arg(long, default_value = "0")]`          |
| `--memory-limit`      | string   | ""          | `--memory-limit <SIZE>`       | Parse custom (e.g., "8G")                      |
| `--gc-control`        | string   | "auto"      | N/A                             | Non applicable en Rust (pas de GC)             |

> **Note** : Le flag `--gc-control` de Go n'a pas d'équivalent en Rust puisqu'il n'y a pas de garbage collector. Les allocations sont gérées via RAII et l'arena allocator.

---

*Fin de la Phase 1 — Fondations & Analyse des exigences*
*Document rédigé en français pour le projet FibCalc-rs*
*Prochaine phase : Phase 2 — Spécifications algorithmiques détaillées (T2.1–T2.18)*

# Phase 2 — Spécifications Algorithmiques

> **Portage FibGo → Rust** | Tâches T2.1 à T2.18
> Chaque section spécifie un algorithme ou sous-système avec pseudocode, diagrammes d'état, formules mathématiques et notes de traduction Rust.

---

## Table des matières

| Tâche | Titre                                                             | Ligne  |
| ------ | ----------------------------------------------------------------- | ------ |
| T2.1   | Fast Doubling : itération bit-à-bit et transitions d'état      | §2.1  |
| T2.2   | Fast Doubling : logique de parallélisation                       | §2.2  |
| T2.3   | Exponentiation Matricielle : gestion d'état et pooling           | §2.3  |
| T2.4   | Strassen : logique de commutation et fallback                     | §2.4  |
| T2.5   | Sélection des paramètres FFT (k, n, modulus de Fermat)          | §2.5  |
| T2.6   | Arithmétique de Fermat (Shift, Mul, Sqr, normalisation)          | §2.6  |
| T2.7   | FFT : structure récursive et cas de base                         | §2.7  |
| T2.8   | Opérations polynomiales (Poly, PolValues, transformées)         | §2.8  |
| T2.9   | Optimisation de réutilisation de transformée FFT pour le carré | §2.9  |
| T2.10  | Sélection adaptative de stratégie de multiplication             | §2.10 |
| T2.11  | Fast Doubling Modulaire (--last-digits)                           | §2.11 |
| T2.12  | Fast path itératif (n ≤ 93)                                     | §2.12 |
| T2.13  | Retour résultat zéro-copie                                      | §2.13 |
| T2.14  | Méthodologie de comparaison inter-algorithmes                    | §2.14 |
| T2.15  | Générateur de séquence et optimisation Skip                    | §2.15 |
| T2.16  | Sélection de calculateur depuis la configuration                 | §2.16 |
| T2.17  | Preuves de correction algorithmique                               | §2.17 |
| T2.18  | Carte de couverture de tests par algorithme                       | §2.18 |

---

## T2.1 — Fast Doubling : itération bit-à-bit et transitions d'état

### 2.1.1 Fondement mathématique

L'algorithme Fast Doubling repose sur deux identités dérivées de la matrice Q de Fibonacci :

```
Q = [[1, 1], [1, 0]]
Q^n = [[F(n+1), F(n)], [F(n), F(n-1)]]
```

Par élévation au carré de Q^k, on obtient :

```
F(2k)   = F(k) × [2·F(k+1) - F(k)]
F(2k+1) = F(k+1)² + F(k)²
```

**Reformulation utilisée dans le code** (équivalente algébriquement) :

```
F(2k)   = 2·F(k)·F(k+1) - F(k)²
F(2k+1) = F(k+1)² + F(k)²
```

### 2.1.2 Pseudocode MSB→LSB

L'algorithme itère sur les bits de `n` du bit de poids fort (MSB) vers le bit de poids faible (LSB).

```
FONCTION FastDoubling(n: u64) → BigInt:
    SOIT numBits = bits_significatifs(n)   // bits.Len64(n)
    SOIT FK  = BigInt(0)                    // F(k) courant
    SOIT FK1 = BigInt(1)                    // F(k+1) courant
    SOIT T1, T2, T3 = BigInt temporaires

    POUR i DE (numBits - 1) VERS 0 DÉCROISSANT:
        // ── Étape de doublement ──
        T3 ← FK × FK1                      // Produit croisé
        T1 ← FK1²                          // Carré de F(k+1)
        T2 ← FK²                           // Carré de F(k)

        // Post-multiplication : calcul de F(2k) et F(2k+1)
        T3 ← T3 << 1                       // T3 = 2·FK·FK1
        T3 ← T3 - T2                       // T3 = F(2k) = 2·FK·FK1 - FK²
        T1 ← T1 + T2                       // T1 = F(2k+1) = FK1² + FK²

        // ── Rotation de pointeurs (zéro-copie) ──
        (FK, FK1, T2, T3, T1) ← (T3, T1, FK, FK1, T2)

        // ── Étape d'addition conditionnelle ──
        SI bit(n, i) == 1:
            T1 ← FK + FK1                  // Somme
            (FK, FK1, T1) ← (FK1, T1, FK)  // Rotation

    RETOURNER FK
```

### 2.1.3 Diagramme d'état de la machine

```
┌─────────────────────────────────────────────────────┐
│                    INITIALISATION                     │
│  FK=0, FK1=1, i=numBits-1                           │
└──────────────────────┬──────────────────────────────┘
                       │
                       ▼
              ┌────────────────┐
              │  i >= 0 ?      │──── NON ──→ [RETOURNER FK]
              └───────┬────────┘
                  OUI │
                      ▼
        ┌──────────────────────────┐
        │   ÉTAPE_DOUBLEMENT       │
        │   T3 = FK × FK1         │
        │   T1 = FK1²             │
        │   T2 = FK²              │
        │   T3 = 2·T3 - T2        │
        │   T1 = T1 + T2          │
        └────────────┬─────────────┘
                     │
                     ▼
        ┌──────────────────────────┐
        │   ROTATION_POINTEURS     │
        │   FK ← T3 (= F(2k))     │
        │   FK1 ← T1 (= F(2k+1))  │
        │   T2 ← ancien FK        │
        │   T3 ← ancien FK1       │
        │   T1 ← ancien T2        │
        └────────────┬─────────────┘
                     │
                     ▼
              ┌────────────────┐
              │ bit(n,i) == 1?│
              └───┬────────┬───┘
              OUI │        │ NON
                  ▼        │
        ┌──────────────┐   │
        │ ÉTAPE_ADDITION│   │
        │ T1 = FK+FK1  │   │
        │ FK ← FK1     │   │
        │ FK1 ← T1     │   │
        │ T1 ← anc. FK │   │
        └──────┬───────┘   │
               │           │
               ▼           ▼
        ┌──────────────────────┐
        │      i ← i - 1       │
        └──────────┬───────────┘
                   │
                   └──→ (retour au test i >= 0)
```

### 2.1.4 Rotation de pointeurs — Détail de l'échange 5-way

La ligne Go `s.FK, s.FK1, s.T2, s.T3, s.T1 = s.T3, s.T1, s.FK, s.FK1, s.T2` effectue un échange simultané de 5 pointeurs. C'est une opération O(1) qui évite toute copie des données `big.Int`.

**Avant rotation :**

| Variable | Contient          | Signification |
| -------- | ----------------- | ------------- |
| FK       | Ancien F(k)       | Obsolète     |
| FK1      | Ancien F(k+1)     | Obsolète     |
| T1       | FK1² + FK²      | = F(2k+1)     |
| T2       | FK²              | Temporaire    |
| T3       | 2·FK·FK1 - FK² | = F(2k)       |

**Après rotation :**

| Variable | Contient   | Signification  |
| -------- | ---------- | -------------- |
| FK       | F(2k)      | Nouveau F(k)   |
| FK1      | F(2k+1)    | Nouveau F(k+1) |
| T1       | Ancien T2  | Libre          |
| T2       | Ancien FK  | Libre          |
| T3       | Ancien FK1 | Libre          |

### 2.1.5 Notes de traduction Rust

```rust
// Rust : l'échange de pointeurs se traduit par std::mem::swap ou réaffectation de Box<BigUint>
// Tuple destructuring natif en Rust :
let (fk, fk1, t2, t3, t1) = (t3, t1, fk, fk1, t2);

// Attention : en Rust, BigUint de la crate `num-bigint` n'a pas de sémantique de pointeur
// implicite comme *big.Int en Go. On utilisera Box<BigUint> ou directement la valeur
// avec des std::mem::swap pour l'échange zéro-copie.
```

**Complexité** : O(log n) itérations × O(M(n)) par multiplication, où M(n) est le coût de multiplication de nombres à n bits.

---

## T2.2 — Fast Doubling : logique de parallélisation

### 2.2.1 Arbre de décision parallèle/séquentiel

La parallélisation des 3 multiplications du doublement est conditionnée par plusieurs critères :

```
FONCTION ShouldParallelize(opts, fkBitLen, fk1BitLen) → bool:
    maxBitLen ← max(fkBitLen, fk1BitLen)

    // Cas FFT : les opérations FFT saturent les cœurs CPU
    SI opts.FFTThreshold > 0 ET maxBitLen > opts.FFTThreshold:
        RETOURNER maxBitLen > ParallelFFTThreshold  // 5 000 000 bits

    // Cas standard : paralléliser si assez grand
    threshold ← opts.ParallelThreshold  // défaut: 4096 bits
    RETOURNER maxBitLen > threshold
```

**Diagramme de décision :**

```
┌─────────────────────────┐
│ GOMAXPROCS > 1 ET       │
│ ParallelThreshold > 0 ? │
└───────┬─────────┬───────┘
    OUI │         │ NON
        ▼         └──→ [SÉQUENTIEL]
┌───────────────────┐
│ maxBitLen >       │
│ FFTThreshold ?    │
└───┬───────────┬───┘
OUI │           │ NON
    ▼           ▼
┌───────────┐ ┌──────────────────┐
│ maxBitLen │ │ maxBitLen >      │
│ > 5M bits?│ │ ParallelThres. ? │
└──┬────┬───┘ └──┬──────────┬────┘
OUI│  NON│    OUI │       NON│
   ▼     ▼       ▼          ▼
[PAR] [SEQ]   [PAR]      [SEQ]
```

### 2.2.2 Fork-Join 3-way

Lorsque la parallélisation est activée, les 3 multiplications sont lancées en parallèle :

```
FONCTION ExecuteDoublingStepParallel(ctx, strategy, state, opts):
    errCollector ← nouvel ErrorCollector
    wg ← WaitGroup(3)

    // Goroutine 1 : T3 = FK × FK1 (produit croisé)
    LANCER goroutine:
        SI ctx.annulé: errCollector.set(erreur); RETOUR
        state.T3 ← strategy.Multiply(state.T3, state.FK, state.FK1, opts)

    // Goroutine 2 : T1 = FK1² (carré)
    LANCER goroutine:
        SI ctx.annulé: errCollector.set(erreur); RETOUR
        state.T1 ← strategy.Square(state.T1, state.FK1, opts)

    // Goroutine 3 : T2 = FK² (carré)
    LANCER goroutine:
        SI ctx.annulé: errCollector.set(erreur); RETOUR
        state.T2 ← strategy.Square(state.T2, state.FK, opts)

    wg.Wait()
    RETOURNER errCollector.Err()
```

**Sécurité mémoire** : Chaque goroutine écrit dans un tampon destination distinct (T1, T2, T3) et lit les mêmes sources (FK, FK1) en lecture seule. Aucun verrou n'est nécessaire.

### 2.2.3 Collecte d'erreurs

Le pattern `parallel.ErrorCollector` utilise `atomic.CompareAndSwap` pour capturer la première erreur parmi les goroutines concurrentes :

```rust
// Traduction Rust : utiliser un Arc<Mutex<Option<Error>>> ou
// std::sync::atomic pour first-error-wins
// Alternativement : tokio::task::JoinSet avec .join_next().await
```

### 2.2.4 Sémaphore de tâches

Le sémaphore global `taskSemaphore` limite la concurrence à `NumCPU × 2` goroutines :

```
CAPACITÉ_SÉMAPHORE = runtime.NumCPU() × 2
```

En Rust, cela se traduit par un `tokio::sync::Semaphore` ou un `crossbeam::channel::bounded`.

### 2.2.5 Interaction avec la FFT

Lorsque les opérandes dépassent `FFTThreshold`, la parallélisation au niveau Fibonacci est **désactivée** (sauf pour opérandes > 5M bits) car le moteur FFT sature déjà les cœurs CPU via son propre sémaphore (`bigfft.concurrencySemaphore` limité à `NumCPU`).

| Couche    | Sémaphore           | Capacité       | Condition d'activation                                     |
| --------- | -------------------- | --------------- | ---------------------------------------------------------- |
| Fibonacci | taskSemaphore        | NumCPU × 2     | maxBitLen > ParallelThreshold ET maxBitLen ≤ FFTThreshold |
| FFT       | concurrencySemaphore | NumCPU          | Récursion FFT, size ≥ 4, depth < 3                       |
| Les deux  | —                   | NumCPU × 3 max | maxBitLen > ParallelFFTThreshold (5M bits)                 |

---

## T2.3 — Exponentiation Matricielle : gestion d'état et pooling

### 2.3.1 Structure `matrixState`

L'état complet pour l'exponentiation matricielle comprend :

```
matrixState {
    res        : matrix 2×2    // Résultat accumulé (initialisé à I)
    p          : matrix 2×2    // Base courante Q^(2^i)
    tempMatrix : matrix 2×2    // Tampon pour échange de pointeurs

    // Temporaires Strassen (7 produits)
    p1..p7 : BigInt

    // Temporaires Strassen (8 sommes/différences)
    s1..s8 : BigInt

    // Temporaires pour carré symétrique
    t1..t5 : BigInt
}
```

**Total** : 3 matrices (12 BigInt) + 20 BigInt scalaires = **32 BigInt** pré-alloués.

### 2.3.2 Cycle de vie du pooling

```
┌──────────────────┐     ┌──────────────────┐     ┌──────────────────┐
│  sync.Pool       │     │   matrixState     │     │   Calcul         │
│  (matrixStatePool)│     │   acquise         │     │   en cours       │
└────────┬─────────┘     └────────┬──────────┘     └────────┬─────────┘
         │                        │                          │
         │  acquireMatrixState()  │                          │
         │ ───────────────────→   │                          │
         │                        │  Reset() : res=I, p=Q   │
         │                        │ ─────────────────────→   │
         │                        │                          │
         │                        │   Calcul terminé         │
         │                        │ ←─────────────────────   │
         │  releaseMatrixState()  │                          │
         │ ←───────────────────   │                          │
         │                        │                          │
         │  SI checkLimit(...)    │                          │
         │    → abandonné au GC   │                          │
         │  SINON → remis au pool │                          │
         └────────────────────────┘                          │
```

### 2.3.3 Protocole de vérification des limites

Avant remise en pool, **chaque** BigInt est vérifié :

```
FONCTION releaseMatrixState(state):
    SI checkLimit(p1) OU checkLimit(p2) OU ... OU checkLimit(t5)
       OU checkMatrixLimit(res) OU checkMatrixLimit(p) OU checkMatrixLimit(tempMatrix):
        RETOURNER  // Abandonner au GC — objet trop volumineux
    pool.Put(state)

FONCTION checkLimit(z: *BigInt) → bool:
    RETOURNER z ≠ nil ET z.BitLen() > MaxPooledBitLen  // 100M bits
```

### 2.3.4 Optimisation de la matrice symétrique

La matrice de Fibonacci est symétrique (b = c). Le carré d'une matrice symétrique nécessite :

- **4 multiplications** au lieu de 8 (standard) ou 7 (Strassen)
- 3 carrés (a², b², d²) + 1 multiplication (b × (a+d))

```
FONCTION squareSymmetricMatrix(dest, mat, state):
    ad ← mat.a + mat.d
    a2 ← mat.a²
    b2 ← mat.b²
    d2 ← mat.d²
    bAd ← mat.b × ad

    dest.a ← a2 + b2
    dest.b ← bAd
    dest.c ← bAd          // Symétrie préservée
    dest.d ← b2 + d2
```

### 2.3.5 Boucle d'exponentiation binaire (LSB→MSB)

**Note** : Contrairement au Fast Doubling (MSB→LSB), l'exponentiation matricielle itère du LSB au MSB.

```
FONCTION ExecuteMatrixLoop(n, state):
    SI n == 0: RETOURNER BigInt(0)
    exponent ← n - 1
    numBits ← bits_significatifs(exponent)

    POUR i DE 0 À numBits-1:
        SI bit(exponent, i) == 1:
            tempMatrix ← res × p       // Multiplication matricielle
            (res, tempMatrix) ← (tempMatrix, res)  // Échange de pointeurs

        SI i < numBits - 1:
            tempMatrix ← squareSymmetric(p)  // Carré symétrique
            (p, tempMatrix) ← (tempMatrix, p)

    RETOURNER res.a  // Élément [0,0] = F(n)
```

### 2.3.6 Notes de traduction Rust

```rust
// En Rust, le pooling sync.Pool n'existe pas nativement.
// Options :
// 1. crossbeam::queue::ArrayQueue<MatrixState> (lock-free, borné)
// 2. object-pool crate
// 3. thread_local! avec RefCell<Vec<MatrixState>> (zéro contention)
//
// La vérification de limites se traduit par un trait ou une méthode :
// fn fits_pool(&self) -> bool { self.bit_len() <= MAX_POOLED_BIT_LEN }
```

---

## T2.4 — Strassen : logique de commutation et fallback

### 2.4.1 Critères d'activation

La commutation Strassen est déterminée par la taille en bits des éléments matriciels :

```
FONCTION multiplyMatrices(dest, m1, m2, state, inParallel, fftThreshold, strassenThreshold):
    effectiveThreshold ← strassenThreshold
    SI effectiveThreshold == 0:
        effectiveThreshold ← defaultStrassenThresholdBits  // 256 bits par défaut

    SI maxBitLenTwoMatrices(m1, m2) ≤ effectiveThreshold:
        multiplyMatrix2x2(dest, m1, m2, state)  // Naïf : 8 multiplications
    SINON:
        multiplyMatrixStrassen(dest, m1, m2, state)  // Strassen : 7 multiplications
```

### 2.4.2 Strassen-Winograd : les 7 multiplications

L'implémentation utilise la variante Winograd qui réduit les additions de 18 à 15 :

**Pré-calculs (8 additions/soustractions) :**

```
S1 = A₂₁ + A₂₂           S5 = B₁₂ - B₁₁
S2 = S1 - A₁₁             S6 = B₂₂ - S5
S3 = A₁₁ - A₂₁            S7 = B₂₂ - B₁₂
S4 = A₁₂ - S2             S8 = S6 - B₂₁
```

**7 multiplications :**

```
P1 = S2 × S6
P2 = A₁₁ × B₁₁
P3 = A₁₂ × B₂₁
P4 = S3 × S7
P5 = S1 × S5
P6 = S4 × B₂₂
P7 = A₂₂ × S8
```

**Post-calculs (7 additions/soustractions) :**

```
T1 = P1 + P2              C₁₁ = P2 + P3
T2 = T1 + P4              C₁₂ = T1 + P5 + P6
                            C₂₁ = T2 - P7
                            C₂₂ = T2 + P5
```

### 2.4.3 Comparaison des coûts

| Méthode           | Multiplications   | Additions | Seuil (bits)              |
| ------------------ | ----------------- | --------- | ------------------------- |
| Naïf 2×2         | 8                 | 4         | ≤ strassenThreshold      |
| Strassen-Winograd  | 7                 | 15        | > strassenThreshold       |
| Carré symétrique | 4 (3 sqr + 1 mul) | 4         | Toujours (si symétrique) |

### 2.4.4 Seuil par défaut et configuration

```
DefaultStrassenThreshold = 3072 bits  (constants.go)
defaultStrassenThresholdBits = 256 bits (atomic, matrix_ops.go — défaut init())
```

**Note** : Il y a deux seuils. `DefaultStrassenThreshold` (3072) est utilisé via `normalizeOptions()` pour `opts.StrassenThreshold`. Le `defaultStrassenThresholdBits` (256) est un fallback atomique dans `multiplyMatrices()` quand le seuil est 0.

### 2.4.5 Notes de traduction Rust

```rust
// Les 7 multiplications parallèles se traduisent par rayon::scope ou
// std::thread::scope pour le fork-join borné.
// Le pattern executeTasks[T, PT] avec generics et pointer constraint
// se traduit par un trait Task + impl pour MulTask/SqrTask.
```

---

## T2.5 — Sélection des paramètres FFT (k, n, modulus de Fermat)

### 2.5.1 Algorithme de sélection `fftSize`

La fonction `fftSize(x, y)` détermine les paramètres FFT optimaux :

```
FONCTION fftSize(x, y: nat) → (k: uint, m: int):
    words ← len(x) + len(y)
    bits ← words × _W                    // _W = 64 sur architectures 64 bits

    // Sélection de K = 2^k via la table fftSizeThreshold
    k ← len(fftSizeThreshold)            // Valeur maximale par défaut
    POUR i DE 0 À len(fftSizeThreshold)-1:
        SI fftSizeThreshold[i] > bits:
            k ← i
            SORTIR

    // Taille des chunks : m mots par coefficient polynomial
    // Contrainte : (m << k) > words, i.e. K×m > words
    m ← words >> k + 1

    RETOURNER (k, m)
```

### 2.5.2 Table des seuils FFT

```
fftSizeThreshold = [
    0, 0, 0,                           // k=0,1,2 : inutilisé
    4 KiB,   8 KiB,   16 KiB,          // k=3,4,5
    32 KiB,  64 KiB,  256 KiB,         // k=6,7,8
    1 MiB,   3 MiB,                     // k=9,10
    8 MiB,   30 MiB,  100 MiB,         // k=11,12,13
    300 MiB, 600 MiB                    // k=14,15
]
```

**Heuristique** : K ≈ 2·√N où N est le nombre total de bits du résultat.

### 2.5.3 Calcul de `valueSize` (taille des coefficients)

```
FONCTION valueSize(k: uint, m: int, extra: uint) → int:
    // Les coefficients de P×Q sont < b^(2m) × K
    // Donc on a besoin de : W × valueSize ≥ 2×m×W + K
    n ← 2 × m × _W + k                 // Bits nécessaires
    K' ← max(1 << (k - extra), _W)     // Granularité d'alignement
    n ← ((n / K') + 1) × K'            // Arrondi au multiple supérieur
    RETOURNER n / _W                    // Conversion en mots
```

### 2.5.4 Cas particulier : carré (`fftSizeSqr`)

Pour le carré, la taille résultante est `2 × len(x)` au lieu de `len(x) + len(y)` :

```
FONCTION fftSizeSqr(x: nat) → (k: uint, m: int):
    words ← 2 × len(x)
    // Même algorithme que fftSize, mais avec words = 2×len(x)
    ...
```

### 2.5.5 Modulus de Fermat

L'arithmétique FFT opère modulo `2^(n×W) + 1` (nombre de Fermat généralisé), où :

- `n` = `valueSize(k, m, extra)` en mots
- Le modulus M = 2^(n×64) + 1 sur une architecture 64 bits

La racine K-ième primitive de l'unité θ est une puissance de 2, ce qui permet de remplacer les multiplications par des décalages (shifts).

### 2.5.6 Notes de traduction Rust

```rust
// La table fftSizeThreshold est un const [i64; 16].
// La sélection par itération linéaire est acceptable car len=16.
// _W sera cfg-dépendant : #[cfg(target_pointer_width = "64")] const W: usize = 64;
```

---

## T2.6 — Arithmétique de Fermat (Shift, Mul, Sqr, normalisation)

### 2.6.1 Représentation

Un nombre de Fermat `fermat` représente un élément de Z/(2^(n×W)+1) par un slice de `n+1` mots. Le dernier mot est 0 ou 1, correspondant aux deux représentants possibles satisfaisant cette contrainte.

```
type fermat = [Word; n+1]  // n mots de données + 1 mot de débordement (0 ou 1)
```

### 2.6.2 Normalisation (`norm`)

La normalisation ramène le dernier mot à {0, 1} :

```
FONCTION norm(z: fermat):
    n ← len(z) - 1
    c ← z[n]
    SI c == 0: RETOUR

    // Cas simple : z[0] >= c → soustraire de z[0]
    SI z[0] >= c:
        z[n] ← 0
        z[0] -= c
        RETOUR

    // Cas général : propagation de l'emprunt
    subVW(z, z, c)
    SI c > 1:
        z[n] -= c - 1
        c ← 1
    SI z[n] == 1:
        z[n] ← 0
        RETOUR
    addVW(z, z, 1)
```

### 2.6.3 Shift : `(x << k) mod (2^(n×W)+1)`

Le décalage modulo un nombre de Fermat est le cœur de l'efficacité FFT car il remplace les multiplications par des opérations de décalage.

**Propriété clé** : Décaler de n×W bits équivaut à prendre l'opposé modulo 2^(n×W)+1.

```
FONCTION Shift(z, x: fermat, k: int):
    n ← len(x) - 1
    k ← k mod (2 × n × W)
    SI k < 0: k += 2 × n × W

    neg ← FAUX
    SI k ≥ n × W:
        k -= n × W
        neg ← VRAI

    kw, kb ← k / W, k % W     // Mots entiers + bits restants

    SI non neg:
        // x = a·2^(n-k)W + b  →  z = (b << k) - a
        copy(z[kw:], x[:n-kw])
        b ← subVV(z[:kw+1], z[:kw+1], x[n-kw:])
        propager_emprunt(z, kw+1, b)
    SINON:
        // Négatif + décalage
        copy(z[:kw+1], x[n-kw:n+1])
        b ← subVV(z[kw:n], z[kw:n], x[:n-kw])
        z[n] -= b

    // Ajuster +1 (compensation de la soustraction de 1 ajoutée au début)
    ajuster_carry(z)

    // Décalage final de kb bits
    shlVU(z, z, kb)
    z.norm()
```

### 2.6.4 Multiplication de Fermat (`Mul`)

```
FONCTION Mul(z, x, y: fermat) → fermat:
    n ← len(x) - 1

    SI n < smallMulThreshold (30):
        // Multiplication naïve O(n²)
        basicMul(z, x, y)
    SINON:
        // Déléguer à math/big.Mul via big.Int
        z ← big.Int(x).Mul(big.Int(y))

    // Réduction modulo 2^(n×W)+1 :
    // z = z[:n] - z[n:2n] + z[2n]
    c1 ← addVW(z[:n], z[:n], z[2n])     // + z[2n]
    c2 ← subVV(z[:n], z[:n], z[n:2n])   // - z[n:2n]
    z[n] ← c1
    addVW(z, z, c2)                       // + carry de soustraction
    z.norm()
    RETOURNER z
```

### 2.6.5 Carré de Fermat (`Sqr`)

L'implémentation `basicSqr` exploite la symétrie x[i]×x[j] = x[j]×x[i] pour économiser ~50% des produits partiels :

```
FONCTION basicSqr(z, x: fermat):
    n ← len(x)
    clear(z[:2n])

    // Termes hors-diagonale : x[i] × x[j] pour j > i
    POUR i DE 0 À n-2:
        SI x[i] ≠ 0:
            z[i+n] ← addMulVVW(z[2i+1:i+n], x[i+1:n], x[i])

    // Doubler les termes hors-diagonale (décalage gauche de 1 bit)
    shlVU(z[:2n], z[:2n], 1)

    // Ajouter les termes diagonaux : x[i]²
    POUR i DE 0 À n-1:
        SI x[i] ≠ 0:
            hi, lo ← Mul(x[i], x[i])  // Produit 1-mot × 1-mot
            ajouter hi:lo à z[2i:2n]
```

### 2.6.6 ShiftHalf : multiplication par √2

Le demi-décalage `ShiftHalf(x, k)` implémente la multiplication par 2^(k/2) :

```
√2 mod (2^(n×W)+1) = 2^(3n×W/4) - 2^(n×W/4)
```

Donc ShiftHalf(x, k) pour k impair = Shift(x, a) - Shift(x, b) avec a = (k-1)/2 + 3nW/4, b = (k-1)/2 + nW/4.

### 2.6.7 Notes de traduction Rust

```rust
// fermat se traduit par Vec<u64> ou [u64; N] si taille connue.
// Les opérations vectorielles (addVV, subVV, addMulVVW, shlVU) seront
// implémentées via :
// 1. std::arch intrinsics (ADX, MULX sur x86_64)
// 2. Crate `crypto-bigint` pour les opérations portables
// 3. Fallback Rust pur avec checked/wrapping arithmetic
```

---

## T2.7 — FFT : structure récursive et cas de base

### 2.7.1 Arbre de récursion

La FFT est une transformation de Cooley-Tukey récursive opérant sur des vecteurs de nombres de Fermat :

```
fourier(K=2^k éléments) :
    ├── fourier(K/2 éléments pairs)    // src[0], src[2], src[4], ...
    ├── fourier(K/2 éléments impairs)  // src[1], src[3], src[5], ...
    └── Reconstruction (papillon)       // K opérations ShiftHalf + Add/Sub
```

### 2.7.2 Cas de base

```
FONCTION fourierRecursive(dst, src, backward, n, k, size, depth, tmp, tmp2):
    // Cas de base 0 : copie directe
    SI size == 0:
        copy(dst[0], src[0])
        RETOUR

    // Cas de base 1 : papillon simple (2 éléments)
    SI size == 1:
        dst[0] ← src[0] + src[stride]    // stride = 1 << (k - size)
        dst[1] ← src[0] - src[stride]
        RETOUR

    // Cas récursif...
```

### 2.7.3 Opérations papillon (Butterfly)

La phase de reconstruction combine les demi-transformées :

```
POUR i DE 0 À K/2 - 1:
    tmp ← ShiftHalf(dst2[i], i × ω2shift, tmp2)
    dst2[i] ← dst1[i] - tmp              // Branche "basse"
    dst1[i] ← dst1[i] + tmp              // Branche "haute"
```

Où `ω2shift = (4 × n × W) >> size` est le pas de rotation (puissance de θ).

### 2.7.4 Parallélisation de la récursion FFT

```
SI size ≥ ParallelFFTRecursionThreshold (4) ET depth < MaxParallelFFTDepth (3):
    ESSAYER acquérir token du sémaphore (non-bloquant):
        SUCCÈS:
            // Demi-transformée impaire en parallèle
            LANCER goroutine avec nouveaux tampons tmp
            // Demi-transformée paire dans le thread courant
            fourierRecursive(dst1, src_pairs, ...)
            ATTENDRE goroutine
        ÉCHEC:
            // Fallback séquentiel
            fourierRecursive(dst1, src_pairs, ...)
            fourierRecursive(dst2, src_impairs, ...)
```

### 2.7.5 Diagramme de l'arbre de récursion (k=4, K=16)

```
                    fourier(size=4, K=16)
                   /                     \
          fourier(size=3, K=8)     fourier(size=3, K=8)
         /          \              /          \
    f(size=2)  f(size=2)      f(size=2)  f(size=2)
    /   \       /   \          /   \       /   \
  f(1) f(1)  f(1) f(1)     f(1) f(1)  f(1) f(1)
```

Chaque nœud interne effectue K/2 opérations papillon sur sa taille.

### 2.7.6 Notes de traduction Rust

```rust
// La récursion FFT se traduit directement en Rust.
// Pour la parallélisation, utiliser rayon::scope ou std::thread::scope
// avec un Arc<Semaphore> pour le contrôle de concurrence.
// Les tampons temporaires : Vec<u64> alloués par l'allocateur bump
// ou depuis un pool thread-local.
```

---

## T2.8 — Opérations polynomiales (Poly, PolValues, transformées)

### 2.8.1 Structure `Poly`

Un polynôme `Poly` représente un entier N via ses coefficients dans la base b^m :

```
Poly {
    K: uint     // log2 de la longueur FFT (K éléments = 2^K)
    M: int      // Taille en mots de chaque coefficient
    A: Vec<nat> // Jusqu'à 2^K coefficients de M mots chacun
}
```

**Conversion entier → polynôme** (`polyFromNat`) :

```
FONCTION polyFromNat(x: nat, k: uint, m: int) → Poly:
    length ← ceil(len(x) / m)
    A ← Vec de length tranches de m mots
    POUR i DE 0 À length-1:
        SI len(x) < m:
            A[i] ← copier x, compléter par des zéros
            SORTIR
        A[i] ← x[:m]
        x ← x[m:]
    RETOURNER Poly{K: k, M: m, A: A}
```

**Conversion polynôme → entier** (`IntTo`) :

```
FONCTION IntTo(poly: Poly, dst: nat) → nat:
    // Reconstituer N = Σ A[i] × b^(i×M) avec propagation de retenue
    n ← allouer ou réutiliser dst
    POUR i, coeff DANS poly.A:
        addVV(n[i×M:], n[i×M:], coeff)
        propager_retenue(n, i×M + len(coeff))
    RETOURNER trim(n)
```

### 2.8.2 Structure `PolValues`

Les valeurs évaluées d'un polynôme aux racines de l'unité :

```
PolValues {
    K:      uint        // log2 de la longueur FFT
    N:      int         // Longueur des coefficients en mots
    Values: Vec<fermat> // 2^K valeurs de Fermat de (N+1) mots
}
```

### 2.8.3 Transformée directe (`Transform`)

```
FONCTION Transform(poly: Poly, n: int) → PolValues:
    K ← 1 << poly.K

    // Préparer vecteur d'entrée
    input ← allouer K éléments de taille (n+1)
    POUR i DE 0 À K-1:
        SI i < len(poly.A): copy(input[i], poly.A[i])

    // Allouer vecteur de sortie
    values ← allouer K éléments de taille (n+1)

    // Exécuter FFT directe
    fourier(values, input, backward=FAUX, n, poly.K)

    RETOURNER PolValues{K: poly.K, N: n, Values: values}
```

### 2.8.4 Transformée inverse (`InvTransform`)

```
FONCTION InvTransform(v: PolValues) → Poly:
    K ← 1 << v.K

    // FFT inverse
    p ← allouer K éléments de taille (v.N+1)
    fourier(p, v.Values, backward=VRAI, v.N, v.K)

    // Diviser par K (décalage de -k bits en Fermat)
    // et reconvertir en coefficients
    a ← Vec de K nat
    POUR i DE 0 À K-1:
        u.Shift(p[i], -k)
        copy(p[i], u)
        a[i] ← p[i]

    RETOURNER Poly{K: v.K, M: 0, A: a}
```

### 2.8.5 Multiplication point-à-point (`Mul` sur PolValues)

```
FONCTION PolValues.Mul(p, q: PolValues) → PolValues:
    // Allouer tampon temporaire pour multiplication de Fermat
    buf ← allouer fermat(8 × p.N)

    r.Values ← allouer K éléments
    POUR i DE 0 À K-1:
        r.Values[i] ← buf.Mul(p.Values[i], q.Values[i])

    RETOURNER r
```

### 2.8.6 Carré point-à-point (`Sqr` sur PolValues)

```
FONCTION PolValues.Sqr(p: PolValues) → PolValues:
    buf ← allouer fermat(8 × p.N)

    r.Values ← allouer K éléments
    POUR i DE 0 À K-1:
        r.Values[i] ← buf.Sqr(p.Values[i])

    RETOURNER r
```

### 2.8.7 Notes de traduction Rust

```rust
// Poly et PolValues se traduisent directement en structs Rust.
// Les allocations contiguës de fermat slices (acquireWordSliceUnsafe)
// se traduisent par un Vec<u64> unique découpé en sous-tranches.
// Attention à la lifetime: les sous-tranches doivent emprunter le Vec parent.
// Solution : utiliser un arena allocator (bumpalo crate) ou indices.
```

---

## T2.9 — Optimisation de réutilisation de transformée FFT pour le carré

### 2.9.1 Principe

Pour calculer les 3 produits du doublement (FK×FK1, FK1², FK²), l'optimisation consiste à :

1. **Transformer FK et FK1 une seule fois** (2 FFT directes)
2. **Calculer les 3 produits** dans le domaine fréquentiel (multiplication/carré point-à-point)
3. **3 FFT inverses** pour reconvertir

**Économie** : 3 transformées directes économisées (on en fait 2 au lieu de 5 pour 3 produits).

### 2.9.2 Pseudocode de `executeDoublingStepFFT`

```
FONCTION executeDoublingStepFFT(ctx, state, opts, inParallel):
    // 1. Déterminer les paramètres FFT
    fk1Words ← len(state.FK1.Bits())
    targetWords ← 2 × fk1Words + 2
    k, m ← GetFFTParams(targetWords)
    nWords ← ValueSize(k, m, 2)

    // 2. Transformer les opérandes UNE SEULE FOIS
    pFk  ← PolyFromInt(state.FK, k, m)
    fkPoly  ← pFk.Transform(nWords)        // FFT directe de FK

    pFk1 ← PolyFromInt(state.FK1, k, m)
    fk1Poly ← pFk1.Transform(nWords)       // FFT directe de FK1

    // 3. Calculer les 3 produits (parallèle ou séquentiel)
    SI inParallel:
        LANCER 3 goroutines:
            goroutine 1: T3 ← InvFFT(fkPoly × fk1Poly)   // FK × FK1
            goroutine 2: T1 ← InvFFT(fk1Poly²)            // FK1²
            goroutine 3: T2 ← InvFFT(fkPoly²)             // FK²
        ATTENDRE toutes

    SINON:
        T3 ← fkPoly.Mul(fk1Poly) → InvTransform → IntToBigInt(state.T3)
        T1 ← fk1Poly.Sqr()       → InvTransform → IntToBigInt(state.T1)
        T2 ← fkPoly.Sqr()        → InvTransform → IntToBigInt(state.T2)
```

### 2.9.3 Sécurité en accès concurrent

Les méthodes `PolValues.Mul()` et `PolValues.Sqr()` sont **en lecture seule** sur le récepteur — elles lisent `p.Values[i]` sans le modifier. Le tampon `buf` est local à chaque goroutine. Cela permet de partager `fkPoly` et `fk1Poly` entre 3 goroutines sans clonage.

### 2.9.4 Comparaison des coûts FFT

| Approche                     | Transformées directes | Produits point-à-point | Transformées inverses | Total FFT      |
| ---------------------------- | ---------------------- | ----------------------- | ---------------------- | -------------- |
| Naïve (3 Mul séparés)     | 6                      | 3                       | 3                      | 12             |
| Avec réutilisation (carré) | 2                      | 3                       | 3                      | 8              |
| **Économie**          | **-4**           | 0                       | 0                      | **-33%** |

### 2.9.5 Notes de traduction Rust

```rust
// En Rust, les PolValues partagés entre threads nécessitent Arc<PolValues>
// ou des références empruntées via rayon::scope / std::thread::scope.
// Puisque Mul/Sqr sont read-only, &PolValues suffit (pas besoin de Mutex).
```

---

## T2.10 — Sélection adaptative de stratégie de multiplication

### 2.10.1 Les trois stratégies

| Stratégie | Classe Go             | Comportement                    | Cas d'utilisation                |
| ---------- | --------------------- | ------------------------------- | -------------------------------- |
| Adaptive   | `AdaptiveStrategy`  | math/big si petit, FFT si grand | Production (défaut)             |
| FFT-Only   | `FFTOnlyStrategy`   | FFT pour toutes les opérations | Benchmark / très grands nombres |
| Karatsuba  | `KaratsubaStrategy` | math/big.Mul toujours           | Tests / comparaison              |

### 2.10.2 Organigramme de sélection (AdaptiveStrategy)

```
┌─────────────────────────────────┐
│ AdaptiveStrategy.ExecuteStep()  │
└──────────────┬──────────────────┘
               │
               ▼
    ┌──────────────────────┐
    │ FK1.BitLen() >       │
    │ FFTThreshold ?       │
    └──┬───────────────┬───┘
   OUI │               │ NON
       ▼               ▼
┌──────────────┐  ┌──────────────────────────────────┐
│ executeDoubl-│  │ executeDoublingStepMultiplications│
│ ingStepFFT() │  │ (via smartMultiply/smartSquare)   │
│ (réutil. FFT)│  └──────────────────┬───────────────┘
└──────────────┘                     │
                                     ▼
                          ┌──────────────────────┐
                          │ smartMultiply(z,x,y): │
                          │                       │
                          │ SI bx > FFTThreshold  │
                          │ ET by > FFTThreshold:  │
                          │   → bigfft.MulTo()    │
                          │ SINON:                 │
                          │   → z.Mul(x, y)       │
                          └───────────────────────┘
```

### 2.10.3 Sélection par le DoublingFramework

```
POUR chaque bit i de n (MSB→LSB):
    fkBitLen  ← FK.BitLen()
    fk1BitLen ← FK1.BitLen()
    bitLen ← fkBitLen

    // Déterminer si FFT est utilisé
    usedFFT ← bitLen > opts.FFTThreshold

    // Déterminer si parallélisation est bénéfique
    shouldParallel ← useParallel ET
                     shouldParallelizeMultiplicationCached(opts, fkBitLen, fk1BitLen)

    // Déléguer à la stratégie
    strategy.ExecuteStep(ctx, state, opts, shouldParallel)
```

### 2.10.4 Tiers de multiplication dans `smartMultiply`

```
Tier 1 : FFT         — bx > FFTThreshold ET by > FFTThreshold → bigfft.MulTo()
Tier 2 : Karatsuba   — sinon → math/big.Mul() (qui utilise Karatsuba en interne)
```

Pour `smartSquare` :

```
Tier 1 : FFT         — bx > FFTThreshold → bigfft.SqrTo()
Tier 2 : Karatsuba   — sinon → math/big.Mul(x, x)
```

### 2.10.5 Notes de traduction Rust

```rust
// Les stratégies se traduisent par un trait Multiplier + DoublingStepExecutor.
// L'enum dispatch est plus idiomatique en Rust :
enum Strategy {
    Adaptive { fft_threshold: usize },
    FftOnly,
    Karatsuba,
}

impl DoublingStepExecutor for Strategy {
    fn execute_step(&self, ...) -> Result<(), Error> {
        match self {
            Strategy::Adaptive { fft_threshold } => { ... }
            Strategy::FftOnly => { ... }
            Strategy::Karatsuba => { ... }
        }
    }
}
```

---

## T2.11 — Fast Doubling Modulaire (`--last-digits`)

### 2.11.1 Algorithme

`FastDoublingMod(n, m)` calcule F(n) mod m en utilisant les mêmes identités de doublement, mais avec réduction modulaire à chaque étape :

```
FONCTION FastDoublingMod(n: u64, m: BigInt) → BigInt:
    SI m ≤ 0: ERREUR("modulus doit être positif")
    SI n == 0: RETOURNER 0

    fk  ← 0         // F(k)
    fk1 ← 1         // F(k+1)
    t1, t2 ← temporaires

    POUR i DE (numBits-1) VERS 0:
        // F(2k) = F(k) × (2·F(k+1) - F(k)) mod m
        t1 ← (2 × fk1 - fk) mod m
        SI t1 < 0: t1 += m              // Gérer le mod négatif
        t1 ← (t1 × fk) mod m

        // F(2k+1) = F(k+1)² + F(k)² mod m
        t2 ← (fk1² + fk²) mod m

        fk ← t1
        fk1 ← t2

        // Étape d'addition conditionnelle
        SI bit(n, i) == 1:
            t1 ← (fk + fk1) mod m
            fk ← fk1
            fk1 ← t1

    RETOURNER fk
```

### 2.11.2 Preuve de mémoire O(K)

Soit K = log₁₀(m) le nombre de chiffres du modulus.

- **Variables** : `fk`, `fk1`, `t1`, `t2` — 4 BigInt
- **Taille de chaque variable** : Au plus O(K) chiffres (car toujours réduit mod m)
- **Mémoire totale** : 4 × O(K) = O(K) bits, **indépendant de n**

Pour `--last-digits K`, le modulus est m = 10^K, donc la mémoire est O(K) bits.

### 2.11.3 Comparaison avec le calcul complet

| Aspect                | F(n) complet     | F(n) mod m       |
| --------------------- | ---------------- | ---------------- |
| Mémoire              | O(n) bits        | O(K) bits        |
| Temps                 | O(log n × M(n)) | O(log n × M(K)) |
| Cas F(10^18) mod 10^6 | Impossible       | < 1 seconde      |

### 2.11.4 Gestion du mod négatif

Le résultat de `(2·fk1 - fk) mod m` peut être négatif si fk > 2·fk1. L'ajout conditionnel de m garantit un résultat dans [0, m).

### 2.11.5 Notes de traduction Rust

```rust
// En Rust, num-bigint::BigUint n'a pas de nombres négatifs.
// Utiliser checked_sub() ou travailler avec BigInt signé puis convertir.
// Alternative : calculer (2*fk1 + m - fk) % m pour éviter les négatifs.
```

---

## T2.12 — Fast path itératif (n ≤ 93)

### 2.12.1 Justification du seuil

F(93) = 12 200 160 415 121 876 738 est le plus grand nombre de Fibonacci tenant dans un `u64` (< 2^64). F(94) = 19 740 274 219 868 223 167 > 2^64.

### 2.12.2 Algorithme itératif

```
FONCTION calculateSmall(n: u64) → BigInt:
    SI n == 0: RETOURNER BigInt(0)
    SI n == 1: RETOURNER BigInt(1)

    a ← BigInt(0)
    b ← BigInt(1)
    POUR i DE 2 À n:
        a ← a + b
        (a, b) ← (b, a)    // Échange de pointeurs

    RETOURNER b
```

### 2.12.3 Prévention de débordement

Bien que le code Go utilise `big.Int` même pour le fast path (pour uniformité d'interface), en Rust on peut optimiser avec `u64` pour n ≤ 93 :

```rust
fn calculate_small_u64(n: u64) -> u64 {
    if n == 0 { return 0; }
    if n == 1 { return 1; }
    let (mut a, mut b) = (0u64, 1u64);
    for _ in 2..=n {
        let temp = a.wrapping_add(b); // Pas de débordement car n ≤ 93
        a = b;
        b = temp;
    }
    b
}
```

**Preuve d'absence de débordement** : F(93) < 2^64, et chaque F(i) pour i ≤ 93 est strictement inférieur à 2^64. L'addition a + b ne déborde jamais car a = F(i-2) et b = F(i-1), et leur somme F(i) ≤ F(93) < 2^64.

### 2.12.4 Intégration dans le décorateur

```
FONCTION FibCalculator.Calculate(n, opts):
    SI n ≤ MaxFibUint64 (93):
        reporter(1.0)           // Progrès complet immédiat
        RETOURNER calculateSmall(n)

    // Sinon : déléguer au coreCalculator (Fast Doubling, Matrix, FFT)
    RETOURNER core.CalculateCore(n, opts)
```

### 2.12.5 Notes de traduction Rust

```rust
// Optimisation Rust : utiliser u128 pour étendre à F(186) avant BigUint.
// F(186) = 332 825 110 087 067 562 321 196 029 789 634 457 848
// qui tient dans un u128 (< 2^128).
const MAX_FIB_U64: u64 = 93;
const MAX_FIB_U128: u64 = 186;  // Extension possible en Rust
```

---

## T2.13 — Retour résultat zéro-copie

### 2.13.1 Problème

Le résultat F(n) est stocké dans `state.FK` (Fast Doubling) ou `state.res.a` (Matrix). Copier ce BigInt serait O(n) en nombre de mots (ex: ~109K mots = 850 KB pour F(10M)).

### 2.13.2 Solution : vol de pointeur (pointer stealing)

```go
// Go — Fast Doubling
result := s.FK           // Voler le pointeur
s.FK = new(big.Int)      // Remplacer par un BigInt vide (24 bytes)
return result, nil

// Go — Matrix Exponentiation
result := state.res.a    // Voler l'élément [0,0]
state.res.a = new(big.Int)
return result, nil
```

### 2.13.3 Invariant de sécurité

L'état (`CalculationState` ou `matrixState`) doit rester valide pour être remis dans le pool. En remplaçant le pointeur volé par un `new(big.Int)`, l'état reste cohérent pour `ReleaseState()`/`releaseMatrixState()`.

### 2.13.4 Coût

| Opération          | Coût                                          |
| ------------------- | ---------------------------------------------- |
| Copie O(n)          | ~850 KB pour F(10M), ~8.5 MB pour F(100M)      |
| Pointer stealing    | 24 bytes (allocation d'un header big.Int vide) |
| **Économie** | ~99.997% de la copie éliminée                |

### 2.13.5 Mapping Rust : `std::mem::replace`

```rust
// Rust — équivalent exact
fn execute_doubling_loop(&mut self, state: &mut CalculationState) -> BigUint {
    // Voler le résultat, remplacer par une valeur par défaut
    let result = std::mem::replace(&mut state.fk, BigUint::ZERO);
    result
}

// Ou avec std::mem::take (raccourci pour replace par Default)
let result = std::mem::take(&mut state.fk);  // fk ← BigUint::default() = 0
```

**Avantage Rust** : `std::mem::replace` est une opération O(1) et ne nécessite aucune allocation. C'est exactement le pattern de pointer stealing de Go, mais formalisé et type-safe.

### 2.13.6 Interaction avec le pool

```
SÉQUENCE COMPLÈTE:

1. state ← AcquireState()           // Obtenir du pool
2. result ← ExecuteDoublingLoop()    // Calculer
3. result ← mem::take(&state.fk)    // Voler le résultat
4. ReleaseState(state)              // Remettre dans le pool (avec fk = 0)
```

---

## T2.14 — Méthodologie de comparaison inter-algorithmes

### 2.14.1 Flux d'exécution concurrente

```
FONCTION ExecuteCalculations(calculators, cfg):
    results ← Vec de taille len(calculators)
    progressChan ← canal bufferisé (len(calculators) × 50)

    // Lancer le rapporteur de progression
    LANCER goroutine: DisplayProgress(progressChan, len(calculators))

    // Cas rapide : un seul calculateur
    SI len(calculators) == 1:
        results[0] ← calculators[0].Calculate(cfg.N, opts)
    SINON:
        // errgroup pour exécution concurrente
        g ← errgroup.WithContext(ctx)
        POUR i, calc DANS calculators:
            g.Go(func: results[i] ← calc.Calculate(cfg.N, opts))
        g.Wait()

    close(progressChan)
    ATTENDRE DisplayProgress terminé
    RETOURNER results
```

### 2.14.2 Détection de divergence

```
FONCTION AnalyzeComparisonResults(results):
    // 1. Trier par durée (succès avant erreurs)
    sort(results, par: succès_d'abord, puis par durée)

    // 2. Identifier le premier résultat valide
    firstValid ← premier résultat sans erreur

    // 3. Comparer bit-à-bit
    mismatch ← FAUX
    POUR chaque result DANS results:
        SI result.Err == nil ET result.Result ≠ firstValid.Result:
            mismatch ← VRAI
            SORTIR

    // 4. Rapporter
    SI mismatch:
        RETOURNER ExitErrorMismatch (3)
    SINON:
        RETOURNER ExitSuccess (0)
```

### 2.14.3 Comparaison bit-à-bit

La comparaison utilise `big.Int.Cmp()` qui effectue une comparaison lexicographique sur la représentation interne (slice de mots). En Rust, `BigUint::eq()` effectue la même opération.

### 2.14.4 Tri par vitesse

```
RÈGLE DE TRI:
    1. Succès avant erreurs
    2. Parmi les succès : tri croissant par durée
    3. Parmi les erreurs : ordre original
```

### 2.14.5 Notes de traduction Rust

```rust
// errgroup se traduit par tokio::task::JoinSet ou rayon::scope
// Le tri se fait avec sort_by(|a, b| ...)
// La comparaison big int utilise PartialEq sur num_bigint::BigUint
```

---

## T2.15 — Générateur de séquence et optimisation Skip

### 2.15.1 Interface `SequenceGenerator`

```
TRAIT SequenceGenerator:
    Next(ctx) → BigInt        // Avancer d'un pas, retourner F(index)
    Current() → BigInt        // Valeur courante sans avancer
    Index() → u64             // Index courant
    Reset()                   // Revenir à F(0)
    Skip(ctx, n) → BigInt     // Sauter à F(n) efficacement
```

### 2.15.2 Implémentation itérative

```
IterativeGenerator {
    current: BigInt    // F(index)
    next:    BigInt    // F(index+1)
    index:   u64
    started: bool
    calculator: Option<Calculator>  // Initialisé paresseusement pour Skip
}

FONCTION Next(ctx):
    SI non started:
        started ← VRAI
        RETOURNER copie de current    // F(0) = 0

    index += 1
    (current, next) ← (next, current + next)
    RETOURNER copie de current
```

### 2.15.3 Optimisation Skip

Le Skip utilise un seuil pour décider entre itération et calcul direct :

```
FONCTION Skip(ctx, n):
    currentIdx ← index courant

    // Cas 1 : saut court → itérer
    SI n >= currentIdx ET n - currentIdx < iterativeThreshold (1000):
        PENDANT index < n:
            Next(ctx)
        RETOURNER Current()

    // Cas 2 : saut long → utiliser Calculator O(log n)
    SI calculator == nil:
        calculator ← GlobalFactory().Get("fast")

    result ← calculator.Calculate(n)
    nextResult ← calculator.Calculate(n + 1)

    // Mettre à jour l'état pour permettre des Next() ultérieurs
    current ← result
    next ← nextResult
    index ← n
    RETOURNER copie de current
```

### 2.15.4 Seuil itératif vs Calculator

| Saut (distance) | Méthode                   | Complexité                                 |
| --------------- | -------------------------- | ------------------------------------------- |
| < 1000          | Itération                 | O(distance × d) où d = nombre de chiffres |
| ≥ 1000         | Calculator (Fast Doubling) | O(log n × M(n))                            |

### 2.15.5 Notes de traduction Rust

```rust
// L'initialisation paresseuse du Calculator se traduit par
// OnceCell<Box<dyn Calculator>> ou lazy_static!
// La protection par Mutex (mu sync.Mutex) peut être remplacée par
// un design non-thread-safe (pas de Sync/Send) ou par un RwLock.
```

---

## T2.16 — Sélection de calculateur depuis la configuration

### 2.16.1 Algorithme de sélection

```
FONCTION GetCalculatorsToRun(cfg, factory) → Vec<Calculator>:
    SI cfg.Algo == "all":
        keys ← factory.List()    // Triés alphabétiquement
        calculators ← Vec vide
        POUR chaque key DANS keys:
            SI calc ← factory.Get(key) sans erreur:
                calculators.push(calc)
        RETOURNER calculators

    // Sélection unique
    SI calc ← factory.Get(cfg.Algo) sans erreur:
        RETOURNER [calc]

    RETOURNER nil    // Algorithme inconnu
```

### 2.16.2 Algorithmes enregistrés par défaut

| Clé         | Classe                    | Description                               |
| ------------ | ------------------------- | ----------------------------------------- |
| `"fast"`   | `OptimizedFastDoubling` | Fast Doubling adaptatif (Karatsuba + FFT) |
| `"matrix"` | `MatrixExponentiation`  | Exponentiation matricielle (Strassen)     |
| `"fft"`    | `FFTBasedCalculator`    | Fast Doubling FFT-only                    |
| `"gmp"`    | `GMPCalculator`         | GMP (optionnel, build tag `gmp`)        |

### 2.16.3 Parsing des valeurs valides

```
Valeurs acceptées pour --algo :
  "all"    → Exécuter tous les algorithmes enregistrés, comparer les résultats
  "fast"   → Fast Doubling adaptatif (défaut)
  "matrix" → Exponentiation Matricielle
  "fft"    → Fast Doubling FFT-only
  "gmp"    → GMP (si compilé avec -tags=gmp)
```

### 2.16.4 Pattern Factory avec cache

```
DefaultFactory {
    creators:    HashMap<String, fn() → coreCalculator>  // Fonctions de création
    calculators: HashMap<String, Calculator>              // Cache d'instances
}

FONCTION Get(name):
    // Double-checked locking
    SI name DANS calculators (lecture):
        RETOURNER calculators[name]

    // Créer et cacher (écriture)
    creator ← creators[name]
    calc ← NewCalculator(creator())
    calculators[name] ← calc
    RETOURNER calc
```

### 2.16.5 Notes de traduction Rust

```rust
// Le pattern Factory se traduit par :
// - HashMap<&str, fn() -> Box<dyn Calculator>> pour les créateurs
// - HashMap<&str, Arc<dyn Calculator>> pour le cache
// - RwLock pour la thread-safety (ou DashMap pour lock-free)
// L'enregistrement auto via init() (GMP) se traduit par inventory/linkme crate
// ou #[ctor] pour les constructeurs statiques.
```

---

## T2.17 — Preuves de correction algorithmique

### 2.17.1 Identité 1 : Fast Doubling

**Énoncé** :

```
F(2k)   = F(k) × [2·F(k+1) - F(k)]
F(2k+1) = F(k+1)² + F(k)²
```

**Esquisse de preuve** :

1. Soit Q = [[1,1],[1,0]]. Par induction, Q^n = [[F(n+1), F(n)], [F(n), F(n-1)]].
2. Q^(2k) = (Q^k)². En multipliant Q^k par lui-même :

```
[F(k+1) F(k)]²   [F(k+1)²+F(k)²    F(k+1)F(k)+F(k)F(k-1)]
[F(k)   F(k-1)] = [F(k)F(k+1)+F(k-1)F(k)  F(k)²+F(k-1)²  ]
```

3. Élément [0,0] : F(2k+1) = F(k+1)² + F(k)² ✓
4. Élément [0,1] : F(2k) = F(k)(F(k+1) + F(k-1)) = F(k)(2F(k+1) - F(k)) ✓
   (en utilisant F(k-1) = F(k+1) - F(k))

### 2.17.2 Identité 2 : Exponentiation matricielle

**Énoncé** : Q^n = [[F(n+1), F(n)], [F(n), F(n-1)]] pour tout n ≥ 1.

**Preuve par induction** :

- **Base** : Q^1 = [[1,1],[1,0]] = [[F(2), F(1)], [F(1), F(0)]] ✓
- **Pas inductif** : Si Q^k = [[F(k+1), F(k)], [F(k), F(k-1)]], alors :

```
Q^(k+1) = Q^k × Q = [[F(k+1)+F(k), F(k+1)], [F(k)+F(k-1), F(k)]]
         = [[F(k+2), F(k+1)], [F(k+1), F(k)]] ✓
```

### 2.17.3 Identité 3 : Strassen-Winograd

**Énoncé** : Pour des matrices 2×2, Strassen-Winograd calcule C = A×B avec 7 multiplications.

**Vérification** : On peut vérifier algébriquement que :

```
C₁₁ = P2 + P3 = A₁₁B₁₁ + A₁₂B₂₁  ✓
C₁₂ = T1 + P5 + P6 = A₁₁B₁₂ + A₁₂B₂₂  ✓
C₂₁ = T2 - P7 = A₂₁B₁₁ + A₂₂B₂₁  ✓
C₂₂ = T2 + P5 = A₂₁B₁₂ + A₂₂B₂₂  ✓
```

(La vérification complète nécessite l'expansion de toutes les substitutions S1-S8, P1-P7, T1-T2.)

### 2.17.4 Identité 4 : Symétrie du carré matriciel

**Énoncé** : Si M est symétrique (b = c), alors M² est symétrique.

**Preuve** :

```
M² = [[a²+b², b(a+d)], [b(a+d), b²+d²]]
```

Comme (M²)₁₂ = b(a+d) = (M²)₂₁, la symétrie est préservée. ✓

### 2.17.5 Identité 5 : Correction modulaire

**Énoncé** : `FastDoublingMod(n, m)` retourne F(n) mod m.

**Preuve** : Par induction sur le nombre de bits. À chaque itération, les identités de doublement sont appliquées modulo m. La réduction modulaire préserve l'égalité car :

```
(a × b) mod m = ((a mod m) × (b mod m)) mod m
(a + b) mod m = ((a mod m) + (b mod m)) mod m
```

La gestion du cas négatif (ajout de m) préserve l'équivalence modulo m. ✓

### 2.17.6 Identité 6 : Correction du carré de Fermat

**Énoncé** : Pour x dans Z/(2^(nW)+1), `basicSqr` calcule x² mod (2^(nW)+1).

**Preuve** : Les termes hors-diagonale x[i]·x[j] apparaissent deux fois (symétrie de la multiplication). Le doublement par décalage gauche de 1 bit est correct. L'ajout des termes diagonaux x[i]² complète le carré. La réduction modulo 2^(nW)+1 est identique à celle de `Mul`. ✓

---

## T2.18 — Carte de couverture de tests par algorithme

### 2.18.1 Matrice composant × type de test

| Composant                       | Unit | Table-driven | Golden File | Fuzz                        | Property          | Bench | E2E |
| ------------------------------- | ---- | ------------ | ----------- | --------------------------- | ----------------- | ----- | --- |
| **Fast Doubling**         | ✓   | ✓           | ✓          | FuzzFastDoublingConsistency | gopter identities | ✓    | ✓  |
| **FFT-Based**             | ✓   | ✓           | ✓          | FuzzFFTBasedConsistency     | —                | ✓    | ✓  |
| **Matrix Exp.**           | ✓   | ✓           | ✓          | —                          | —                | ✓    | ✓  |
| **Strassen**              | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **Carré symétrique**    | ✓   | —           | —          | —                          | —                | —    | —  |
| **FastDoublingMod**       | ✓   | ✓           | —          | FuzzFastDoublingMod         | —                | —    | —  |
| **calculateSmall**        | ✓   | ✓           | ✓          | —                          | —                | —    | —  |
| **AdaptiveStrategy**      | ✓   | —           | —          | —                          | —                | —    | —  |
| **FFTOnlyStrategy**       | ✓   | —           | —          | —                          | —                | —    | —  |
| **KaratsubaStrategy**     | ✓   | —           | —          | —                          | —                | —    | —  |
| **ShouldParallelize**     | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **SequenceGenerator**     | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **Skip optimization**     | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **Factory/Registry**      | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **Orchestrator**          | ✓   | ✓           | —          | —                          | —                | —    | ✓  |
| **AnalyzeResults**        | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **CalculationState pool** | ✓   | —           | —          | —                          | —                | —    | —  |
| **matrixState pool**      | ✓   | —           | —          | —                          | —                | —    | —  |
| **CalculationArena**      | ✓   | —           | —          | —                          | —                | —    | —  |
| **bigfft.Mul**            | ✓   | ✓           | —          | —                          | —                | ✓    | —  |
| **bigfft.Sqr**            | ✓   | ✓           | —          | —                          | —                | ✓    | —  |
| **fermat ops**            | ✓   | ✓           | —          | —                          | —                | —    | —  |
| **Poly/PolValues**        | ✓   | —           | —          | —                          | —                | —    | —  |
| **FFT recursion**         | ✓   | —           | —          | —                          | —                | —    | —  |
| **Progress reporting**    | ✓   | ✓           | —          | FuzzProgressMonotonicity    | —                | —    | —  |
| **DynamicThreshold**      | ✓   | ✓           | —          | —                          | —                | —    | —  |

### 2.18.2 Fichiers de test principaux

| Fichier                                           | Couverture                                                                                                                                   |
| ------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------- |
| `internal/fibonacci/fibonacci_test.go`          | Tests unitaires et table-driven pour tous les calculateurs                                                                                   |
| `internal/fibonacci/fibonacci_fuzz_test.go`     | 5 cibles fuzz (FuzzFastDoublingConsistency, FuzzFFTBasedConsistency, FuzzFibonacciIdentities, FuzzProgressMonotonicity, FuzzFastDoublingMod) |
| `internal/fibonacci/fibonacci_property_test.go` | Tests property-based via gopter                                                                                                              |
| `internal/fibonacci/example_test.go`            | Tests d'exemple (documentation exécutable)                                                                                                  |
| `internal/bigfft/*_test.go`                     | Tests unitaires et benchmarks FFT                                                                                                            |
| `internal/orchestration/orchestrator_test.go`   | Tests d'orchestration concurrente                                                                                                            |
| `test/e2e/`                                     | Tests end-to-end du binaire CLI                                                                                                              |

### 2.18.3 Golden file

Le fichier `internal/fibonacci/testdata/fibonacci_golden.json` contient des valeurs pré-calculées de F(n) pour des n de référence. Chaque algorithme est validé contre ces valeurs.

### 2.18.4 Cibles Fuzz

| Cible Fuzz                      | Description                | Invariant vérifié                |
| ------------------------------- | -------------------------- | ---------------------------------- |
| `FuzzFastDoublingConsistency` | Fast Doubling vs itératif | F_fast(n) == F_iter(n)             |
| `FuzzFFTBasedConsistency`     | FFT vs Fast Doubling       | F_fft(n) == F_fast(n)              |
| `FuzzFibonacciIdentities`     | Identités mathématiques  | F(m+n) = F(m-1)F(n) + F(m)F(n+1)   |
| `FuzzProgressMonotonicity`    | Monotonie du progrès      | progress[i] ≤ progress[i+1]       |
| `FuzzFastDoublingMod`         | Modulaire vs complet       | FastDoublingMod(n,m) == F(n) mod m |

### 2.18.5 Objectif de couverture

- **Cible** : > 75% de couverture globale (tel que spécifié dans CLAUDE.md)
- **Hot paths** : Les boucles de doublement et les opérations FFT doivent atteindre > 90%

### 2.18.6 Stratégie de portage des tests en Rust

| Type Go                      | Équivalent Rust                                 |
| ---------------------------- | ------------------------------------------------ |
| `testing.T` / table-driven | `#[test]` + macros `test_case` ou `rstest` |
| `testing.F` (fuzz)         | `cargo-fuzz` / `libfuzzer` / `proptest`    |
| gopter (property-based)      | `proptest` crate                               |
| `testing.B` (bench)        | `criterion` crate                              |
| Golden files JSON            | `serde_json` + `include_str!`                |
| E2E via `exec.Command`     | `assert_cmd` crate                             |

---

## Annexe A — Constantes de configuration par défaut

| Constante                         | Valeur                  | Fichier source             | Usage                                |
| --------------------------------- | ----------------------- | -------------------------- | ------------------------------------ |
| `DefaultParallelThreshold`      | 4 096 bits              | `constants.go`           | Seuil de parallélisation            |
| `DefaultFFTThreshold`           | 500 000 bits            | `constants.go`           | Seuil FFT vs Karatsuba               |
| `DefaultStrassenThreshold`      | 3 072 bits              | `constants.go`           | Seuil Strassen vs naïf              |
| `ParallelFFTThreshold`          | 5 000 000 bits          | `constants.go`           | Réactivation parallèle en mode FFT |
| `MaxPooledBitLen`               | 100 000 000 bits        | `common.go`              | Limite de taille pour le pool        |
| `MaxFibUint64`                  | 93                      | `calculator.go`          | Seuil fast path itératif            |
| `fftThreshold` (bigfft)         | 1 800 mots (~115 Kbits) | `fft.go`                 | Seuil interne FFT                    |
| `smallMulThreshold`             | 30 mots                 | `fermat.go`              | Seuil basicMul vs big.Int            |
| `ParallelFFTRecursionThreshold` | 4 (log2)                | `fft_recursion.go`       | Seuil récursion FFT parallèle      |
| `MaxParallelFFTDepth`           | 3                       | `fft_recursion.go`       | Profondeur max récursion parallèle |
| `ProgressReportThreshold`       | 0.01 (1%)               | `constants.go`           | Seuil de rapport de progrès         |
| `iterativeThreshold` (Skip)     | 1 000                   | `generator_iterative.go` | Seuil Skip itératif vs Calculator   |
| `defaultStrassenThresholdBits`  | 256 bits                | `matrix_ops.go`          | Fallback Strassen atomique           |
| `DynamicAdjustmentInterval`     | 5 itérations           | `dynamic_threshold.go`   | Intervalle d'ajustement dynamique    |
| `HysteresisMargin`              | 0.15 (15%)              | `dynamic_threshold.go`   | Marge d'hystérésis                 |

---

## Annexe B — Glossaire des types pour le portage Rust

| Type Go                        | Type Rust recommandé                                 | Notes                                |
| ------------------------------ | ----------------------------------------------------- | ------------------------------------ |
| `*big.Int`                   | `num_bigint::BigUint`                               | Pas de signe négatif pour Fibonacci |
| `[]big.Word` (nat)           | `Vec<u64>`                                          | Même représentation little-endian  |
| `fermat`                     | `Vec<u64>` (n+1 éléments)                         | Wrapper newtype recommandé          |
| `sync.Pool`                  | `crossbeam::queue::ArrayQueue` ou `thread_local!` | Pas d'équivalent direct             |
| `sync.WaitGroup`             | `std::thread::scope` ou `rayon::scope`            | Scoped threads préférés           |
| `chan struct{}` (sémaphore) | `tokio::sync::Semaphore` ou `crossbeam`           | Selon async vs sync                  |
| `atomic.Int32`               | `AtomicI32`                                         | Direct                               |
| `context.Context`            | `tokio::CancellationToken` ou `Arc<AtomicBool>`   | Pattern d'annulation                 |
| `errgroup.Group`             | `tokio::task::JoinSet` ou `rayon::scope`          | Selon async vs sync                  |
| `parallel.ErrorCollector`    | `Arc<Mutex<Option<Error>>>`                         | First-error-wins                     |

---

*Fin de la Phase 2 — Spécifications Algorithmiques*

# Phase 3 — Système d'Observation & Suivi de Progression (T3.1–T3.10)

---

## T3.1 — Architecture du Patron Observer (UML, Cycle de Vie, Thread-Safety)

### Vue d'ensemble

Le système Observer découple le moteur de calcul de la couche de présentation. Les algorithmes de Fibonacci émettent des événements de progression normalisés (0.0 → 1.0) sans connaître leurs consommateurs (CLI, TUI, logs, métriques).

### Diagramme de classes UML

```
┌─────────────────────────────┐
│    «interface»               │
│    ProgressObserver          │
├─────────────────────────────┤
│ + Update(calcIndex: int,     │
│          progress: float64)  │
└──────────┬──────────────────┘
           │ implémente
     ┌─────┼──────────────────────────┐
     │     │                          │
     ▼     ▼                          ▼
┌──────────────┐ ┌───────────────┐ ┌──────────────┐
│ Channel      │ │ Logging       │ │ NoOp         │
│ Observer     │ │ Observer      │ │ Observer     │
├──────────────┤ ├───────────────┤ ├──────────────┤
│ -channel     │ │ -logger       │ │              │
│  chan<-       │ │ -threshold    │ │              │
│  ProgressUpd │ │ -lastLog      │ │              │
│              │ │ -mu: Mutex    │ │              │
├──────────────┤ ├───────────────┤ ├──────────────┤
│ +Update()    │ │ +Update()     │ │ +Update()    │
└──────────────┘ └───────────────┘ └──────────────┘

┌────────────────────────────────────────────────┐
│              ProgressSubject                    │
├────────────────────────────────────────────────┤
│ - observers: []ProgressObserver                │
│ - mu: sync.RWMutex                             │
├────────────────────────────────────────────────┤
│ + NewProgressSubject() → *ProgressSubject      │
│ + Register(observer: ProgressObserver)         │
│ + Unregister(observer: ProgressObserver)       │
│ + Notify(calcIndex: int, progress: float64)    │
│ + ObserverCount() → int                        │
│ + AsProgressCallback(calcIndex) → Callback     │
│ + Freeze(calcIndex) → ProgressCallback         │
└────────────────────────────────────────────────┘
```

### Cycle de vie Register → Notify → Unregister

```
Fil principal                     Fil de calcul
     │                                  │
     ├─── NewProgressSubject() ─────────┤
     │         │                        │
     ├─── Register(ChannelObserver) ────┤
     ├─── Register(LoggingObserver) ────┤
     │         │                        │
     ├─── Freeze(calcIndex) ───────┐    │
     │         │                   │    │
     │         │              snapshot  │
     │         │              (lock-free)
     │         │                   │    │
     │         │                   ├──→ reporter(0.05)
     │         │                   ├──→ reporter(0.15)
     │         │                   ├──→ reporter(0.50)
     │         │                   ├──→ reporter(1.00)
     │         │                   │    │
     ├─── Unregister (optionnel) ──┤    │
     │         │                        │
     ▼         ▼                        ▼
```

### Thread-Safety — Analyse Go

| Opération          | Verrou                  | Contention attendue                |
| ------------------- | ----------------------- | ---------------------------------- |
| `Register()`      | `mu.Lock()`           | Rare (initialisation uniquement)   |
| `Unregister()`    | `mu.Lock()`           | Rare (nettoyage uniquement)        |
| `Notify()`        | `mu.RLock()`          | Fréquent — verrouillage partagé |
| `Freeze()`        | `mu.RLock()` (1 fois) | Unique — puis lock-free           |
| `ObserverCount()` | `mu.RLock()`          | Diagnostic uniquement              |

**Implémentation Go** (`internal/fibonacci/observer.go`):

```go
type ProgressSubject struct {
    observers []ProgressObserver
    mu        sync.RWMutex
}

func (s *ProgressSubject) Register(observer ProgressObserver) {
    if observer == nil { return }
    s.mu.Lock()
    defer s.mu.Unlock()
    s.observers = append(s.observers, observer)
}

func (s *ProgressSubject) Notify(calcIndex int, progress float64) {
    s.mu.RLock()
    defer s.mu.RUnlock()
    for _, observer := range s.observers {
        observer.Update(calcIndex, progress)
    }
}
```

### Transposition Rust

```rust
use std::sync::{Arc, RwLock};

/// Trait Observer pour les notifications de progression.
pub trait ProgressObserver: Send + Sync {
    fn update(&self, calc_index: usize, progress: f64);
}

/// Sujet observable gérant les abonnements et notifications.
pub struct ProgressSubject {
    observers: RwLock<Vec<Arc<dyn ProgressObserver>>>,
}

impl ProgressSubject {
    pub fn new() -> Self {
        Self {
            observers: RwLock::new(Vec::new()),
        }
    }

    pub fn register(&self, observer: Arc<dyn ProgressObserver>) {
        let mut obs = self.observers.write().unwrap();
        obs.push(observer);
    }

    pub fn unregister(&self, observer: &Arc<dyn ProgressObserver>) {
        let mut obs = self.observers.write().unwrap();
        obs.retain(|o| !Arc::ptr_eq(o, observer));
    }

    pub fn notify(&self, calc_index: usize, progress: f64) {
        let obs = self.observers.read().unwrap();
        for observer in obs.iter() {
            observer.update(calc_index, progress);
        }
    }

    pub fn observer_count(&self) -> usize {
        self.observers.read().unwrap().len()
    }
}
```

**Choix Rust** : `RwLock<Vec<Arc<dyn ProgressObserver>>>` — le `RwLock` autorise les lectures concurrentes (Notify) tout en sérialisant les écritures (Register/Unregister). `Arc` permet le partage multi-thread du trait object.

---

## T3.2 — Mécanisme Freeze() pour Snapshots Lock-Free

### Problème résolu

Dans la boucle de calcul interne (itérant sur les bits de n), chaque appel à `Notify()` acquiert un `RLock`. Pour F(100M), cela représente ~27 appels à `Notify()` (seuil 1%), chacun nécessitant une acquisition de verrou. Bien que les `RLock` soient peu coûteux, `Freeze()` élimine totalement ce coût.

### Sémantique du Snapshot

```
         Temps ──────────────────────────────────────────────►

Phase 1: Registration         Phase 2: Calcul (boucle chaude)
┌─────────────────────┐       ┌────────────────────────────────┐
│ Register(obs1)      │       │  reporter(0.01)   ← lock-free │
│ Register(obs2)      │       │  reporter(0.05)   ← lock-free │
│ Register(obs3)      │       │  reporter(0.15)   ← lock-free │
│                     │       │  ...                           │
│ reporter = Freeze() │─────► │  reporter(1.00)   ← lock-free │
│    ↑                │       │                                │
│    └─ copie snapshot│       │  Aucun verrou acquis !         │
└─────────────────────┘       └────────────────────────────────┘
```

### Implémentation Go

```go
func (s *ProgressSubject) Freeze(calcIndex int) ProgressCallback {
    s.mu.RLock()
    snapshot := make([]ProgressObserver, len(s.observers))
    copy(snapshot, s.observers)
    s.mu.RUnlock()

    // Fermeture capturant le snapshot — aucun verrou nécessaire
    return func(progress float64) {
        for _, observer := range snapshot {
            observer.Update(calcIndex, progress)
        }
    }
}
```

### Analyse de concurrence

| Propriété                 | Garantie                                                               |
| --------------------------- | ---------------------------------------------------------------------- |
| Visibilité du snapshot     | Les observers enregistrés au moment de `Freeze()` sont capturés    |
| Enregistrements tardifs     | Non visibles dans le snapshot figé                                    |
| Désenregistrements tardifs | Le snapshot retient une référence → pas de dangling pointer (GC Go) |
| Coût par appel reporter    | 0 acquisition de verrou, 1 boucle sur slice                            |

### Transposition Rust

```rust
/// Type alias pour le callback de progression.
pub type ProgressCallback = Box<dyn Fn(f64) + Send + Sync>;

impl ProgressSubject {
    /// Crée un snapshot lock-free des observers actuels.
    ///
    /// Le snapshot capture les observers enregistrés au moment de l'appel.
    /// Les appels ultérieurs au callback ne nécessitent aucun verrou.
    pub fn freeze(&self, calc_index: usize) -> ProgressCallback {
        // Une seule acquisition de verrou
        let snapshot: Vec<Arc<dyn ProgressObserver>> = {
            let obs = self.observers.read().unwrap();
            obs.clone() // Clone les Arc (incrémente compteurs de référence)
        };

        // Fermeture capturant le snapshot par déplacement
        Box::new(move |progress: f64| {
            for observer in &snapshot {
                observer.update(calc_index, progress);
            }
        })
    }
}
```

**Point clé Rust** : Le `clone()` du `Vec<Arc<dyn ProgressObserver>>` ne clone que les pointeurs `Arc` (incrémentation atomique du compteur de référence), pas les observers eux-mêmes. Coût: O(n) atomiques, n = nombre d'observers (typiquement 1–3).

### Chemin rapide sans observers

```go
// Go — CalculateWithObservers
var reporter ProgressCallback
if subject != nil && subject.ObserverCount() > 0 {
    reporter = subject.Freeze(calcIndex)
} else {
    reporter = func(float64) {} // No-op — coût nul
}
```

```rust
// Rust — equivalent
let reporter: ProgressCallback = if subject.observer_count() > 0 {
    subject.freeze(calc_index)
} else {
    Box::new(|_: f64| {}) // No-op
};
```

---

## T3.3 — Modèle Géométrique de Travail (Série de Puissances de 4)

### Modèle mathématique

Les algorithmes O(log n) pour Fibonacci (Fast Doubling, Matrix Exponentiation) itèrent sur les bits de n du MSB (bit de poids fort) au LSB (bit de poids faible). À chaque étape, les opérandes doublent approximativement en taille, quadruplant le coût de multiplication.

#### Formule du travail total

```
TotalWork = Σ(i=0 → numBits-1) 4^i = (4^numBits - 1) / 3
```

Ceci est la somme partielle d'une série géométrique de raison r = 4 :

```
S_n = a × (r^n - 1) / (r - 1)   avec a = 1, r = 4
    = (4^n - 1) / (4 - 1)
    = (4^n - 1) / 3
```

#### Travail par étape

Pour le bit `i` (comptant de `numBits - 1` vers 0):

```
stepIndex = numBits - 1 - i
WorkOfStep(i) = 4^stepIndex = 4^(numBits - 1 - i)
```

#### Progression

```
Progress(i) = WorkDone(i) / TotalWork
            = Σ(j=0 → stepIndex) 4^j / ((4^numBits - 1) / 3)
            = (4^(stepIndex+1) - 1) / (4^numBits - 1)
```

### Exemples numériques

#### Exemple 1 : n = 1 000 000 (numBits = 20)

| Itération i | stepIndex | WorkOfStep (4^si) | WorkDone cumulé | Progress (%) |
| :----------: | :-------: | ----------------: | ---------------: | -----------: |
|   19 (MSB)   |     0     |                 1 |                1 |      0.000 % |
|      18      |     1     |                 4 |                5 |      0.000 % |
|      15      |     4     |               256 |              341 |      0.000 % |
|      10      |     9     |           262 144 |          349 525 |      0.025 % |
|      5      |    14    |       268 435 456 |      357 913 941 |       25.6 % |
|      2      |    17    |     ~1.7 × 10^10 |    ~2.3 × 10^10 |       82.5 % |
|   0 (LSB)   |    19    |     ~2.7 × 10^11 |    ~3.7 × 10^11 |        100 % |

**TotalWork(20) = (4^20 - 1) / 3 ≈ 3.66 × 10^11**

#### Exemple 2 : n = 10 000 000 (numBits = 24)

| Phase            | Bits traités | Progress approximative |
| ---------------- | :-----------: | :--------------------: |
| 20 premiers bits |    19 → 4    |         ~0.4 %         |
| Bits 4 → 2      |    3 bits    |         ~6.2 %         |
| Bit 1            |     1 bit     |         ~25 %         |
| Bit 0 (final)    |     1 bit     |        → 100 %        |

**Observation** : ~75% du travail total est concentré dans les 2 derniers bits.

### Distribution du travail

```
Travail (log)
    │
    │                                                    ████
    │                                               ████████
    │                                          ████████████
    │                                     ████████████████
    │                                ████████████████████
    │                           ████████████████████████
    │                      ████████████████████████████
    │                 ████████████████████████████████
    │   ░░░░░░░░████████████████████████████████████
    │   ░░░░░░░░████████████████████████████████████
    └───────────────────────────────────────────────► bits
       MSB                                       LSB
       (travail minimal)              (travail maximal)

    ░ = 50% premiers bits ≈ 0.02% du travail total
    █ = derniers 3 bits ≈ 75% du travail total
```

---

## T3.4 — Précomputation des Puissances de 4 (Table Globale [64]float64)

### Architecture zéro-allocation

La table globale `[64]float64` est initialisée au démarrage du programme via `init()`. Pour les entrées typiques (n ≤ 2^64), `numBits ≤ 64`, donc `PrecomputePowers4` retourne un sous-slice de la table globale **sans aucune allocation mémoire**.

### Implémentation Go

```go
// Table globale — initialisée une seule fois au démarrage
var powersOf4 [64]float64

func init() {
    powersOf4[0] = 1.0
    for i := 1; i < 64; i++ {
        powersOf4[i] = powersOf4[i-1] * 4.0
    }
}

func PrecomputePowers4(numBits int) []float64 {
    if numBits <= 0 {
        return nil
    }
    if numBits > 64 {
        // Cas rare : fallback avec allocation
        powers := make([]float64, numBits)
        copy(powers, powersOf4[:])
        for i := 64; i < numBits; i++ {
            powers[i] = powers[i-1] * 4.0
        }
        return powers
    }
    return powersOf4[:numBits] // Zéro allocation !
}
```

### Transposition Rust

```rust
use std::sync::LazyLock;

/// Table globale des puissances de 4, initialisée paresseusement.
/// Pour n: u64, numBits ≤ 64, donc 64 entrées suffisent.
static POWERS_OF_4: LazyLock<[f64; 64]> = LazyLock::new(|| {
    let mut table = [0.0f64; 64];
    table[0] = 1.0;
    for i in 1..64 {
        table[i] = table[i - 1] * 4.0;
    }
    table
});

/// Retourne un slice de la table globale des puissances de 4.
/// Zéro allocation pour numBits ≤ 64 (cas typique).
pub fn precompute_powers4(num_bits: usize) -> &'static [f64] {
    if num_bits == 0 {
        return &[];
    }
    if num_bits > 64 {
        // En Rust, on ne peut pas retourner une référence à une allocation locale.
        // Alternative : retourner un Vec ou utiliser un paramètre de sortie.
        panic!("num_bits > 64 not supported for u64 inputs");
    }
    &POWERS_OF_4[..num_bits]
}
```

**Alternative Rust avec `const`** :

```rust
/// Évaluation à la compilation (const fn).
const fn make_powers_of_4() -> [f64; 64] {
    let mut table = [0.0f64; 64];
    table[0] = 1.0;
    let mut i = 1;
    while i < 64 {
        table[i] = table[i - 1] * 4.0;
        i += 1;
    }
    table
}

/// Table calculée à la compilation — strictement zéro coût à l'exécution.
const POWERS_OF_4: [f64; 64] = make_powers_of_4();
```

**Avantage Rust** : Avec `const fn`, la table est calculée **au moment de la compilation**, éliminant même le coût de l'initialisation paresseuse.

### Comparaison des coûts

| Opération                 | Go                    | Rust (const)        |
| -------------------------- | --------------------- | ------------------- |
| Initialisation             | `init()` au runtime | Compilation         |
| Lookup par itération      | O(1) — index slice   | O(1) — index array |
| Allocation par appel       | 0 (slice global)      | 0 (ref statique)    |
| `math.Pow(4, x)` évité | ~50 ns/appel          | ~50 ns/appel        |

---

## T3.5 — Seuil de Rapport de Progression (Déclenchement Conditionnel 1%)

### Constante de seuil

```go
// constants.go
const ProgressReportThreshold = 0.01 // 1%
```

### Logique de déclenchement

Le rapport de progression est émis si et seulement si l'une des conditions suivantes est remplie :

```
Conditions de rapport :
  ┌──────────────────────────────────────────────┐
  │  1. currentProgress - lastReported ≥ 0.01    │  ← changement ≥ 1%
  │  OU                                          │
  │  2. i == numBits - 1                         │  ← première itération (MSB)
  │  OU                                          │
  │  3. i == 0                                   │  ← dernière itération (LSB)
  └──────────────────────────────────────────────┘
```

### Implémentation Go — `ReportStepProgress`

```go
func ReportStepProgress(progressReporter ProgressCallback, lastReported *float64,
    totalWork, workDone float64, i, numBits int, powers []float64) float64 {

    stepIndex := numBits - 1 - i
    workOfStep := powers[stepIndex] // O(1) lookup

    currentTotalDone := workDone + workOfStep

    if totalWork > 0 {
        currentProgress := currentTotalDone / totalWork
        if currentProgress-*lastReported >= ProgressReportThreshold ||
           i == 0 || i == numBits-1 {
            progressReporter(currentProgress)
            *lastReported = currentProgress
        }
    }
    return currentTotalDone
}
```

### Transposition Rust

```rust
const PROGRESS_REPORT_THRESHOLD: f64 = 0.01;

/// Rapporte la progression si le seuil est atteint ou si c'est
/// la première/dernière itération.
pub fn report_step_progress(
    reporter: &dyn Fn(f64),
    last_reported: &mut f64,
    total_work: f64,
    work_done: f64,
    i: usize,
    num_bits: usize,
    powers: &[f64],
) -> f64 {
    let step_index = num_bits - 1 - i;
    let work_of_step = powers[step_index];
    let current_total_done = work_done + work_of_step;

    if total_work > 0.0 {
        let current_progress = current_total_done / total_work;
        if current_progress - *last_reported >= PROGRESS_REPORT_THRESHOLD
            || i == 0
            || i == num_bits - 1
        {
            reporter(current_progress);
            *last_reported = current_progress;
        }
    }
    current_total_done
}
```

### Impact sur le nombre d'appels

Pour F(10M) avec numBits = 24 :

- Sans seuil : 24 appels (1 par bit)
- Avec seuil 1% : ~27 appels (première + dernière + ~25 seuils franchis)
- Avec seuil 5% : ~22 appels
- Avec seuil 10% : ~14 appels

Le seuil de 1% offre le meilleur compromis entre fluidité de l'affichage et coût des notifications.

---

## T3.6 — ChannelObserver (Envoi Non-bloquant, select/default)

### Architecture

Le `ChannelObserver` fait le pont entre le patron Observer et la communication par canaux Go, utilisée par le système CLI/TUI existant.

### Implémentation Go

```go
type ChannelObserver struct {
    channel chan<- ProgressUpdate
}

func NewChannelObserver(ch chan<- ProgressUpdate) *ChannelObserver {
    return &ChannelObserver{channel: ch}
}

func (o *ChannelObserver) Update(calcIndex int, progress float64) {
    if o.channel == nil {
        return
    }
    // Borner la progression à [0.0, 1.0]
    if progress > 1.0 {
        progress = 1.0
    }
    update := ProgressUpdate{CalculatorIndex: calcIndex, Value: progress}

    // Envoi non-bloquant — le patron select/default
    select {
    case o.channel <- update:
        // Envoyé avec succès
    default:
        // Canal plein → on ignore la mise à jour (la UI rattrapera)
    }
}
```

### Patron `select/default` — Analyse détaillée

```
Canal avec capacité 100 (tampon)
┌──────────────────────────────────────────────────┐
│ slot 0 │ slot 1 │ ... │ slot 98 │ slot 99        │
│  ████  │  ████  │     │  ████   │  ░░░░  (libre) │
└──────────────────────────────────────────────────┘
                                     ↑
                                  pointeur d'écriture

Cas 1 : Canal non plein → select choisit le case
  → L'update est envoyé, la goroutine continue immédiatement

Cas 2 : Canal plein → select choisit default
  → L'update est ignoré, la goroutine continue immédiatement
  → AUCUN blocage — critique dans la boucle de calcul hot-path
```

**Capacité recommandée** : 100 slots (définie dans `internal/orchestration/`). Suffisante pour que l'UI ne perde jamais de mise à jour en conditions normales.

### Transposition Rust

```rust
use std::sync::mpsc;

/// Observer qui envoie les mises à jour de progression via un canal MPSC.
pub struct ChannelObserver {
    sender: mpsc::SyncSender<ProgressUpdate>,
}

impl ChannelObserver {
    pub fn new(sender: mpsc::SyncSender<ProgressUpdate>) -> Self {
        Self { sender }
    }
}

impl ProgressObserver for ChannelObserver {
    fn update(&self, calc_index: usize, progress: f64) {
        let progress = progress.min(1.0); // Clamp
        let update = ProgressUpdate {
            calculator_index: calc_index,
            value: progress,
        };
        // try_send = non-bloquant (équivalent de select/default)
        let _ = self.sender.try_send(update);
        // Ignore Err(TrySendError::Full) — la UI rattrapera
    }
}
```

**Alternative avec `crossbeam::channel`** :

```rust
use crossbeam::channel::{Sender, TrySendError};

pub struct ChannelObserver {
    sender: Sender<ProgressUpdate>,
}

impl ProgressObserver for ChannelObserver {
    fn update(&self, calc_index: usize, progress: f64) {
        let update = ProgressUpdate {
            calculator_index: calc_index,
            value: progress.min(1.0),
        };
        // crossbeam try_send — zéro allocation, lock-free
        let _ = self.sender.try_send(update);
    }
}
```

**Recommandation** : Utiliser `crossbeam::channel` pour des performances supérieures (lock-free, zéro allocation par envoi).

---

## T3.7 — LoggingObserver (Throttling Temporel, Sélection du Niveau de Log)

### Architecture

Le `LoggingObserver` émet des logs structurés via `zerolog` avec un système de throttling basé sur un seuil de changement minimum pour éviter le spam de logs.

### Implémentation Go

```go
type LoggingObserver struct {
    logger    zerolog.Logger
    threshold float64         // Changement minimum pour logger (ex: 0.1 = 10%)
    lastLog   map[int]float64 // Dernière progression loggée par calculateur
    mu        sync.Mutex      // Protège lastLog
}

func (o *LoggingObserver) Update(calcIndex int, progress float64) {
    o.mu.Lock()
    defer o.mu.Unlock()

    lastProgress := o.lastLog[calcIndex]

    // Logger aux frontières ou aux changements significatifs
    shouldLog := progress >= 1.0 ||
        lastProgress == 0 && progress > 0 ||
        progress-lastProgress >= o.threshold

    if shouldLog {
        o.logger.Debug().
            Int("calculator", calcIndex).
            Float64("progress", progress).
            Str("percent", fmt.Sprintf("%.1f%%", progress*100)).
            Msg("calculation progress")
        o.lastLog[calcIndex] = progress
    }
}
```

### Conditions de déclenchement du log

```
                    shouldLog = true si :
  ┌─────────────────────────────────────────────────┐
  │  1. progress ≥ 1.0             (complétion)     │
  │  OU                                             │
  │  2. lastProgress == 0 &&       (premier progrès)│
  │     progress > 0                                │
  │  OU                                             │
  │  3. progress - lastProgress    (changement      │
  │     ≥ threshold                 significatif)   │
  └─────────────────────────────────────────────────┘
```

### Transposition Rust

```rust
use std::collections::HashMap;
use std::sync::Mutex;
use tracing::{debug, Level};

pub struct LoggingObserver {
    threshold: f64,
    last_log: Mutex<HashMap<usize, f64>>,
}

impl LoggingObserver {
    pub fn new(threshold: f64) -> Self {
        let threshold = if threshold <= 0.0 { 0.1 } else { threshold };
        Self {
            threshold,
            last_log: Mutex::new(HashMap::new()),
        }
    }
}

impl ProgressObserver for LoggingObserver {
    fn update(&self, calc_index: usize, progress: f64) {
        let mut last_log = self.last_log.lock().unwrap();
        let last_progress = *last_log.get(&calc_index).unwrap_or(&0.0);

        let should_log = progress >= 1.0
            || (last_progress == 0.0 && progress > 0.0)
            || (progress - last_progress >= self.threshold);

        if should_log {
            debug!(
                calculator = calc_index,
                progress = progress,
                percent = format!("{:.1}%", progress * 100.0),
                "calculation progress"
            );
            last_log.insert(calc_index, progress);
        }
    }
}
```

**Choix Rust** : `tracing` (crate standard de logging structuré) remplace `zerolog`. Le `Mutex<HashMap>` est acceptable car le `LoggingObserver` n'est jamais sur le hot-path après `Freeze()` (il est dans le snapshot, mais le throttling limite les appels effectifs).

---

## T3.8 — NoOpObserver (Patron Null Object)

### Patron de conception

Le `NoOpObserver` implémente le patron Null Object : il satisfait l'interface `ProgressObserver` sans effectuer aucune opération. Cela élimine les vérifications `nil` dans le code client.

### Implémentation Go

```go
type NoOpObserver struct{}

func NewNoOpObserver() *NoOpObserver {
    return &NoOpObserver{}
}

func (o *NoOpObserver) Update(calcIndex int, progress float64) {
    // Intentionnellement vide — patron Null Object
}
```

### Cas d'utilisation

| Scénario                     | Usage                                       |
| ----------------------------- | ------------------------------------------- |
| Tests unitaires               | Observer par défaut — pas de side effects |
| Mode silencieux (`--quiet`) | Aucune sortie de progression                |
| Benchmarks                    | Éliminer le coût des notifications        |
| Observer optionnel            | Éviter les vérifications `nil`          |

### Transposition Rust

```rust
/// Observer null — ne fait rien, élimine les vérifications d'option.
pub struct NoOpObserver;

impl ProgressObserver for NoOpObserver {
    #[inline]
    fn update(&self, _calc_index: usize, _progress: f64) {
        // Intentionnellement vide
    }
}
```

**Optimisation Rust** : L'annotation `#[inline]` permet au compilateur d'éliminer complètement l'appel lors de la monomorphisation, rendant le coût strictement nul.

---

## T3.9 — Structure ProgressUpdate (Champs et Sémantique)

### Définition

```go
type ProgressUpdate struct {
    CalculatorIndex int     // Identifiant unique du calculateur (0, 1, 2...)
    Value           float64 // Progression normalisée [0.0, 1.0]
}
```

### Sémantique des champs

| Champ               | Type        | Plage      | Sémantique                                          |
| ------------------- | ----------- | ---------- | ---------------------------------------------------- |
| `CalculatorIndex` | `int`     | [0, N-1]   | Identifie le calculateur parmi N calculs concurrents |
| `Value`           | `float64` | [0.0, 1.0] | Progression normalisée (0% → 100%)                 |

### Invariants

1. **Monotonie** : `Value` ne décroît jamais pour un `CalculatorIndex` donné
2. **Bornes** : `0.0 ≤ Value ≤ 1.0` (clampé dans `ChannelObserver.Update`)
3. **Complétion** : Le dernier `ProgressUpdate` pour un calcul réussi a `Value ≈ 1.0`
4. **Unicité d'index** : Chaque calculateur concurrent utilise un `CalculatorIndex` distinct

### Callback fonctionnel associé

```go
type ProgressCallback func(progress float64)
```

Le `ProgressCallback` est la forme simplifiée utilisée par les algorithmes internes. Il encapsule le `CalculatorIndex` via fermeture (closure), évitant de le passer à chaque appel.

### Transposition Rust

```rust
/// Mise à jour de progression envoyée via canal.
#[derive(Debug, Clone, Copy)]
pub struct ProgressUpdate {
    /// Identifiant du calculateur (pour calculs concurrents).
    pub calculator_index: usize,
    /// Progression normalisée [0.0, 1.0].
    pub value: f64,
}

/// Callback fonctionnel pour le rapport de progression.
/// Encapsule l'index du calculateur par capture.
pub type ProgressCallback = Box<dyn Fn(f64) + Send + Sync>;
```

**Point Rust** : `#[derive(Copy)]` est possible car les deux champs (`usize`, `f64`) sont `Copy`, rendant le transfert par canal sans allocation.

---

## T3.10 — Cycle de Vie d'Enregistrement des Observers (Diagramme de Séquence)

### Diagramme de séquence complet

```
Orchestrateur          ProgressSubject         ChannelObs       LoggingObs       Algorithme
     │                       │                     │                │                │
     │  NewProgressSubject() │                     │                │                │
     │──────────────────────►│                     │                │                │
     │                       │                     │                │                │
     │  Register(chanObs)    │                     │                │                │
     │──────────────────────►│ mu.Lock()           │                │                │
     │                       │ append(chanObs)     │                │                │
     │                       │ mu.Unlock()         │                │                │
     │                       │                     │                │                │
     │  Register(logObs)     │                     │                │                │
     │──────────────────────►│ mu.Lock()           │                │                │
     │                       │ append(logObs)      │                │                │
     │                       │ mu.Unlock()         │                │                │
     │                       │                     │                │                │
     │           CalculateWithObservers(subject, calcIndex, n, opts)                │
     │────────────────────────────────────────────────────────────────────────────►  │
     │                       │                     │                │                │
     │                       │  Freeze(calcIndex)  │                │                │
     │                       │◄─────────────────────────────────────────────────────│
     │                       │  mu.RLock()         │                │                │
     │                       │  copy(snapshot)     │                │                │
     │                       │  mu.RUnlock()       │                │                │
     │                       │  return reporter ───────────────────────────────────►│
     │                       │                     │                │                │
     │                       │     [BOUCLE DE CALCUL — LOCK-FREE]                   │
     │                       │                     │                │                │
     │                       │                     │  reporter(0.01)                 │
     │                       │                     │◄───────────────────────────────│
     │                       │                     │  Update(idx, 0.01)             │
     │                       │                     │  select: ch <- update          │
     │                       │                     │                │                │
     │                       │                     │                │  Update(0, 0.01)
     │                       │                     │                │◄──────────────│
     │                       │                     │                │  (sous seuil,  │
     │                       │                     │                │   pas de log)  │
     │                       │                     │                │                │
     │                       │                     │  reporter(0.50)                 │
     │                       │                     │◄───────────────────────────────│
     │                       │                     │  ch <- update  │                │
     │                       │                     │                │  Update(0,0.50)│
     │                       │                     │                │  → Debug log   │
     │                       │                     │                │                │
     │                       │                     │  reporter(1.00)                 │
     │                       │                     │◄───────────────────────────────│
     │                       │                     │  ch <- update  │  Update(0,1.0) │
     │                       │                     │                │  → Debug log   │
     │                       │                     │                │                │
     │           return (result, nil)              │                │                │
     │◄────────────────────────────────────────────────────────────────────────────│
     │                       │                     │                │                │
     │  [Nettoyage implicite — GC Go / Drop Rust]  │                │                │
     │                       │                     │                │                │
     ▼                       ▼                     ▼                ▼                ▼
```

### Points critiques du cycle de vie

1. **Phase d'enregistrement** : Sérialisée (Write Lock), se fait avant le calcul
2. **Gel (Freeze)** : Transition unique de l'état mutable à l'état immuable
3. **Phase de notification** : Entièrement lock-free, sur le hot-path du calcul
4. **Nettoyage** : Automatique — GC en Go, `Drop` en Rust (les `Arc` décrémenteront)

---

# Phase 4 — Gestion Mémoire & Concurrence (T4.1–T4.12)

---

## T4.1 — Allocateur Bump pour CalculationArena (Protocole d'Allocation, Mapping bumpalo)

### Vue d'ensemble

Le `CalculationArena` pré-alloue un bloc contigu de mémoire (`[]big.Word`) pour tous les `big.Int` temporaires d'un calcul Fibonacci. Cela élimine le tracking GC par buffer et permet une libération O(1) via `Reset()`.

### Protocole d'allocation

```
 CalculationArena
┌──────────────────────────────────────────────────────────┐
│ buf: []big.Word                                          │
│ ┌──────┬──────┬──────┬──────┬──────┬─────────────────┐  │
│ │  FK  │ FK1  │  T1  │  T2  │  T3  │    (marge)      │  │
│ │ words│ words│ words│ words│ words│                   │  │
│ └──────┴──────┴──────┴──────┴──────┴─────────────────┘  │
│         ↑                                                │
│      offset (avance à chaque AllocBigInt)                │
└──────────────────────────────────────────────────────────┘
```

### Dimensionnement

```go
func NewCalculationArena(n uint64) *CalculationArena {
    if n < 1000 {
        return &CalculationArena{} // Pas d'arène pour les petits n
    }
    estimatedBits := float64(n) * 0.69424   // log2(φ) ≈ 0.69424
    wordsPerInt := int(estimatedBits/64) + 1
    totalWords := wordsPerInt * 10           // 5 état + 5 marge
    return &CalculationArena{
        buf: make([]big.Word, totalWords),
    }
}
```

**Formule** :

```
Taille de F(n) ≈ n × log₂(φ) bits ≈ n × 0.69424 bits
Words par big.Int = ⌈n × 0.69424 / 64⌉ + 1
Total words = wordsPerInt × 10  (5 CalculationState + 5 marge)
```

### Implémentation Go — AllocBigInt

```go
func (a *CalculationArena) AllocBigInt(words int) *big.Int {
    if words <= 0 {
        return new(big.Int)
    }
    z := new(big.Int)
    if a.buf == nil || a.offset+words > len(a.buf) {
        // Fallback : allocation sur le tas
        buf := make([]big.Word, 0, words)
        z.SetBits(buf)
        return z
    }
    slice := a.buf[a.offset : a.offset+words : a.offset+words]
    a.offset += words
    z.SetBits(slice[:0]) // longueur 0, capacité words — z vaut 0
    return z
}
```

### Transposition Rust avec `bumpalo`

```rust
use bumpalo::Bump;
use num_bigint::BigUint;

/// Arène de calcul utilisant bumpalo pour l'allocation bump.
pub struct CalculationArena {
    bump: Bump,
}

impl CalculationArena {
    /// Crée une arène dimensionnée pour F(n).
    pub fn new(n: u64) -> Self {
        if n < 1000 {
            return Self { bump: Bump::new() };
        }
        let estimated_bits = n as f64 * 0.69424;
        let bytes_per_int = (estimated_bits / 8.0) as usize + 8;
        let total_bytes = bytes_per_int * 10; // 5 état + 5 marge
        Self {
            bump: Bump::with_capacity(total_bytes),
        }
    }

    /// Alloue un buffer de `words` mots depuis l'arène.
    /// Retourne un Vec<u64> dont le stockage est dans l'arène.
    pub fn alloc_words(&self, words: usize) -> &mut [u64] {
        self.bump.alloc_slice_fill_default(words)
    }

    /// Réinitialise l'arène — O(1), toutes les allocations deviennent invalides.
    pub fn reset(&mut self) {
        self.bump.reset();
    }
}
```

**Avantage bumpalo** :

- Allocation O(1) par incrément de pointeur
- Libération O(1) via `reset()` (pas de destructeurs individuels)
- Localité de cache excellente (mémoire contiguë)
- Sécurité mémoire garantie par le borrow checker (les références à l'arène ne survivent pas au `reset`)

### Comparaison Go vs Rust

| Aspect               | Go (CalculationArena)              | Rust (bumpalo::Bump)          |
| -------------------- | ---------------------------------- | ----------------------------- |
| Sécurité mémoire  | Manuelle (Reset invalide les réf) | Borrow checker (compile-time) |
| Fallback si épuisé | Allocation tas                     | Allocation tas (ou panic)     |
| Fragmentation        | Zéro (contigu)                    | Zéro (contigu)               |
| Libération          | Reset() — O(1)                    | reset() — O(1)               |
| Tracking GC          | Le bloc unique est tracké         | Pas de GC                     |

---

## T4.2 — Contrôleur GC (Stratégie Go : Désactiver GC + Limite Douce → Alternative RAII Rust)

### Problème résolu

Pour les grands calculs (n ≥ 1 000 000), le ramasse-miettes (GC) Go provoque des pauses imprévisibles qui dégradent les performances. Le `GCController` désactive temporairement le GC avec un filet de sécurité (limite mémoire douce).

### Implémentation Go

```go
type GCController struct {
    mode              GCMode   // "auto", "aggressive", "disabled"
    originalGCPercent int
    active            bool
    startStats        runtime.MemStats
    endStats          runtime.MemStats
}

const GCAutoThreshold uint64 = 1_000_000

func (gc *GCController) Begin() {
    if !gc.active { return }
    runtime.ReadMemStats(&gc.startStats)
    gc.originalGCPercent = debug.SetGCPercent(-1) // Désactive le GC
    // Filet de sécurité : limite douce = 3× la mémoire système actuelle
    if gc.startStats.Sys > 0 {
        limit := int64(float64(gc.startStats.Sys) * 3)
        if limit > 0 {
            debug.SetMemoryLimit(limit)
        }
    }
}

func (gc *GCController) End() {
    if !gc.active { return }
    runtime.ReadMemStats(&gc.endStats)
    debug.SetGCPercent(gc.originalGCPercent) // Restaure le GC
    debug.SetMemoryLimit(math.MaxInt64)       // Retire la limite
    runtime.GC()                               // Force un cycle GC
}
```

### Flux de contrôle

```
        Begin()                          End()
           │                                │
           ▼                                ▼
  ┌──────────────────┐           ┌──────────────────┐
  │ ReadMemStats()   │           │ ReadMemStats()   │
  │ SetGCPercent(-1) │           │ SetGCPercent(orig)│
  │ SetMemoryLimit(  │           │ SetMemoryLimit(∞) │
  │   3 × Sys)       │           │ runtime.GC()     │
  └──────────────────┘           └──────────────────┘
           │                                │
           ▼                                ▼
  ┌──────────────────────────────────────────┐
  │         CALCUL SANS GC                    │
  │  ┌─────────────────────────────────────┐ │
  │  │ Allocation depuis arène/pool        │ │
  │  │ Pas de pauses GC                    │ │
  │  │ Limite douce = filet de sécurité    │ │
  │  │ Si limite atteinte → GC d'urgence   │ │
  │  └─────────────────────────────────────┘ │
  └──────────────────────────────────────────┘
```

### Alternative Rust — RAII et Absence de GC

Rust n'a pas de ramasse-miettes. La gestion mémoire est déterministe via RAII (Resource Acquisition Is Initialization). Cependant, des préoccupations analogues existent :

```rust
/// Contrôleur d'allocations pour calculs intensifs.
///
/// En Rust, pas de GC à désactiver. Ce contrôleur gère :
/// - La pré-allocation des arènes
/// - Le monitoring de la mémoire (optionnel)
/// - Les statistiques d'allocation
pub struct MemoryController {
    mode: MemoryMode,
    arena: Option<CalculationArena>,
    start_allocated: usize,
}

#[derive(Debug, Clone, Copy)]
pub enum MemoryMode {
    /// Gestion standard — arènes si n ≥ seuil.
    Auto,
    /// Pré-allocation agressive de tous les buffers.
    Aggressive,
    /// Aucune optimisation mémoire.
    Standard,
}

impl MemoryController {
    pub fn new(mode: MemoryMode, n: u64) -> Self {
        let arena = match mode {
            MemoryMode::Auto if n >= 1_000_000 => Some(CalculationArena::new(n)),
            MemoryMode::Aggressive => Some(CalculationArena::new(n)),
            _ => None,
        };
        Self {
            mode,
            arena,
            start_allocated: Self::current_allocated(),
        }
    }

    fn current_allocated() -> usize {
        // Utiliser jemalloc stats ou std::alloc::GlobalAlloc tracking
        0 // Placeholder
    }
}

impl Drop for MemoryController {
    fn drop(&mut self) {
        // RAII : l'arène et tous ses buffers sont libérés automatiquement
        // Aucune action explicite nécessaire
    }
}
```

### Comparaison des approches

| Aspect              | Go (GCController)             | Rust (RAII)                      |
| ------------------- | ----------------------------- | -------------------------------- |
| Problème résolu   | Pauses GC imprévisibles      | Fragmentation, localité cache   |
| Mécanisme          | `SetGCPercent(-1)` + limite | Arènes +`Drop` automatique    |
| Filet de sécurité | Limite mémoire douce         | OOM killer OS +`try_reserve`   |
| Restauration        | Explicite dans `End()`      | Automatique via `Drop`         |
| Statistiques        | `runtime.MemStats` delta    | `jemalloc_ctl` ou custom alloc |

---

## T4.3 — Pool BigInt avec Classes de Taille (Cycle Acquire/Release, Cap 100M bits, Classes Power-of-4)

### Architecture des pools par classes de taille

Le package `bigfft` implémente 4 familles de pools par classes de taille en puissances de 4 :

```
wordSlicePools ([]big.Word)                   Classes de taille
┌──────────────────────────────────────────────────────────┐
│ Pool[0]: 64 words       (512 B)    ← 4^3               │
│ Pool[1]: 256 words      (2 KB)     ← 4^4               │
│ Pool[2]: 1 024 words    (8 KB)     ← 4^(5)             │
│ Pool[3]: 4 096 words    (32 KB)    ← 4^6               │
│ Pool[4]: 16 384 words   (128 KB)   ← 4^(7)             │
│ Pool[5]: 65 536 words   (512 KB)   ← 4^8               │
│ Pool[6]: 262 144 words  (2 MB)     ← 4^9               │
│ Pool[7]: 1 048 576 words (8 MB)    ← 4^(10)            │
│ Pool[8]: 4 194 304 words (32 MB)   ← 4^(11)            │
│ Pool[9]: 16 777 216 words (128 MB) ← 4^(12)            │
└──────────────────────────────────────────────────────────┘
```

### Sélection O(1) de l'index de pool

```go
// Calcul bitwise O(1) au lieu de recherche linéaire O(n)
func getWordSlicePoolIndex(size int) int {
    if size <= 0 { return 0 }
    if size > wordSliceSizes[len(wordSliceSizes)-1] { return -1 }
    idx := (bits.Len(uint(size-1)) - 5) / 2
    if idx < 0 { idx = 0 }
    return idx
}
```

**Dérivation** : Les tailles sont des puissances de 4 commençant à 4^3 = 64. Pour une taille `s`, l'index est :

```
idx = (⌈log₂(s)⌉ - 5) / 2
    = (bits.Len(s-1) - 5) / 2
```

### Cycle Acquire → Use → Release

```
acquire                    use                      release
┌──────────┐          ┌──────────┐          ┌──────────────────┐
│ Pool.Get()│          │ Écriture │          │ cap(slice) ?     │
│ clear()   │──────►   │ dans le  │──────►   │ → correspond     │
│ [:size]   │          │ slice    │          │   au pool        │
└──────────┘          └──────────┘          │ → Pool.Put([:cap])│
                                             └──────────────────┘
```

### Plafond MaxPooledBitLen = 100M bits

```go
const MaxPooledBitLen = 100_000_000 // 100M bits ≈ 12.5 MB

func ReleaseState(s *CalculationState) {
    if s == nil { return }
    // Éviter de garder des objets trop volumineux en pool
    if checkLimit(s.FK) || checkLimit(s.FK1) ||
       checkLimit(s.T1) || checkLimit(s.T2) || checkLimit(s.T3) {
        return // Laisser le GC récupérer
    }
    statePool.Put(s)
}

func checkLimit(z *big.Int) bool {
    return z != nil && z.BitLen() > MaxPooledBitLen
}
```

### Transposition Rust

```rust
use std::sync::Mutex;

/// Classes de taille pour les pools de buffers.
/// Puissances de 4 : 64, 256, 1024, 4096, 16384, 65536, 262144, 1M, 4M, 16M
const WORD_POOL_SIZES: [usize; 10] = [
    64, 256, 1024, 4096, 16384, 65536, 262144, 1_048_576, 4_194_304, 16_777_216,
];

/// Pool de buffers par classe de taille.
pub struct SizedPool {
    pools: [Mutex<Vec<Vec<u64>>>; 10],
}

impl SizedPool {
    pub fn new() -> Self {
        Self {
            pools: std::array::from_fn(|_| Mutex::new(Vec::new())),
        }
    }

    /// Sélection O(1) de l'index de pool.
    fn pool_index(size: usize) -> Option<usize> {
        if size == 0 { return Some(0); }
        if size > WORD_POOL_SIZES[WORD_POOL_SIZES.len() - 1] { return None; }
        let bits_needed = usize::BITS - (size - 1).leading_zeros();
        let idx = (bits_needed.saturating_sub(5)) as usize / 2;
        Some(idx.min(WORD_POOL_SIZES.len() - 1))
    }

    /// Acquiert un buffer de la classe de taille appropriée.
    pub fn acquire(&self, size: usize) -> Vec<u64> {
        if let Some(idx) = Self::pool_index(size) {
            let mut pool = self.pools[idx].lock().unwrap();
            if let Some(mut buf) = pool.pop() {
                buf.resize(size, 0);
                buf.fill(0); // Clear
                return buf;
            }
            return vec![0u64; WORD_POOL_SIZES[idx]];
        }
        vec![0u64; size] // Trop grand → allocation directe
    }

    /// Libère un buffer dans le pool.
    pub fn release(&self, buf: Vec<u64>) {
        let cap = buf.capacity();
        if let Some(idx) = Self::pool_index(cap) {
            if WORD_POOL_SIZES[idx] == cap {
                let mut pool = self.pools[idx].lock().unwrap();
                pool.push(buf);
            }
            // Sinon → Drop automatique
        }
    }
}
```

**Note Rust** : Les `Mutex<Vec<Vec<u64>>>` sont moins performants que `sync.Pool` en Go (qui est intégré au runtime). Alternative : utiliser `crossbeam` ou `object-pool` crate pour des performances comparables.

---

## T4.4 — Pré-chauffage des Pools (Prédiction de Taille, Seuils Adaptatifs)

### Objectif

Pré-allouer les buffers dans les pools **avant** le calcul pour éviter les allocations pendant la boucle de calcul hot-path.

### Implémentation Go

```go
func PreWarmPools(n uint64) {
    est := EstimateMemoryNeeds(n)

    // Nombre de buffers adaptatif selon la taille du calcul
    numBuffers := 2    // Défaut pour petits calculs
    if n >= 10_000_000 {
        numBuffers = 6
    } else if n >= 1_000_000 {
        numBuffers = 5
    } else if n >= 100_000 {
        numBuffers = 4
    }

    // Pré-chauffage des 4 familles de pools
    // wordSlicePools, fermatPools, natSlicePools, fermatSlicePools
    // ...
}
```

### Table des seuils

| Plage de n     | numBuffers | Justification                            |
| -------------- | :--------: | ---------------------------------------- |
| n < 100 000    |     2     | Overhead minimal                         |
| 100K ≤ n < 1M |     4     | Quelques buffers nécessaires            |
| 1M ≤ n < 10M  |     5     | FFT activé, plus de buffers temporaires |
| n ≥ 10M       |     6     | Maximum pour calculs très lourds        |

### Mécanisme d'initialisation unique

```go
var poolsWarmed atomic.Bool

func EnsurePoolsWarmed(maxN uint64) {
    if poolsWarmed.CompareAndSwap(false, true) {
        PreWarmPools(maxN)
    }
}
```

### Transposition Rust

```rust
use std::sync::Once;

static POOLS_WARMED: Once = Once::new();

/// Assure le pré-chauffage des pools exactement une fois.
pub fn ensure_pools_warmed(max_n: u64) {
    POOLS_WARMED.call_once(|| {
        pre_warm_pools(max_n);
    });
}

fn pre_warm_pools(n: u64) {
    let est = estimate_memory_needs(n);
    let num_buffers = match n {
        n if n >= 10_000_000 => 6,
        n if n >= 1_000_000 => 5,
        n if n >= 100_000 => 4,
        _ => 2,
    };

    let pool = global_pool();
    for _ in 0..num_buffers {
        let buf = vec![0u64; est.max_word_slice_size];
        pool.release(buf);
        // ... idem pour les autres familles
    }
}
```

---

## T4.5 — Allocateur Bump FFT (O(1) Pointer Bump, Mapping bumpalo::Bump)

### Architecture

Le `BumpAllocator` du package `bigfft` fournit des allocations O(1) pour les buffers temporaires FFT. Contrairement au `CalculationArena` (niveau Fibonacci), il est spécialisé pour les opérations FFT et est distribué via `sync.Pool`.

### Implémentation Go

```go
type BumpAllocator struct {
    buffer []big.Word
    offset int
}

func (ba *BumpAllocator) Alloc(n int) []big.Word {
    if ba.offset+n > len(ba.buffer) {
        return make([]big.Word, n) // Fallback
    }
    slice := ba.buffer[ba.offset : ba.offset+n]
    ba.offset += n
    clear(slice) // Zéro pour sécurité
    return slice
}
```

### Flux d'allocation

```
BumpAllocator.buffer (pré-alloué)
┌─────────────────────────────────────────────────────┐
│ fermat[0]  │ fermat[1]  │ fermat[2]  │ tmp │ tmp2  │
│ (n+1 words)│ (n+1 words)│ (n+1 words)│     │       │
└─────────────────────────────────────────────────────┘
              ↑ offset avance →
```

### Interface TempAllocator — Stratégie d'allocation

```go
type TempAllocator interface {
    AllocFermatTemp(n int) (fermat, func())
    AllocFermatSlice(K, n int) ([]fermat, []big.Word, func())
}

// Deux implémentations :
// 1. PoolAllocator — utilise sync.Pool, cleanup retourne au pool
// 2. BumpAllocatorAdapter — utilise bump allocator, cleanup = no-op
```

### Estimation de capacité

```go
func EstimateBumpCapacity(wordLen int) int {
    // K = nombre de coefficients FFT ≈ 2*sqrt(bits)
    // Transform temp: K × (n+1) words
    // Multiply temp: 8 × n words
    // Total: (2×transformTemp + multiplyTemp) × 1.1  (marge 10%)
    // ...
}
```

### Transposition Rust

```rust
use bumpalo::Bump;

/// Allocateur bump pour les opérations FFT.
/// Chaque goroutine/thread possède sa propre instance.
pub struct FftBumpAllocator {
    bump: Bump,
}

impl FftBumpAllocator {
    /// Crée un allocateur dimensionné pour des opérations sur `word_len` mots.
    pub fn with_capacity(word_len: usize) -> Self {
        let capacity = estimate_bump_capacity(word_len);
        Self {
            bump: Bump::with_capacity(capacity * std::mem::size_of::<u64>()),
        }
    }

    /// Alloue un slice de `n` mots — O(1) par bump de pointeur.
    pub fn alloc_words(&self, n: usize) -> &mut [u64] {
        self.bump.alloc_slice_fill_default(n)
    }

    /// Alloue un buffer Fermat de taille n+1.
    pub fn alloc_fermat(&self, n: usize) -> &mut [u64] {
        self.alloc_words(n + 1)
    }

    /// Réinitialise l'allocateur — O(1).
    pub fn reset(&mut self) {
        self.bump.reset();
    }
}

/// Pool thread-local d'allocateurs bump.
thread_local! {
    static FFT_BUMP: std::cell::RefCell<Option<FftBumpAllocator>> =
        std::cell::RefCell::new(None);
}

/// Acquiert un allocateur bump pour le thread courant.
pub fn with_fft_bump<F, R>(word_len: usize, f: F) -> R
where
    F: FnOnce(&FftBumpAllocator) -> R,
{
    FFT_BUMP.with(|cell| {
        let mut opt = cell.borrow_mut();
        let alloc = opt.get_or_insert_with(|| {
            FftBumpAllocator::with_capacity(word_len)
        });
        alloc.reset();
        f(alloc)
    })
}
```

**Avantage Rust `thread_local!`** : Élimine la contention du `sync.Pool` en Go. Chaque thread possède son allocateur bump, zéro synchronisation nécessaire.

---

## T4.6 — Validation du Budget Mémoire (Formules d'Estimation)

### Structure MemoryEstimate

```go
type MemoryEstimate struct {
    StateBytes     uint64 // 5 big.Int temporaires de CalculationState
    FFTBufferBytes uint64 // Allocateur bump + buffers FFT
    CacheBytes     uint64 // Cache de transformées FFT
    OverheadBytes  uint64 // GC + runtime overhead
    TotalBytes     uint64
}
```

### Formules d'estimation

```
Taille de F(n) en bits = n × 0.69424 (log₂(φ))
Words par big.Int      = ⌈n × 0.69424 / 64⌉ + 1
Bytes par big.Int      = words × 8

Composantes :
  StateBytes    = bytesPerFib × 5    (FK, FK1, T1, T2, T3)
  FFTBufferBytes = bytesPerFib × 3    (allocateur bump)
  CacheBytes     = bytesPerFib × 2    (cache transformées)
  OverheadBytes  = stateBytes × 1     (GC + runtime ≈ 1×)

  TotalBytes = State + FFT + Cache + Overhead
             = bytesPerFib × (5 + 3 + 2 + 5)
             = bytesPerFib × 11
```

### Exemples numériques

|             n |   F(n) bits | Bytes/big.Int |  State |    FFT |  Cache | Overhead |    **Total** |
| ------------: | ----------: | ------------: | -----: | -----: | -----: | -------: | -----------------: |
|     1 000 000 |     694 240 |        ~86 KB | 430 KB | 258 KB | 172 KB |   430 KB |  **~1.3 MB** |
|    10 000 000 |   6 942 400 |       ~868 KB | 4.2 MB | 2.5 MB | 1.7 MB |   4.2 MB | **~12.6 MB** |
|   100 000 000 |  69 424 000 |       ~8.5 MB |  42 MB |  25 MB |  17 MB |    42 MB |  **~126 MB** |
| 1 000 000 000 | 694 240 000 |        ~85 MB | 425 MB | 255 MB | 170 MB |   425 MB |  **~1.3 GB** |

### Transposition Rust

```rust
/// Estimation de la mémoire nécessaire pour calculer F(n).
#[derive(Debug, Clone)]
pub struct MemoryEstimate {
    pub state_bytes: u64,
    pub fft_buffer_bytes: u64,
    pub cache_bytes: u64,
    pub overhead_bytes: u64,
    pub total_bytes: u64,
}

pub fn estimate_memory_usage(n: u64) -> MemoryEstimate {
    let bits_per_fib = n as f64 * 0.69424;
    let words_per_fib = (bits_per_fib / 64.0) as u64 + 1;
    let bytes_per_fib = words_per_fib * 8;

    let state_bytes = bytes_per_fib * 5;
    let fft_bytes = bytes_per_fib * 3;
    let cache_bytes = bytes_per_fib * 2;
    // En Rust, pas de GC → overhead réduit (~0.5× au lieu de 1×)
    let overhead_bytes = bytes_per_fib * 2; // allocateur + fragmentation

    let total = state_bytes + fft_bytes + cache_bytes + overhead_bytes;
    MemoryEstimate {
        state_bytes,
        fft_buffer_bytes: fft_bytes,
        cache_bytes,
        overhead_bytes,
        total_bytes: total,
    }
}
```

---

## T4.7 — Estimation Mémoire FFT (Transformée + Temporaires + Pool)

### Architecture mémoire FFT

```
Mémoire FFT pour une multiplication
┌────────────────────────────────────────────────────────┐
│ 1. Coefficients d'entrée : K × (n+1) words            │
│    → Polynôme a: K coefficients de type fermat(n+1)    │
│                                                        │
│ 2. Valeurs transformées : K × (n+1) words              │
│    → polValues: K valeurs fermat après NTT              │
│                                                        │
│ 3. Temporaires de récursion : 2 × (n+1) words          │
│    → tmp, tmp2 dans fftState                           │
│                                                        │
│ 4. Buffer de multiplication : 8 × n words              │
│    → Temporaire pour mul/sqr fermat                    │
│                                                        │
│ 5. Buffer de résultat : 2 × wordLen words              │
│    → Résultat de la multiplication                     │
└────────────────────────────────────────────────────────┘
```

### Formule d'estimation (bigfft/memory_est.go)

```go
func EstimateMemoryNeeds(n uint64) MemoryEstimate {
    bitLen := uint64(float64(n) * 0.69424)
    wordLen := int((bitLen + 63) / 64)

    maxWordSlice := wordLen * 2
    maxFermat := estimateFermatSize(wordLen)
    maxNatSlice := estimateSliceCount(wordLen)
    maxFermatSlice := estimateSliceCount(wordLen)

    return MemoryEstimate{
        MaxWordSliceSize:   maxWordSlice,
        MaxFermatSize:      maxFermat,
        MaxNatSliceSize:    maxNatSlice,
        MaxFermatSliceSize: maxFermatSlice,
    }
}
```

### Estimation de capacité du bump allocator

```go
func EstimateBumpCapacity(wordLen int) int {
    bits := wordLen * _W
    k := determineFftSize(bits)  // K = 2^k
    K := 1 << k
    n := wordLen/K + 1

    transformTemp := K * (n + 1)    // Buffers de transformée
    multiplyTemp := 8 * n           // Buffers de multiplication

    // Marge de sécurité de 10%
    total := (2*transformTemp + multiplyTemp) * 11 / 10
    return total
}
```

### Transposition Rust

```rust
/// Estimation de la mémoire FFT pour des nombres de `word_len` mots.
pub fn estimate_fft_memory(word_len: usize) -> FftMemoryEstimate {
    let bits = word_len * 64;
    let k = determine_fft_size(bits);
    let big_k = 1usize << k;
    let n = word_len / big_k + 1;

    let transform_temp = big_k * (n + 1);
    let multiply_temp = 8 * n;
    let result_buffer = word_len * 2;

    FftMemoryEstimate {
        coefficients_words: big_k * (n + 1),
        values_words: big_k * (n + 1),
        recursion_temp_words: 2 * (n + 1),
        multiply_temp_words: multiply_temp,
        result_words: result_buffer,
        bump_capacity_words: (2 * transform_temp + multiply_temp) * 11 / 10,
    }
}
```

---

## T4.8 — Pooling de CalculationState/matrixState (Cycle Acquire → Reset → Use → Return)

### Cycle de vie complet

```
          ┌──────────────┐
          │  sync.Pool   │
          │ (statePool)  │
          └──────┬───────┘
                 │
          AcquireState()
          Pool.Get() → *CalculationState
          Reset() : FK=0, FK1=1
                 │
                 ▼
          ┌──────────────────┐
          │  UTILISATION      │
          │  FK, FK1 → calcul │
          │  T1, T2, T3 temp │
          └──────────────────┘
                 │
          ReleaseState(s)
          checkLimit() sur les 5 big.Int
                 │
         ┌───────┴──────────┐
         │                  │
    BitLen ≤ 100M      BitLen > 100M
    bits chacun         bits (un ou +)
         │                  │
    Pool.Put(s)        s abandonné
    (réutilisable)     (GC récupère)
         │                  │
         ▼                  ▼
    ┌─────────┐       ┌──────────┐
    │  Pool   │       │   GC     │
    └─────────┘       └──────────┘
```

### Implémentation Go — CalculationState

```go
type CalculationState struct {
    FK, FK1, T1, T2, T3 *big.Int
}

var statePool = sync.Pool{
    New: func() any {
        return &CalculationState{
            FK:  new(big.Int),
            FK1: new(big.Int),
            T1:  new(big.Int),
            T2:  new(big.Int),
            T3:  new(big.Int),
        }
    },
}

func AcquireState() *CalculationState {
    s := statePool.Get().(*CalculationState)
    s.Reset() // FK=0, FK1=1
    return s
}

func ReleaseState(s *CalculationState) {
    if s == nil { return }
    if checkLimit(s.FK) || checkLimit(s.FK1) ||
       checkLimit(s.T1) || checkLimit(s.T2) || checkLimit(s.T3) {
        return // Trop gros → laisser le GC
    }
    statePool.Put(s)
}
```

### Optimisation Zero-Copy Result Return

```go
// Dans ExecuteDoublingLoop — fin de la boucle
result := s.FK           // "Vol" du pointeur FK
s.FK = new(big.Int)      // Remplacement par un big.Int vide
return result, nil        // Le state reste valide pour le pool
```

**Coût** : 1 allocation de header `big.Int` (24 octets) au lieu d'une copie O(n) mots.

### Transposition Rust

```rust
use std::sync::Mutex;

/// État de calcul réutilisable.
pub struct CalculationState {
    pub fk: BigUint,
    pub fk1: BigUint,
    pub t1: BigUint,
    pub t2: BigUint,
    pub t3: BigUint,
}

impl CalculationState {
    pub fn new() -> Self {
        Self {
            fk: BigUint::ZERO,
            fk1: BigUint::from(1u32),
            t1: BigUint::ZERO,
            t2: BigUint::ZERO,
            t3: BigUint::ZERO,
        }
    }

    pub fn reset(&mut self) {
        self.fk = BigUint::ZERO;
        self.fk1 = BigUint::from(1u32);
        // T1..T3 : pas besoin de clear (écrasés avant lecture)
    }
}

/// Pool d'états via Mutex<Vec>.
/// Alternative : crossbeam ObjectPool pour de meilleures performances.
pub struct StatePool {
    pool: Mutex<Vec<CalculationState>>,
    max_bit_len: usize,
}

impl StatePool {
    pub fn new() -> Self {
        Self {
            pool: Mutex::new(Vec::new()),
            max_bit_len: 100_000_000, // 100M bits
        }
    }

    pub fn acquire(&self) -> CalculationState {
        let mut pool = self.pool.lock().unwrap();
        match pool.pop() {
            Some(mut state) => {
                state.reset();
                state
            }
            None => CalculationState::new(),
        }
    }

    pub fn release(&self, state: CalculationState) {
        // Vérification de la taille avant de retourner au pool
        if state.fk.bits() > self.max_bit_len as u64
            || state.fk1.bits() > self.max_bit_len as u64
        {
            return; // Drop automatique
        }
        let mut pool = self.pool.lock().unwrap();
        pool.push(state);
    }
}
```

**Zero-Copy en Rust** :

```rust
// std::mem::take remplace la valeur par Default et retourne l'originale
let result = std::mem::take(&mut state.fk); // fk devient BigUint::ZERO
return Ok(result);
```

---

## T4.9 — Modèle de Concurrence Complet (4 Patrons + Mapping Go → Rust)

### Les 4 patrons de concurrence de FibGo

```
┌─────────────────────────────────────────────────────────────────────┐
│                    MODÈLE DE CONCURRENCE FIBGO                      │
├────────────────┬──────────────────┬─────────────────────────────────┤
│ Patron         │ Utilisation      │ Fichiers Go                     │
├────────────────┼──────────────────┼─────────────────────────────────┤
│ 1. errgroup    │ Orchestration    │ orchestration/orchestrator.go   │
│                │ (N calculateurs  │                                 │
│                │  en parallèle)   │                                 │
├────────────────┼──────────────────┼─────────────────────────────────┤
│ 2. WaitGroup + │ Multiplications  │ fibonacci/doubling_framework.go │
│    Semaphore   │ parallèles dans  │ fibonacci/common.go             │
│                │ la boucle de     │                                 │
│                │ doubling         │                                 │
├────────────────┼──────────────────┼─────────────────────────────────┤
│ 3. Canaux      │ Communication    │ fibonacci/observers.go          │
│    tamponnés   │ progression      │ orchestration/interfaces.go     │
│                │ (observer → UI)  │                                 │
├────────────────┼──────────────────┼─────────────────────────────────┤
│ 4. sync.Pool   │ Réutilisation    │ fibonacci/fastdoubling.go       │
│                │ mémoire          │ bigfft/pool.go                  │
│                │ (BigInt, buffers) │ bigfft/bump.go                 │
└────────────────┴──────────────────┴─────────────────────────────────┘
```

### Mapping détaillé Go → Rust

#### Patron 1 : errgroup → rayon/tokio

```
Go:  errgroup.Group avec contexte
     g.Go(func() error { calc1.Calculate(...) })
     g.Go(func() error { calc2.Calculate(...) })
     g.Go(func() error { calc3.Calculate(...) })
     g.Wait()

Rust Option A (rayon — recommandé pour CPU-bound):
     rayon::scope(|s| {
         s.spawn(|_| calc1.calculate(&ctx));
         s.spawn(|_| calc2.calculate(&ctx));
         s.spawn(|_| calc3.calculate(&ctx));
     });

Rust Option B (tokio::task::spawn_blocking):
     let handles = calcs.iter().map(|c| {
         tokio::task::spawn_blocking(move || c.calculate(&ctx))
     }).collect::<Vec<_>>();
     futures::future::try_join_all(handles).await?;
```

#### Patron 2 : WaitGroup + Semaphore → rayon::scope + parking_lot

```
Go:  var wg sync.WaitGroup
     sem := make(chan struct{}, NumCPU*2)
     wg.Add(3)
     go func() { sem <- struct{}{}; defer func(){ <-sem }(); ... }()
     wg.Wait()

Rust (rayon — work-stealing implicite, pas de sémaphore nécessaire):
     rayon::scope(|s| {
         s.spawn(|_| { t3 = strategy.multiply(fk, fk1); });
         s.spawn(|_| { t1 = strategy.square(fk1); });
         s.spawn(|_| { t2 = strategy.square(fk); });
     });
     // rayon gère automatiquement le pool de threads
```

#### Patron 3 : Canaux tamponnés → crossbeam::channel

```
Go:  ch := make(chan ProgressUpdate, 100)
     select { case ch <- update: default: }

Rust: let (tx, rx) = crossbeam::channel::bounded(100);
      let _ = tx.try_send(update); // Non-bloquant
```

#### Patron 4 : sync.Pool → crossbeam ObjectPool / thread_local

```
Go:  var pool = sync.Pool{New: func() any { return &State{} }}
     s := pool.Get().(*State)
     defer pool.Put(s)

Rust Option A (crossbeam):
     static POOL: Lazy<ObjectPool<State>> = Lazy::new(|| {
         ObjectPool::new(State::new, |s| s.reset())
     });
     let guard = POOL.pull();

Rust Option B (thread_local):
     thread_local! { static POOL: RefCell<Vec<State>> = RefCell::new(vec![]); }
```

### Tableau récapitulatif

| Patron Go                     | Équivalent Rust                   | Crate                 |
| ----------------------------- | ---------------------------------- | --------------------- |
| `errgroup.Group`            | `rayon::scope`                   | `rayon`             |
| `sync.WaitGroup`            | `rayon::scope`                   | `rayon`             |
| `chan struct{}`(sémaphore) | Implicite (rayon threadpool)       | `rayon`             |
| `chan T` (tamponné)        | `crossbeam::channel::bounded`    | `crossbeam-channel` |
| `select { default: }`       | `try_send()`                     | `crossbeam-channel` |
| `sync.Pool`                 | `object-pool` / `thread_local` | `object-pool`       |
| `sync.RWMutex`              | `parking_lot::RwLock`            | `parking_lot`       |
| `sync.Once`                 | `std::sync::Once`                | stdlib                |
| `atomic.Bool`               | `AtomicBool`                     | stdlib                |
| `context.Context`           | `Arc<AtomicBool>` + tokio        | stdlib /`tokio`     |

---

## T4.10 — Sémaphore de Tâches et Exécution Générique (2×NumCPU, Patron Pointer Constraint)

### Sémaphore basée sur canal

```go
// Canal tamponné de capacité 2×NumCPU — sert de sémaphore
var taskSemaphore chan struct{}
var taskSemaphoreOnce sync.Once

func getTaskSemaphore() chan struct{} {
    taskSemaphoreOnce.Do(func() {
        taskSemaphore = make(chan struct{}, runtime.NumCPU()*2)
    })
    return taskSemaphore
}
```

**Dimensionnement** : `2 × NumCPU` goroutines maximum pour les multiplications Fibonacci. Le facteur 2 compense le fait que les goroutines bloquent sur les opérations big.Int.

### Exécution générique avec contrainte de pointeur

```go
type task interface {
    execute() error
}

func executeTasks[T any, PT interface {
    *T
    task
}](tasks []T, inParallel bool) error {
    if inParallel {
        sem := getTaskSemaphore()
        var wg sync.WaitGroup
        var ec parallel.ErrorCollector
        wg.Add(len(tasks))
        for i := range tasks {
            go func(t PT) {
                defer wg.Done()
                sem <- struct{}{}       // Acquérir token
                defer func() { <-sem }() // Libérer token
                ec.SetError(t.execute())
            }(PT(&tasks[i]))
        }
        wg.Wait()
        return ec.Err()
    }
    // Exécution séquentielle
    for i := range tasks {
        if err := PT(&tasks[i]).execute(); err != nil {
            return err
        }
    }
    return nil
}
```

### Patron Pointer Constraint — Explication

```
executeTasks[T any, PT interface{ *T; task }]

T  = type valeur (multiplicationTask ou squaringTask)
PT = type pointeur vers T qui implémente task

Exemple d'instanciation :
  executeTasks[multiplicationTask, *multiplicationTask](tasks, true)
  executeTasks[squaringTask, *squaringTask](tasks, true)

Avantage :
  - tasks est un []T (valeurs) → pas d'allocations tas pour les tâches
  - PT(&tasks[i]) crée un pointeur vers le slice → pas de copie
  - Une seule implémentation pour N types de tâches
```

### Transposition Rust

```rust
use rayon::prelude::*;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Trait pour les tâches exécutables.
pub trait Task: Send {
    fn execute(&mut self) -> Result<(), FibError>;
}

/// Sémaphore basée sur AtomicUsize (pour limiter la concurrence).
pub struct Semaphore {
    permits: AtomicUsize,
    max_permits: usize,
}

impl Semaphore {
    pub fn new(max: usize) -> Self {
        Self {
            permits: AtomicUsize::new(max),
            max_permits: max,
        }
    }

    pub fn acquire(&self) {
        loop {
            let current = self.permits.load(Ordering::Acquire);
            if current > 0 {
                if self.permits.compare_exchange_weak(
                    current, current - 1,
                    Ordering::AcqRel, Ordering::Relaxed
                ).is_ok() {
                    return;
                }
            }
            std::hint::spin_loop();
        }
    }

    pub fn release(&self) {
        self.permits.fetch_add(1, Ordering::Release);
    }
}

/// Exécute les tâches en parallèle ou séquentiellement.
/// Utilise rayon pour le parallélisme, pas besoin de sémaphore
/// explicite (le thread pool rayon gère la concurrence).
pub fn execute_tasks<T: Task + Send>(
    tasks: &mut [T],
    in_parallel: bool,
) -> Result<(), FibError> {
    if in_parallel {
        // rayon parallélise automatiquement avec son thread pool
        tasks.par_iter_mut()
            .try_for_each(|task| task.execute())
    } else {
        for task in tasks.iter_mut() {
            task.execute()?;
        }
        Ok(())
    }
}
```

**Simplification Rust** : Avec `rayon`, le sémaphore est inutile — le thread pool work-stealing gère automatiquement la concurrence. La taille du pool par défaut est `num_cpus`, modifiable via `rayon::ThreadPoolBuilder`.

---

## T4.11 — Patron de Collection d'Erreurs Parallèles (ErrorCollector, First-Error, Atomics)

### Sémantique First-Error

L'`ErrorCollector` capture uniquement la **première** erreur survenue parmi N goroutines parallèles. Les erreurs ultérieures sont ignorées (patron "first error wins").

### Implémentation Go

```go
type ErrorCollector struct {
    once sync.Once
    err  error
}

func (c *ErrorCollector) SetError(err error) {
    if err != nil {
        c.once.Do(func() {
            c.err = err
        })
    }
}

func (c *ErrorCollector) Err() error {
    return c.err
}
```

### Analyse du mécanisme sync.Once

```
Goroutine 1         Goroutine 2         Goroutine 3
     │                    │                    │
 SetError(err1)      SetError(nil)       SetError(err3)
     │                    │                    │
 once.Do(f) ←────── race condition ──────► once.Do(f)
     │                    │                    │
 f() exécuté         nil → ignoré        once.Do → no-op
 c.err = err1                             (déjà fait)
     │                    │                    │
     ▼                    ▼                    ▼
                   c.Err() → err1
```

### Transposition Rust

```rust
use std::sync::OnceLock;

/// Collecteur first-error pour goroutines/threads parallèles.
pub struct ErrorCollector {
    error: OnceLock<FibError>,
}

impl ErrorCollector {
    pub fn new() -> Self {
        Self {
            error: OnceLock::new(),
        }
    }

    /// Enregistre la première erreur non-nil.
    pub fn set_error(&self, err: FibError) {
        let _ = self.error.set(err); // Ignore si déjà défini
    }

    /// Retourne la première erreur enregistrée.
    pub fn err(&self) -> Option<&FibError> {
        self.error.get()
    }
}
```

**Alternative Rust plus idiomatique** :

```rust
/// Avec rayon, le patron standard est try_for_each qui propage
/// automatiquement la première erreur :
tasks.par_iter_mut()
    .try_for_each(|task| task.execute())?;
// ^ Retourne Err(first_error) dès qu'une tâche échoue
```

**Recommandation** : En Rust, préférer les combinateurs `try_*` de rayon/itertools qui implémentent nativement la sémantique first-error, plutôt qu'un `ErrorCollector` explicite.

---

## T4.12 — Protocole d'Annulation Coopérative (Checkpoints, Propagation d'Erreur, Arc`<AtomicBool>`)

### Modèle Go — context.Context

La boucle de calcul vérifie `ctx.Err()` à chaque itération (checkpoint) :

```go
func (f *DoublingFramework) ExecuteDoublingLoop(ctx context.Context, ...) (*big.Int, error) {
    for i := numBits - 1; i >= 0; i-- {
        // ──── CHECKPOINT D'ANNULATION ────
        if err := ctx.Err(); err != nil {
            return nil, fmt.Errorf(
                "fast doubling calculation canceled at bit %d/%d: %w",
                i, numBits-1, err,
            )
        }

        // ... calcul ...

        // Aussi dans les multiplications parallèles :
        go func() {
            if err := ctx.Err(); err != nil {
                ec.SetError(fmt.Errorf("canceled before multiply: %w", err))
                return
            }
            // ... multiplication ...
        }()
    }
}
```

### Points de checkpoint dans le code Go

```
ExecuteDoublingLoop
    │
    ├── for i := numBits-1; i >= 0; i--
    │     │
    │     ├── CHECKPOINT 1: ctx.Err() avant le doubling step
    │     │
    │     ├── executeDoublingStepMultiplications
    │     │     │
    │     │     ├── [parallèle] CHECKPOINT 2a: ctx.Err() avant multiply FK×FK1
    │     │     ├── [parallèle] CHECKPOINT 2b: ctx.Err() avant square FK1²
    │     │     ├── [parallèle] CHECKPOINT 2c: ctx.Err() avant square FK²
    │     │     │
    │     │     ├── [séquentiel] CHECKPOINT 3a: après multiply
    │     │     └── [séquentiel] CHECKPOINT 3b: après square FK1²
    │     │
    │     └── Post-multiply + addition step
    │
    └── return result, nil
```

### Propagation d'erreur

```
ctx.cancel() appelé (timeout, signal, ou annulation manuelle)
     │
     ▼
ctx.Err() retourne context.Canceled ou context.DeadlineExceeded
     │
     ├─── Boucle principale : return nil, fmt.Errorf("...canceled...: %w", err)
     │
     ├─── Goroutine parallèle : ec.SetError(fmt.Errorf("canceled: %w", err))
     │                           return (goroutine se termine)
     │
     └─── Propagation remonte jusqu'à :
          CalculateWithObservers → Calculate → orchestration → app → exit code 130
```

### Transposition Rust — Arc`<AtomicBool>` + CancellationToken

```rust
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

/// Token d'annulation partagé entre threads.
#[derive(Clone)]
pub struct CancellationToken {
    cancelled: Arc<AtomicBool>,
}

impl CancellationToken {
    pub fn new() -> Self {
        Self {
            cancelled: Arc::new(AtomicBool::new(false)),
        }
    }

    /// Déclenche l'annulation.
    pub fn cancel(&self) {
        self.cancelled.store(true, Ordering::Release);
    }

    /// Vérifie si l'annulation a été demandée.
    #[inline]
    pub fn is_cancelled(&self) -> bool {
        self.cancelled.load(Ordering::Acquire)
    }

    /// Retourne Err si annulé, Ok(()) sinon.
    /// Point de checkpoint dans les boucles de calcul.
    #[inline]
    pub fn check(&self) -> Result<(), FibError> {
        if self.is_cancelled() {
            Err(FibError::Cancelled)
        } else {
            Ok(())
        }
    }
}
```

### Utilisation dans la boucle de calcul Rust

```rust
pub fn execute_doubling_loop(
    token: &CancellationToken,
    reporter: &dyn Fn(f64),
    n: u64,
    opts: &Options,
    state: &mut CalculationState,
    use_parallel: bool,
) -> Result<BigUint, FibError> {
    let num_bits = 64 - n.leading_zeros() as usize;
    let total_work = calc_total_work(num_bits);
    let powers = precompute_powers4(num_bits);
    let mut work_done = 0.0;
    let mut last_reported = -1.0;

    for i in (0..num_bits).rev() {
        // ──── CHECKPOINT D'ANNULATION ────
        token.check().map_err(|e| {
            FibError::CalculationCancelled {
                bit: i,
                total_bits: num_bits,
                source: Box::new(e),
            }
        })?;

        // Doubling step
        if use_parallel && should_parallelize(state, opts) {
            // rayon::scope avec clones du token pour les sous-tâches
            let token = token.clone();
            rayon::scope(|s| {
                s.spawn(|_| {
                    if token.is_cancelled() { return; }
                    // multiply FK × FK1 → T3
                });
                s.spawn(|_| {
                    if token.is_cancelled() { return; }
                    // square FK1 → T1
                });
                s.spawn(|_| {
                    if token.is_cancelled() { return; }
                    // square FK → T2
                });
            });
        } else {
            // Exécution séquentielle avec checkpoints entre les multiplications
            state.t3 = strategy.multiply(&state.fk, &state.fk1)?;
            token.check()?;
            state.t1 = strategy.square(&state.fk1)?;
            token.check()?;
            state.t2 = strategy.square(&state.fk)?;
        }

        // Post-multiply
        // F(2k) = 2·FK·FK1 - FK² = 2·T3 - T2
        // F(2k+1) = FK1² + FK² = T1 + T2
        // ... pointer swaps ...

        // Progression
        work_done = report_step_progress(
            reporter, &mut last_reported,
            total_work, work_done, i, num_bits, powers,
        );
    }

    // Zero-copy result
    Ok(std::mem::take(&mut state.fk))
}
```

### Intégration avec les signaux OS

```rust
use signal_hook::consts::SIGINT;
use signal_hook::flag;

fn setup_signal_handler(token: &CancellationToken) {
    // Le flag SIGINT active l'AtomicBool du token
    flag::register(SIGINT, token.cancelled.clone())
        .expect("Failed to register SIGINT handler");
}
```

### Comparaison Go context.Context vs Rust CancellationToken

| Aspect                  | Go (context.Context)     | Rust (CancellationToken)          |
| ----------------------- | ------------------------ | --------------------------------- |
| Mécanisme              | Channel interne + Mutex  | AtomicBool                        |
| Coût du check          | ~10 ns (lecture channel) | ~1 ns (load atomique)             |
| Timeout intégré       | `context.WithTimeout`  | Séparé (`tokio::time`)        |
| Héritage parent-enfant | Oui (`WithCancel`)     | Manuel (`clone()`)              |
| Valeurs attachées      | `context.WithValue`    | Non (type-safe séparé)          |
| Propagation d'erreur    | `ctx.Err()` → error   | `check()` → Result<(), Error>` |
| Signal OS               | `signal.NotifyContext` | `signal_hook::flag`             |

### Diagramme de propagation d'annulation Rust

```
Signal SIGINT ─────────────►  CancellationToken.cancelled = true
                                       │
    ┌──────────────────────────────────┤
    │                                  │
    ▼                                  ▼
 Boucle principale               rayon::scope
 token.check()?                   ├── spawn: token.is_cancelled() → return
     │                            ├── spawn: token.is_cancelled() → return
     ▼                            └── spawn: token.is_cancelled() → return
 Err(FibError::Cancelled)
     │
     ▼
 execute_doubling_loop retourne Err
     │
     ▼
 calculate retourne Err
     │
     ▼
 main → process::exit(130)
```

# Phase 5 — Seuils Dynamiques & Calibration

> Portage de `internal/fibonacci/dynamic_threshold.go`, `internal/fibonacci/threshold_types.go`, et de `internal/calibration/*` vers Rust.

---

## T5.1 — Architecture du DynamicThresholdManager

### Objectif

Porter le `DynamicThresholdManager` (Go : `internal/fibonacci/dynamic_threshold.go`) vers une structure Rust thread-safe qui ajuste les seuils FFT et parallélisme pendant le calcul, sur la base de métriques collectées par itération.

### Structure Go d'origine

```go
type DynamicThresholdManager struct {
    mu sync.RWMutex

    currentFFTThreshold      int
    currentParallelThreshold int
    originalFFTThreshold      int
    originalParallelThreshold int

    // Ring buffer — tableau fixe [20]IterationMetric
    metrics      [MaxMetricsHistory]IterationMetric
    metricsCount int  // total jamais-décroissant
    metricsHead  int  // prochain slot d'écriture

    iterationCount     int
    adjustmentInterval int
    lastAdjustment     time.Time
}
```

### Spécification Rust

```rust
use std::sync::RwLock;
use std::time::{Duration, Instant};

/// Capacité maximale du ring buffer de métriques.
const MAX_METRICS_HISTORY: usize = 20;

/// Intervalle d'itérations entre vérifications de seuil.
const DYNAMIC_ADJUSTMENT_INTERVAL: usize = 5;

/// Nombre minimal de métriques avant tout ajustement.
const MIN_METRICS_FOR_ADJUSTMENT: usize = 3;

pub struct DynamicThresholdManager {
    /// Seuils courants (modifiables pendant le calcul).
    current_fft_threshold: RwLock<i32>,
    current_parallel_threshold: RwLock<i32>,

    /// Seuils originaux (pour bornes et comparaison).
    original_fft_threshold: i32,
    original_parallel_threshold: i32,

    /// Ring buffer à capacité fixe — pas d'allocation dynamique.
    metrics: [IterationMetric; MAX_METRICS_HISTORY],
    metrics_count: usize,
    metrics_head: usize,

    /// État d'ajustement.
    iteration_count: usize,
    adjustment_interval: usize,
    last_adjustment: Option<Instant>,
}
```

### Diagramme du Ring Buffer

```
Capacité = 20 slots fixes (pas de Vec, pas de VecDeque)

Index:  [0] [1] [2] [3] [4] [5] ... [18] [19]
         ↑                                  ↑
     plus ancien                     metrics_head
     (si buffer plein)          (prochain slot d'écriture)

Écriture : metrics[metrics_head] = new_metric
            metrics_head = (metrics_head + 1) % 20
            metrics_count += 1

Lecture : si metrics_count <= 20 → copier metrics[0..metrics_count]
          si metrics_count >  20 → copier metrics[0..20] (ordre non garanti,
                                   acceptable car seules les moyennes comptent)
```

### Protocole de concurrence

| Opération             | Thread                      | Mutex requis                            |
| ---------------------- | --------------------------- | --------------------------------------- |
| `record_iteration()` | Thread calcul (unique)      | Aucun — accès single-writer           |
| `should_adjust()`    | Thread calcul (unique)      | Aucun — appelé depuis la même boucle |
| `get_thresholds()`   | Threads multiples (lecture) | `RwLock::read()`                      |
| `get_stats()`        | Thread UI / métriques      | `RwLock::read()`                      |
| `reset()`            | Thread principal            | `RwLock::write()`                     |

### Points d'attention pour le portage

1. **`[MaxMetricsHistory]IterationMetric`** : En Go, le tableau est initialisé à zéro. En Rust, utiliser `[IterationMetric::default(); MAX_METRICS_HISTORY]` ce qui requiert `#[derive(Default, Clone, Copy)]` sur `IterationMetric`.
2. **`sync.RWMutex`** : Mapper vers `std::sync::RwLock`. Toutefois, puisque `record_iteration()` et `should_adjust()` sont appelés depuis un seul goroutine/thread, le mutex n'est nécessaire que pour les lectures cross-thread (`get_thresholds`, `get_stats`). On peut envisager un pattern `AtomicI32` pour les seuils courants afin d'éviter le `RwLock` sur le chemin critique.
3. **`time.Time`** → `std::time::Instant` (pour `last_adjustment`).

---

## T5.2 — Mécanisme d'Hystérésis

### Objectif

Implémenter le mécanisme anti-oscillation qui empêche le `DynamicThresholdManager` de basculer continuellement entre modes FFT/parallèle.

### Constantes Go d'origine

```go
const (
    FFTSpeedupThreshold      = 1.2   // ratio minimal pour switcher vers FFT
    ParallelSpeedupThreshold = 1.1   // ratio minimal pour activer le parallélisme
    HysteresisMargin         = 0.15  // marge de 15% avant tout ajustement
)
```

### Spécification Rust

```rust
/// Ratio minimal de speedup pour basculer vers FFT.
/// Si avg_non_fft_time / avg_fft_time > 1.2, FFT est bénéfique.
const FFT_SPEEDUP_THRESHOLD: f64 = 1.2;

/// Ratio minimal de speedup pour activer le parallélisme.
/// Si avg_seq_time / avg_par_time > 1.1, le parallélisme est bénéfique.
const PARALLEL_SPEEDUP_THRESHOLD: f64 = 1.1;

/// Marge d'hystérésis : le changement relatif doit dépasser 15%
/// avant d'appliquer un ajustement. Prévient les oscillations.
const HYSTERESIS_MARGIN: f64 = 0.15;
```

### Algorithme de vérification de changement significatif

```rust
fn significant_change(old_val: i32, new_val: i32) -> bool {
    if old_val == 0 {
        return new_val != 0;
    }
    let change = ((new_val - old_val) as f64 / old_val as f64).abs();
    change > HYSTERESIS_MARGIN
}
```

### Diagramme d'analyse de stabilité

```
       ┌──────────────────────────────────────────────────────────┐
       │           Boucle de doubling (chaque itération)          │
       └─────────────────────────┬────────────────────────────────┘
                                 │
                    record_iteration(bit_len, duration, used_fft, used_par)
                                 │
                                 ▼
                  iteration_count % adjustment_interval == 0 ?
                       │                        │
                      Non                      Oui
                       │                        │
                   (continuer)                  ▼
                                  metrics_count >= MIN_METRICS (3) ?
                                       │               │
                                      Non             Oui
                                       │               │
                                   (continuer)         ▼
                                            ┌────────────────────┐
                                            │ analyze_fft()      │
                                            │ analyze_parallel() │
                                            └────────┬───────────┘
                                                     │
                                                     ▼
                                       significant_change() pour chaque ?
                                       ┌──────────┐  ┌──────────┐
                                       │ FFT: Δ%  │  │ Par: Δ%  │
                                       │ > 15% ?  │  │ > 15% ?  │
                                       └────┬─────┘  └────┬─────┘
                                            │              │
                                      ┌─────┴──────┐ ┌────┴──────┐
                                      │Oui: ajuster│ │Oui: ajust.│
                                      │Non: garder │ │Non: garder│
                                      └────────────┘ └───────────┘
```

### Bornes de sécurité

| Seuil         | Borne inférieure            | Borne supérieure          |
| ------------- | ---------------------------- | -------------------------- |
| FFT           | 100 000 bits (plancher fixe) | `original_fft × 2`      |
| Parallélisme | 1 024 bits (plancher fixe)   | `original_parallel × 4` |

### Taux d'ajustement

| Direction                        | FFT                         | Parallélisme               |
| -------------------------------- | --------------------------- | --------------------------- |
| Baisser (mode bénéfique)       | `current × 9/10` (-10%)  | `current × 8/10` (-20%)  |
| Augmenter (mode non bénéfique) | `current × 11/10` (+10%) | `current × 12/10` (+20%) |

Le parallélisme a des taux d'ajustement plus agressifs car le surcoût de goroutines/threads est plus sensible aux variations de charge.

---

## T5.3 — Collecte de métriques par itération

### Structure IterationMetric

```go
// Go (internal/fibonacci/threshold_types.go)
type IterationMetric struct {
    BitLen       int
    Duration     time.Duration
    UsedFFT      bool
    UsedParallel bool
}
```

### Spécification Rust

```rust
/// Enregistrement de timing pour une seule itération de doubling.
#[derive(Debug, Clone, Copy, Default)]
pub struct IterationMetric {
    /// Longueur en bits de F(k) à cette itération.
    pub bit_len: usize,
    /// Durée de cette itération.
    pub duration: Duration,
    /// La multiplication FFT a-t-elle été utilisée ?
    pub used_fft: bool,
    /// Le parallélisme multi-thread a-t-il été utilisé ?
    pub used_parallel: bool,
}
```

### Protocole de timing

L'enregistrement se fait à chaque pas de la boucle Fast Doubling, immédiatement après l'exécution du `DoublingStepExecutor` :

```
Pour chaque bit k de n (MSB → LSB) :
    t_start = Instant::now()
    executor.execute_step(a, b, bit_k)      // multiplication + opérations
    duration = t_start.elapsed()
    bit_len = a.bit_len()                   // taille courante du résultat
    used_fft = bit_len >= fft_threshold     // le seuil courant, pas l'original
    used_parallel = bit_len >= par_threshold
    manager.record_iteration(bit_len, duration, used_fft, used_parallel)
```

### Catégorisation FFT vs Non-FFT

Les métriques sont catégorisées **a posteriori** lors de l'analyse, pas lors de la collecte :

```rust
fn get_categorized_metrics(&self) -> (Vec<IterationMetric>, Vec<IterationMetric>) {
    let active = self.get_active_metrics();
    let (fft, non_fft): (Vec<_>, Vec<_>) = active
        .into_iter()
        .partition(|m| m.used_fft);
    (fft, non_fft)
}
```

### ThresholdStats — Structure de reporting

```rust
/// Statistiques courantes du DynamicThresholdManager.
pub struct ThresholdStats {
    pub current_fft: i32,
    pub current_parallel: i32,
    pub original_fft: i32,
    pub original_parallel: i32,
    pub metrics_collected: usize,
    pub iterations_processed: usize,
}
```

### DynamicThresholdConfig — Configuration

```rust
/// Configuration pour l'ajustement dynamique des seuils.
pub struct DynamicThresholdConfig {
    pub initial_fft_threshold: i32,
    pub initial_parallel_threshold: i32,
    pub adjustment_interval: usize,
    pub enabled: bool,
}
```

---

## T5.4 — Algorithme d'ajustement des seuils

### Pseudocode détaillé

```
FONCTION should_adjust() → (new_fft, new_parallel, adjusted)
│
├─ SI iteration_count % adjustment_interval ≠ 0
│     RETOURNER (current_fft, current_parallel, false)
│
├─ SI metrics_count < MIN_METRICS_FOR_ADJUSTMENT (3)
│     RETOURNER (current_fft, current_parallel, false)
│
├─ new_fft ← analyze_fft_threshold()
├─ new_parallel ← analyze_parallel_threshold()
│
├─ fft_changed ← significant_change(current_fft, new_fft)
├─ parallel_changed ← significant_change(current_parallel, new_parallel)
│
├─ SI fft_changed OU parallel_changed
│     SI fft_changed : current_fft ← new_fft
│     SI parallel_changed : current_parallel ← new_parallel
│     last_adjustment ← Instant::now()
│     RETOURNER (current_fft, current_parallel, true)
│
└─ RETOURNER (current_fft, current_parallel, false)

FONCTION analyze_fft_threshold() → i32
│
├─ metrics ← get_active_metrics()
├─ (fft_metrics, non_fft_metrics) ← partition par used_fft
│
├─ SI l'une des listes est vide
│     RETOURNER current_fft_threshold
│
├─ avg_fft_time_per_bit ← Σ(duration) / Σ(bit_len) pour fft_metrics
├─ avg_non_fft_time_per_bit ← Σ(duration) / Σ(bit_len) pour non_fft_metrics
│
├─ ratio ← avg_non_fft / avg_fft
│
├─ SI ratio > 1.2 (FFT_SPEEDUP_THRESHOLD)
│     // FFT est plus rapide → baisser le seuil de 10%
│     new ← current_fft × 9 / 10
│     RETOURNER max(new, 100_000)     // plancher de sécurité
│
├─ SI ratio < 1/1.2 ≈ 0.833
│     // FFT est plus lent → augmenter le seuil de 10%
│     new ← current_fft × 11 / 10
│     RETOURNER min(new, original_fft × 2)  // plafond de sécurité
│
└─ RETOURNER current_fft_threshold  // pas de changement

FONCTION analyze_parallel_threshold() → i32
│
├─ (même structure que analyze_fft avec les différences suivantes)
│
├─ ratio > 1.1 (PARALLEL_SPEEDUP_THRESHOLD)
│     new ← current_parallel × 8 / 10  // baisse de 20%
│     RETOURNER max(new, 1_024)
│
├─ ratio < 1/1.1 ≈ 0.909
│     new ← current_parallel × 12 / 10  // hausse de 20%
│     RETOURNER min(new, original_parallel × 4)
│
└─ RETOURNER current_parallel_threshold
```

### Fonction avg_time_per_bit

```rust
fn avg_time_per_bit(metrics: &[IterationMetric]) -> f64 {
    if metrics.is_empty() {
        return 0.0;
    }
    let total_ns: u128 = metrics.iter().map(|m| m.duration.as_nanos()).sum();
    let total_bits: u128 = metrics.iter().map(|m| m.bit_len as u128).sum();
    if total_bits == 0 {
        return 0.0;
    }
    total_ns as f64 / total_bits as f64
}
```

### Schéma de convergence

```
Itération    1   5   10   15   20   25   30   35   40
             │   │    │    │    │    │    │    │    │
FFT seuil   500K │   450K │   405K │   365K │   365K (stabilisé)
             │   │    │    │    │    │    │    │    │
Par seuil   4096 │  3277  │  2621  │  2097  │  2097 (stabilisé)
             │   │    │    │    │    │    │    │    │
             ▼   ▼    ▼    ▼    ▼    ▼    ▼    ▼    ▼
         Collect. Ajust. Ajust. Ajust. Stable (hystérésis ≤ 15%)
```

---

## T5.5 — Format du profil de calibration

### Schéma JSON complet

```json
{
  "$schema": "https://json-schema.org/draft/2020-12/schema",
  "title": "CalibrationProfile",
  "description": "Profil de calibration FibCalc pour persistance des seuils optimaux",
  "type": "object",
  "required": [
    "cpu_model", "num_cpu", "target_arch", "target_os",
    "rust_version", "word_size", "profile_version",
    "optimal_parallel_threshold", "optimal_fft_threshold",
    "optimal_strassen_threshold", "calibrated_at", "calibration_n"
  ],
  "properties": {
    "cpu_model": {
      "type": "string",
      "description": "Identifiant CPU : '{arch}-{num_cpu}-cores'",
      "examples": ["x86_64-12-cores", "aarch64-8-cores"]
    },
    "num_cpu": {
      "type": "integer",
      "minimum": 1,
      "description": "Nombre de CPUs logiques"
    },
    "target_arch": {
      "type": "string",
      "enum": ["x86_64", "x86", "aarch64", "arm", "riscv64gc"],
      "description": "Architecture cible (remplace GOARCH)"
    },
    "target_os": {
      "type": "string",
      "enum": ["linux", "windows", "macos"],
      "description": "Système d'exploitation cible (remplace GOOS)"
    },
    "rust_version": {
      "type": "string",
      "description": "Version du compilateur Rust",
      "examples": ["1.82.0"]
    },
    "word_size": {
      "type": "integer",
      "enum": [32, 64],
      "description": "Taille du mot machine en bits"
    },
    "optimal_parallel_threshold": {
      "type": "integer",
      "minimum": 0,
      "description": "Seuil optimal de parallélisme en bits"
    },
    "optimal_fft_threshold": {
      "type": "integer",
      "minimum": 0,
      "description": "Seuil optimal FFT en bits"
    },
    "optimal_strassen_threshold": {
      "type": "integer",
      "minimum": 0,
      "description": "Seuil optimal Strassen en bits"
    },
    "calibrated_at": {
      "type": "string",
      "format": "date-time",
      "description": "Horodatage ISO 8601 de la calibration"
    },
    "calibration_n": {
      "type": "integer",
      "minimum": 1,
      "description": "Index de Fibonacci utilisé pour la calibration"
    },
    "calibration_time": {
      "type": "string",
      "description": "Durée totale de la calibration (formatée)",
      "examples": ["45.2s"]
    },
    "profile_version": {
      "type": "integer",
      "const": 2,
      "description": "Version du format de profil"
    }
  }
}
```

### Structure Rust avec serde

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

const CURRENT_PROFILE_VERSION: u32 = 2;
const DEFAULT_PROFILE_FILENAME: &str = ".fibcalc_calibration.json";

#[derive(Debug, Serialize, Deserialize)]
pub struct CalibrationProfile {
    // Identification matérielle
    pub cpu_model: String,
    pub num_cpu: usize,
    pub target_arch: String,   // std::env::consts::ARCH
    pub target_os: String,     // std::env::consts::OS
    pub rust_version: String,
    pub word_size: u32,        // std::mem::size_of::<usize>() * 8

    // Seuils calibrés
    pub optimal_parallel_threshold: i32,
    pub optimal_fft_threshold: i32,
    pub optimal_strassen_threshold: i32,

    // Métadonnées
    pub calibrated_at: DateTime<Utc>,
    pub calibration_n: u64,
    #[serde(default)]
    pub calibration_time: String,

    // Version pour compatibilité ascendante
    pub profile_version: u32,
}
```

### Règles de validation (`is_valid`)

| Champ               | Condition de validité             | Mapping Go → Rust                                       |
| ------------------- | ---------------------------------- | -------------------------------------------------------- |
| `profile_version` | `== CURRENT_PROFILE_VERSION (2)` | Identique                                                |
| `num_cpu`         | `== num_cpus::get()`             | `runtime.NumCPU()` → `num_cpus::get()`              |
| `target_arch`     | `== std::env::consts::ARCH`      | `runtime.GOARCH` → `ARCH`                           |
| `word_size`       | `== size_of::<usize>() * 8`      | `32 << (^uint(0) >> 63)` → `size_of::<usize>() * 8` |

```rust
impl CalibrationProfile {
    pub fn is_valid(&self) -> bool {
        self.profile_version == CURRENT_PROFILE_VERSION
            && self.num_cpu == num_cpus::get()
            && self.target_arch == std::env::consts::ARCH
            && self.word_size == (std::mem::size_of::<usize>() as u32 * 8)
    }

    pub fn is_stale(&self, max_age: Duration) -> bool {
        Utc::now().signed_duration_since(self.calibrated_at)
            > chrono::Duration::from_std(max_age).unwrap_or(chrono::Duration::max_value())
    }
}
```

### Persistance

| Opération         | Go                                   | Rust                                                                       |
| ------------------ | ------------------------------------ | -------------------------------------------------------------------------- |
| Sérialisation     | `json.MarshalIndent`               | `serde_json::to_string_pretty`                                           |
| Écriture          | `os.WriteFile(path, data, 0600)`   | `std::fs::write` + permissions via `std::os::unix::fs::PermissionsExt` |
| Lecture            | `os.ReadFile` + `json.Unmarshal` | `std::fs::read_to_string` + `serde_json::from_str`                     |
| Chemin par défaut | `~/.fibcalc_calibration.json`      | `dirs::home_dir()` + `DEFAULT_PROFILE_FILENAME`                        |

---

## T5.6 — Modes de calibration (full, auto, cached) — Chaîne de fallback à 3 niveaux

### Diagramme de flux de la chaîne de fallback

```
                          ┌──────────────────────────────┐
                          │  AutoCalibrateWithProfile()  │
                          │  Point d'entrée principal    │
                          └──────────────┬───────────────┘
                                         │
                    ┌────────────────────┐│┌────────────────────┐
                    │ Vérifier que le    │││                    │
                    │ calculateur "fast" ││↓                    │
                    │ est disponible     ││                     │
                    └────────────────────┘│                     │
                                         ▼
                          ┌──────────────────────────────┐
                    ┌─────│  TIER 1 : Profil caché       │
                    │     │  LoadOrCreateProfile()        │
                    │     │  profile.is_valid() ?         │
                    │     └──────────────┬───────────────┘
                    │                    │
                   Oui                  Non
                    │                    │
                    ▼                    ▼
         ┌──────────────────┐  ┌──────────────────────────────┐
         │ Appliquer les    │  │  TIER 2 : Micro-benchmarks   │
         │ seuils du profil │  │  QuickCalibrate()             │
         │ RETOURNER (ok)   │  │  ~100ms, confidence ≥ 0.5 ?  │
         └──────────────────┘  └──────────────┬───────────────┘
                                              │
                                     ┌────────┴────────┐
                                    Oui               Non
                                     │                 │
                                     ▼                 ▼
                          ┌─────────────────┐  ┌──────────────────────────────┐
                          │ Appliquer les   │  │  TIER 3 : Calibration full   │
                          │ seuils estimés  │  │  calibrationRunner           │
                          │ Sauver profil   │  │  findBestParallelThreshold() │
                          │ RETOURNER (ok)  │  │  findBestFFTThreshold()      │
                          └─────────────────┘  │  findBestStrassenThreshold() │
                                               └──────────────┬───────────────┘
                                                              │
                                                   ┌──────────┴─────────┐
                                                  Oui                  Non
                                                   │                    │
                                                   ▼                    ▼
                                        ┌─────────────────┐  ┌─────────────────┐
                                        │ Appliquer les   │  │ RETOURNER les   │
                                        │ seuils mesurés  │  │ valeurs par     │
                                        │ Sauver profil   │  │ défaut (échec)  │
                                        │ RETOURNER (ok)  │  └─────────────────┘
                                        └─────────────────┘
```

### Correspondance Go → Rust pour chaque mode

| Mode   | Point d'entrée Go             | Point d'entrée Rust                 |
| ------ | ------------------------------ | ------------------------------------ |
| Full   | `RunCalibration()`           | `pub fn run_full_calibration()`    |
| Auto   | `AutoCalibrateWithProfile()` | `pub fn auto_calibrate()`          |
| Cached | `LoadCachedCalibration()`    | `pub fn load_cached_calibration()` |

### Mode Full (`--calibrate`)

```rust
pub fn run_full_calibration(
    ctx: &Context,       // CancellationToken de tokio ou un Arc<AtomicBool>
    writer: &mut dyn Write,
    calculator: &dyn Calculator,
) -> ExitCode {
    let thresholds = generate_parallel_thresholds();
    let mut results: Vec<CalibrationResult> = Vec::with_capacity(thresholds.len());
    let mut best_duration = Duration::MAX;
    let mut best_threshold = 0i32;

    for &threshold in &thresholds {
        if ctx.is_cancelled() { return ExitCode::Canceled; }

        let start = Instant::now();
        let result = calculator.calculate(ctx, CALIBRATION_N,
            Options { parallel_threshold: threshold, ..Default::default() });
        let duration = start.elapsed();

        match result {
            Ok(_) => {
                results.push(CalibrationResult { threshold, duration, err: None });
                if duration < best_duration {
                    best_duration = duration;
                    best_threshold = threshold;
                }
            }
            Err(e) => results.push(CalibrationResult { threshold, duration, err: Some(e) }),
        }
    }

    // Sauver le profil
    let mut profile = CalibrationProfile::new();
    profile.optimal_parallel_threshold = best_threshold;
    profile.optimal_fft_threshold = estimate_optimal_fft_threshold();
    profile.optimal_strassen_threshold = estimate_optimal_strassen_threshold();
    profile.save(None).ok();

    ExitCode::Success
}
```

### Mode Auto — Latence par tier

| Tier             | Latence typique | Condition de succès           |
| ---------------- | --------------- | ------------------------------ |
| 1 — Cache       | < 1 ms          | `profile.is_valid() == true` |
| 2 — Micro-bench | ~100–150 ms    | `results.confidence >= 0.5`  |
| 3 — Full runner | 2–30 s         | Au moins un trial réussi      |

---

## T5.7 — Estimation adaptative des seuils

### Formules heuristiques (sans benchmark)

#### f(cores) — Seuil parallèle

```rust
pub fn estimate_optimal_parallel_threshold() -> i32 {
    let num_cpu = num_cpus::get();
    match num_cpu {
        1      => 0,      // Pas de parallélisme
        2      => 8192,   // Surcoût goroutine élevé
        3..=4  => 4096,   // Défaut
        5..=8  => 2048,   // Plus de cœurs → seuil plus bas
        9..=16 => 1024,   // Beaucoup de cœurs
        _      => 512,    // 17+ cœurs → parallélisme agressif
    }
}
```

**Raisonnement** : Plus le nombre de cœurs est élevé, plus le surcoût relatif de la synchronisation est faible par rapport au gain de parallélisme. La relation est approximativement :

```
threshold ≈ 8192 / log₂(num_cpu)
```

#### f(arch) — Seuil FFT

```rust
pub fn estimate_optimal_fft_threshold() -> i32 {
    let word_size = std::mem::size_of::<usize>() * 8;
    if word_size == 64 {
        500_000  // 500K bits — optimal pour CPUs modernes avec grands caches L3
    } else {
        250_000  // 250K bits — taille de mot plus petite
    }
}
```

**Raisonnement** : Sur architectures 64 bits, les multiplications math/big utilisent des mots de 64 bits, ce qui double la quantité de données traitées par opération CPU par rapport à 32 bits. Le point de croisement FFT se déplace donc vers des tailles plus grandes.

#### f(CPU) — Seuil Strassen

```rust
pub fn estimate_optimal_strassen_threshold() -> i32 {
    let num_cpu = num_cpus::get();
    if num_cpu >= 4 {
        256   // Avec parallélisme, seuil plus bas
    } else {
        3072  // Défaut depuis constants.go
    }
}
```

**Raisonnement** : L'algorithme de Strassen réduit les multiplications de 8 à 7 au prix d'additions supplémentaires. Avec plusieurs cœurs, les additions supplémentaires peuvent être parallélisées, rendant Strassen bénéfique à des tailles plus petites.

### Génération de candidats de calibration

| Fonction                                 | Nombre de cœurs | Candidats générés                                    |
| ---------------------------------------- | ---------------- | ------------------------------------------------------- |
| `generate_parallel_thresholds()`       | 1                | `[0]`                                                 |
|                                          | 2–4             | `[0, 512, 1024, 2048, 4096]`                          |
|                                          | 5–8             | `[0, 256, 512, 1024, 2048, 4096, 8192]`               |
|                                          | 9–16            | `[0, 256, 512, 1024, 2048, 4096, 8192, 16384]`        |
|                                          | 17+              | `[0, 256, 512, 1024, 2048, 4096, 8192, 16384, 32768]` |
| `generate_quick_fft_thresholds()`      | Tous             | `[0, 750_000, 1_000_000, 1_500_000]`                  |
| `generate_quick_strassen_thresholds()` | Tous             | `[192, 256, 384, 512]`                                |

---

## T5.8 — Stratégie de micro-benchmarks

### Configuration du moteur

| Constante                        | Valeur | Description                         |
| -------------------------------- | ------ | ----------------------------------- |
| `MICRO_BENCH_ITERATIONS`       | 3      | Itérations par test pour moyennage |
| `MICRO_BENCH_TIMEOUT`          | 150 ms | Timeout total de la suite           |
| `MICRO_BENCH_PER_TEST_TIMEOUT` | 30 ms  | Timeout par test individuel         |

### Tailles de test

```rust
const MICRO_BENCH_TEST_SIZES: &[usize] = &[
    500,    // ~32K bits — territoire math/big standard
    2_000,  // ~128K bits — zone seuil parallèle
    8_000,  // ~512K bits — zone seuil FFT
    16_000, // ~1M bits — territoire FFT
];
```

### Matrice de test

Pour chaque taille, 4 configurations sont testées :

| Config | Méthode de multiplication       | Parallélisme |
| ------ | -------------------------------- | ------------- |
| 1      | `num::BigUint::mul` (standard) | Séquentiel   |
| 2      | `num::BigUint::mul` (standard) | Parallèle    |
| 3      | FFT multiplication               | Séquentiel   |
| 4      | FFT multiplication               | Parallèle    |

Total : 4 tailles × 4 configs = **16 tests** exécutés en parallèle avec sémaphore `= num_cpus::get()`.

### Algorithme de sélection de seuil

```
FONCTION find_fft_crossover(results_by_size) → i32
│
├─ crossover_size ← 0
├─ POUR CHAQUE (size, results) DANS results_by_size :
│     avg_std ← moyenne(duration) pour résultats NON-FFT
│     avg_fft ← moyenne(duration) pour résultats FFT
│     SI avg_fft < avg_std :
│         bit_size ← size × 64  // mots → bits (64 bits)
│         SI crossover_size == 0 OU bit_size < crossover_size :
│             crossover_size ← bit_size
│
├─ SI crossover_size == 0 :
│     RETOURNER 1_000_000  // défaut conservateur
│
└─ RETOURNER crossover_size × 9 / 10  // marge de 10%

FONCTION find_parallel_crossover(results_by_size) → i32
│
├─ SI num_cpus == 1 : RETOURNER 0
├─ crossover_size ← 0
├─ POUR CHAQUE (size, results) DANS results_by_size :
│     (filtrer NON-FFT seulement)
│     avg_seq ← moyenne(duration) pour résultats séquentiels
│     avg_par ← moyenne(duration) pour résultats parallèles
│     SI avg_par < avg_seq × 9/10 :  // gain ≥ 10%
│         bit_size ← size × 64
│         SI crossover_size == 0 OU bit_size < crossover_size :
│             crossover_size ← bit_size
│
├─ SI crossover_size == 0 :
│     RETOURNER 4096  // défaut
│
└─ RETOURNER crossover_size
```

### Score de confiance

```
confidence = 0.5  (base — les défauts conservateurs sont raisonnables)
SI crossover FFT trouvé    : confidence += 0.2
SI crossover parallèle trouvé : confidence += 0.2
confidence = min(confidence, 1.0)
```

Le seuil d'acceptation pour l'auto-calibration est `confidence >= 0.5`.

### Génération de nombres de test

```rust
fn generate_test_number(words: usize) -> BigUint {
    let limbs: Vec<u64> = (0..words)
        .map(|i| 0xAAAAAAAAAAAAAAAAu64 ^ (i as u64 * 0x1234567))
        .collect();
    BigUint::new(limbs.iter().map(|&w| w as u32).collect()) // ou via from_bytes_le
}
```

Le pattern `0xAAAA...^ i*0x1234567` est déterministe (reproductible) et exerce tous les bits sans être uniforme.

---

# Phase 6 — Spécification Détaillée du TUI

> Portage de `internal/tui/*` (Bubble Tea / Elm architecture) vers Rust avec `ratatui` + `crossterm`.

---

## T6.1 — Mapping de l'architecture Elm (Bubble Tea → ratatui)

### Vue d'ensemble du mapping

| Concept Bubble Tea (Go)               | Équivalent ratatui (Rust)                                    |
| ------------------------------------- | ------------------------------------------------------------- |
| `tea.Model` (interface)             | Struct `App` implémentant une boucle événementielle      |
| `Model.Init() → tea.Cmd`           | `App::new()` + spawn de tâches tokio initiales             |
| `Model.Update(msg) → (Model, Cmd)` | `App::handle_event(event) → Action`                        |
| `Model.View() → string`            | `App::draw(frame: &mut Frame)`                              |
| `tea.Msg` (interface vide)          | `enum AppMessage { ... }`                                   |
| `tea.Cmd` (fonction)                | `tokio::spawn` ou `mpsc::channel`                         |
| `tea.Program`                       | Boucle `loop { terminal.draw(); handle_events(); }`         |
| `tea.WithAltScreen()`               | `crossterm::terminal::enable_raw_mode()` + alternate screen |

### Structure du Model racine

Le `Model` Go contient **~20 champs** répartis en sous-modèles et état global :

```rust
pub struct App {
    // Sous-modèles (panneaux)
    header: HeaderModel,
    logs: LogsModel,
    metrics: MetricsModel,
    chart: ChartModel,
    footer: FooterModel,

    // Bindings clavier
    keymap: KeyMap,

    // Contexte d'exécution
    cancel_token: CancellationToken,    // remplace ctx + cancel
    parent_cancel: CancellationToken,   // remplace parentCtx
    config: AppConfig,
    calculators: Vec<Box<dyn Calculator>>,
    generation: u64,                    // compteur monotone anti-staleness

    // Canal de messages depuis les goroutines de calcul
    message_rx: mpsc::UnboundedReceiver<AppMessage>,
    message_tx: mpsc::UnboundedSender<AppMessage>,

    // Dimensions du terminal
    width: u16,
    height: u16,

    // État global
    paused: bool,
    done: bool,
    exit_code: i32,
}
```

### Cycle Init / Update / View

```
┌──────────────────────────────────────────────────────────────────┐
│                    Boucle principale ratatui                     │
│                                                                  │
│  loop {                                                          │
│      terminal.draw(|frame| app.draw(frame));    // VIEW          │
│                                                                  │
│      // Collecter événements (non-bloquant, timeout 16ms)        │
│      if crossterm::event::poll(Duration::from_millis(16))? {     │
│          let event = crossterm::event::read()?;                  │
│          app.handle_event(event);               // UPDATE (keys) │
│      }                                                           │
│                                                                  │
│      // Drainer les messages du canal                             │
│      while let Ok(msg) = app.message_rx.try_recv() {            │
│          app.handle_message(msg);               // UPDATE (msgs) │
│      }                                                           │
│                                                                  │
│      if app.should_quit() { break; }                             │
│  }                                                               │
└──────────────────────────────────────────────────────────────────┘
```

### Différences architecturales clés

| Aspect                             | Bubble Tea (Go)                                  | ratatui (Rust)                                 |
| ---------------------------------- | ------------------------------------------------ | ---------------------------------------------- |
| **Immutabilité du modèle** | Copie du Model à chaque Update                  | Mutation in-place (`&mut self`)              |
| **Dispatching de messages**  | `tea.Program.Send()` (thread-safe)             | `mpsc::UnboundedSender::send()`              |
| **Rendu**                    | `View() → String` (le framework fait le diff) | `draw(Frame)` → rendu direct dans le buffer |
| **Commandes asynchrones**    | `tea.Cmd` (thunk retournant un Msg)            | `tokio::spawn` + envoi via channel           |
| **Écran alternatif**        | `tea.WithAltScreen()`                          | `crossterm::terminal::EnterAlternateScreen`  |

---

## T6.2 — Filtrage de messages par génération

### Problème

Quand l'utilisateur appuie sur `r` (Reset), le calcul en cours est annulé et un nouveau démarre. Les goroutines/tâches du calcul précédent peuvent encore envoyer des messages (`CalculationCompleteMsg`, `ProgressMsg`, etc.) qui sont désormais obsolètes.

### Solution : compteur de génération

```rust
// À chaque Reset :
self.generation += 1;
self.cancel_token.cancel();
self.cancel_token = CancellationToken::new();

// Les messages portent la génération :
pub enum AppMessage {
    CalculationComplete { exit_code: i32, generation: u64 },
    ContextCancelled { err: String, generation: u64 },
    Progress { /* ... */ },  // Pas de génération (flux continu, filtré par canal)
    // ...
}
```

### Filtrage dans le handler

```rust
fn handle_message(&mut self, msg: AppMessage) {
    match msg {
        AppMessage::CalculationComplete { exit_code, generation } => {
            if generation != self.generation {
                return;  // message obsolète, ignorer
            }
            self.done = true;
            self.exit_code = exit_code;
            self.header.set_done();
            self.chart.set_done(self.header.elapsed());
            self.footer.set_done(true);
        }
        AppMessage::ContextCancelled { generation, .. } => {
            if generation != self.generation {
                return;  // message obsolète
            }
            self.done = true;
            // Déclencher la sortie
        }
        // Les messages de progrès sont filtrés par le fait que l'ancien canal
        // est droppé lors du Reset (le receiver est recréé).
        _ => { /* ... */ }
    }
}
```

### Diagramme de séquence du Reset

```
Utilisateur    Model              Calcul Gen=0       Calcul Gen=1
    │              │                    │                   │
    │──(r)────────▶│                    │                   │
    │              │──cancel()─────────▶│                   │
    │              │  generation = 1    │                   │
    │              │  nouveau canal     │                   │
    │              │──spawn()───────────┼──────────────────▶│
    │              │                    │                   │
    │              │◀──Complete(gen=0)──│ ← IGNORÉ          │
    │              │                    │ (generation != 1) │
    │              │                    │                   │
    │              │◀──────────────────────Progress(gen=1)──│
    │              │                    │  ← ACCEPTÉ        │
    │              │◀──────────────────────Complete(gen=1)──│
    │              │                    │  ← ACCEPTÉ        │
```

---

## T6.3 — Pattern programRef pour accès au tea.Program

### Problème Go

Bubble Tea copie le `Model` à chaque `Update()`. Les goroutines ne peuvent pas tenir une référence stable au modèle pour envoyer des messages.

### Solution Go : programRef

```go
type programRef struct {
    program *tea.Program
}

func (r *programRef) Send(msg tea.Msg) {
    if r.program != nil {
        r.program.Send(msg)
    }
}
```

Le `programRef` est alloué sur le heap (`&programRef{}`), et le `Model` stocke un `*programRef`. La référence survit aux copies du Model.

### Équivalent Rust : canal mpsc

En Rust avec ratatui, le modèle n'est **pas** copié (mutation in-place). Le problème se transforme : les tâches asynchrones (calcul, sampling métriques) doivent pouvoir envoyer des messages au thread principal.

```rust
// Création dans App::new()
let (message_tx, message_rx) = mpsc::unbounded_channel::<AppMessage>();

// Injection dans les tâches spawned
let tx = self.message_tx.clone();
tokio::spawn(async move {
    // ... calcul ...
    tx.send(AppMessage::CalculationComplete {
        exit_code: 0,
        generation: gen,
    }).ok();
});
```

### Comparaison des patterns

| Aspect        | Go (programRef)                        | Rust (mpsc channel)                                  |
| ------------- | -------------------------------------- | ---------------------------------------------------- |
| Thread-safety | `tea.Program.Send()` est thread-safe | `mpsc::UnboundedSender` est `Send + Clone`       |
| Lifetime      | Heap-allocated, survit aux copies      | `Clone` du sender, survit aux moves                |
| Nil check     | `if r.program != nil`                | Le canal retourne `Err` si le receiver est droppé |
| Multi-sender  | Un seul `programRef` partagé        | Chaque tâche clone le `Sender`                    |

---

## T6.4 — Catalogue des types de messages

### Enum AppMessage complet

```rust
use std::time::{Duration, Instant};

pub enum AppMessage {
    /// Mise à jour de progression d'un calculateur.
    Progress(ProgressMsg),

    /// Le canal de progression a été fermé.
    ProgressDone,

    /// Résultats de la comparaison multi-algorithmes.
    ComparisonResults(ComparisonResultsMsg),

    /// Résultat final du calcul.
    FinalResult(FinalResultMsg),

    /// Erreur de calcul.
    Error(ErrorMsg),

    /// Tick périodique (500ms) pour l'échantillonnage métriques.
    Tick(Instant),

    /// Statistiques mémoire runtime.
    MemStats(MemStatsMsg),

    /// Statistiques système (CPU%, MEM%).
    SysStats(SysStatsMsg),

    /// Calcul terminé.
    CalculationComplete(CalculationCompleteMsg),

    /// Contexte annulé (timeout ou Ctrl+C).
    ContextCancelled(ContextCancelledMsg),

    /// Indicateurs de performance post-calcul.
    Indicators(IndicatorsMsg),
}
```

### Détail de chaque type de message

```rust
/// Correspondance : tui/messages.go:ProgressMsg
pub struct ProgressMsg {
    pub calculator_index: usize,
    pub value: f64,              // 0.0..1.0
    pub average_progress: f64,   // progression moyenne multi-algo
    pub eta: Duration,
}

/// Correspondance : tui/messages.go:ComparisonResultsMsg
pub struct ComparisonResultsMsg {
    pub results: Vec<CalculationResult>,
}

/// Correspondance : tui/messages.go:FinalResultMsg
pub struct FinalResultMsg {
    pub result: CalculationResult,
    pub n: u64,
    pub verbose: bool,
    pub details: bool,
    pub show_value: bool,
}

/// Correspondance : tui/messages.go:ErrorMsg
pub struct ErrorMsg {
    pub err: String,
    pub duration: Duration,
}

/// Correspondance : tui/messages.go:MemStatsMsg
pub struct MemStatsMsg {
    pub alloc: u64,           // RSS courant en octets
    pub heap_sys: u64,        // Mémoire système allouée au heap
    pub num_gc: u32,          // Nombre de cycles GC (N/A en Rust, garder pour compat)
    pub pause_total_ns: u64,  // Pause GC totale (N/A en Rust)
    pub num_threads: usize,   // Remplace NumGoroutine
}

/// Correspondance : tui/messages.go:SysStatsMsg
pub struct SysStatsMsg {
    pub cpu_percent: f64,     // 0.0..100.0
    pub mem_percent: f64,     // 0.0..100.0
}

/// Correspondance : tui/messages.go:CalculationCompleteMsg
pub struct CalculationCompleteMsg {
    pub exit_code: i32,
    pub generation: u64,
}

/// Correspondance : tui/messages.go:ContextCancelledMsg
pub struct ContextCancelledMsg {
    pub err: String,
    pub generation: u64,
}

/// Correspondance : tui/messages.go:IndicatorsMsg
pub struct IndicatorsMsg {
    pub indicators: Indicators,
}
```

### Adaptations Rust

| Champ Go                     | Champ Rust              | Raison                                                      |
| ---------------------------- | ----------------------- | ----------------------------------------------------------- |
| `NumGC uint32`             | `num_gc: u32`         | Pas de GC en Rust, mais conservé pour l'affichage (sera 0) |
| `PauseTotalNs uint64`      | `pause_total_ns: u64` | Idem — sera toujours 0                                     |
| `NumGoroutine int`         | `num_threads: usize`  | Goroutines → threads Rayon/tokio                           |
| `time.Time` (dans TickMsg) | `Instant`             | Monotone, adapté aux mesures de durée                     |

---

## T6.5 — Algorithme de layout adaptatif 60/40

### Constantes de layout

```rust
const HEADER_HEIGHT: u16 = 1;
const FOOTER_HEIGHT: u16 = 1;
const MIN_BODY_HEIGHT: u16 = 4;
const METRICS_FIXED_H: u16 = 7;  // hauteur fixe du panneau métriques
```

### Algorithme de calcul des dimensions

```rust
fn layout_panels(&mut self) {
    let body_height = (self.height - HEADER_HEIGHT - FOOTER_HEIGHT)
        .max(MIN_BODY_HEIGHT);

    // Répartition horizontale : 60% logs, 40% colonne droite
    let logs_width = self.width * 60 / 100;
    let right_width = self.width - logs_width;

    // Répartition verticale colonne droite
    let metrics_h = METRICS_FIXED_H.min(body_height / 2);
    let chart_h = body_height - metrics_h;

    self.header.set_width(self.width);
    self.footer.set_width(self.width);
    self.logs.set_size(logs_width, body_height);
    self.metrics.set_size(right_width, metrics_h);
    self.chart.set_size(right_width, chart_h);
}
```

### Diagramme de layout

```
+━━━━━━━━━━━━━━━━━━━━━━━━━ width ━━━━━━━━━━━━━━━━━━━━━━━━━━━━+
│                    Header (1 ligne)                           │ ← HEADER_HEIGHT
+━━━━━━━━ 60% ━━━━━━━━━╋━━━━━━━━━━━ 40% ━━━━━━━━━━━━━━━━━━━━━+
│                       ┃   MetricsModel                       │
│                       ┃   (METRICS_FIXED_H = 7,              │
│    LogsModel          ┃    capé à body_height/2)             │
│    (body_height)      ┣━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━│
│                       ┃   ChartModel                         │
│                       ┃   (body_height - metrics_h)          │
│                       ┃                                       │
│                       ┃                                       │
+━━━━━━━━━━━━━━━━━━━━━━━┻━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━+
│                    Footer (1 ligne)                           │ ← FOOTER_HEIGHT
+━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━+
```

### Implémentation ratatui avec Layout

```rust
use ratatui::layout::{Constraint, Direction, Layout, Rect};

fn compute_layout(area: Rect) -> AppLayout {
    // Division verticale : header / body / footer
    let vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(HEADER_HEIGHT),
            Constraint::Min(MIN_BODY_HEIGHT),
            Constraint::Length(FOOTER_HEIGHT),
        ])
        .split(area);

    let header_area = vertical[0];
    let body_area = vertical[1];
    let footer_area = vertical[2];

    // Division horizontale du body : 60% / 40%
    let horizontal = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(60),
            Constraint::Percentage(40),
        ])
        .split(body_area);

    let logs_area = horizontal[0];
    let right_area = horizontal[1];

    // Division verticale colonne droite : metrics fixe / chart expansible
    let metrics_h = METRICS_FIXED_H.min(right_area.height / 2);
    let right_vertical = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(metrics_h),
            Constraint::Min(0),
        ])
        .split(right_area);

    AppLayout {
        header: header_area,
        logs: logs_area,
        metrics: right_vertical[0],
        chart: right_vertical[1],
        footer: footer_area,
    }
}
```

### Gestion du redimensionnement

```rust
// crossterm émet un Event::Resize lors du changement de taille du terminal
fn handle_event(&mut self, event: Event) {
    match event {
        Event::Resize(width, height) => {
            self.width = width;
            self.height = height;
            // Le layout est recalculé automatiquement au prochain draw()
        }
        Event::Key(key_event) => self.handle_key(key_event),
        _ => {}
    }
}
```

### Tailles minimales

| Panneau | Largeur min    | Hauteur min |
| ------- | -------------- | ----------- |
| Logs    | 20 chars       | 4 lignes    |
| Metrics | 15 chars       | 3 lignes    |
| Chart   | 15 chars       | 4 lignes    |
| Header  | Pleine largeur | 1 ligne     |
| Footer  | Pleine largeur | 1 ligne     |

---

## T6.6 — Ring buffer pour sparklines

### Structure Go d'origine

```go
// tui/sparkline.go
type RingBuffer struct {
    data  []float64
    head  int
    count int
}
```

### Spécification Rust

```rust
/// Buffer circulaire à capacité dynamique pour échantillons float64.
/// Utilisé pour les sparklines CPU et mémoire.
pub struct RingBuffer {
    data: Vec<f64>,
    head: usize,   // index du prochain slot d'écriture
    count: usize,  // nombre d'éléments valides (≤ capacity)
}

impl RingBuffer {
    pub fn new(capacity: usize) -> Self {
        let capacity = capacity.max(1);
        Self {
            data: vec![0.0; capacity],
            head: 0,
            count: 0,
        }
    }

    /// Ajoute un échantillon, écrasant le plus ancien si plein.
    pub fn push(&mut self, value: f64) {
        self.data[self.head] = value;
        self.head = (self.head + 1) % self.data.len();
        if self.count < self.data.len() {
            self.count += 1;
        }
    }

    /// Retourne le nombre d'échantillons valides.
    pub fn len(&self) -> usize { self.count }

    /// Retourne la capacité du buffer.
    pub fn capacity(&self) -> usize { self.data.len() }

    /// Retourne le dernier échantillon ajouté, ou 0.0 si vide.
    pub fn last(&self) -> f64 {
        if self.count == 0 {
            return 0.0;
        }
        let idx = if self.head == 0 { self.data.len() - 1 } else { self.head - 1 };
        self.data[idx]
    }

    /// Retourne les échantillons en ordre chronologique (plus ancien en premier).
    pub fn slice(&self) -> Vec<f64> {
        if self.count == 0 {
            return Vec::new();
        }
        let mut result = Vec::with_capacity(self.count);
        let start = if self.count < self.data.len() {
            0
        } else {
            self.head  // le plus ancien est juste après head quand plein
        };
        for i in 0..self.count {
            result.push(self.data[(start + i) % self.data.len()]);
        }
        result
    }

    /// Redimensionne le buffer, conservant les échantillons les plus récents.
    pub fn resize(&mut self, new_capacity: usize) {
        let new_capacity = new_capacity.max(1);
        if new_capacity == self.data.len() {
            return;
        }
        let old_data = self.slice();
        self.data = vec![0.0; new_capacity];
        self.head = 0;
        self.count = 0;
        let start = if old_data.len() > new_capacity {
            old_data.len() - new_capacity
        } else {
            0
        };
        for &v in &old_data[start..] {
            self.push(v);
        }
    }

    /// Réinitialise le buffer sans modifier la capacité.
    pub fn reset(&mut self) {
        self.head = 0;
        self.count = 0;
    }
}
```

### Diagramme du wrapping circulaire

```
Capacité = 5, après 7 insertions (valeurs 10, 20, 30, 40, 50, 60, 70)

Étape 1-5 (buffer non plein) :
  data: [10, 20, 30, 40, 50]   head=0  count=5
                                 ↑ (prochaine écriture)

Étape 6 (écrasement) :
  data: [60, 20, 30, 40, 50]   head=1  count=5
              ↑

Étape 7 :
  data: [60, 70, 30, 40, 50]   head=2  count=5
                  ↑

slice() → [30, 40, 50, 60, 70]  (chronologique)
           ↑oldest         ↑newest
```

### Rendu sparkline depuis le RingBuffer

```rust
const SPARKLINE_CHARS: [char; 8] = ['▁', '▂', '▃', '▄', '▅', '▆', '▇', '█'];

pub fn render_sparkline(values: &[f64]) -> String {
    values.iter().map(|&v| {
        let v = v.clamp(0.0, 100.0);
        let idx = ((v / 100.0) * 7.0) as usize;
        SPARKLINE_CHARS[idx.min(7)]
    }).collect()
}
```

---

## T6.7 — Rendu Braille

### Mapping Unicode Braille

Les caractères Braille Unicode (U+2800 à U+28FF) encodent une grille de 2×4 points par caractère :

```
Caractère Braille = U+2800 + somme des bits activés

Position des dots :          Bits correspondants :
┌─────┬─────┐               ┌──────┬──────┐
│ Dot1│ Dot4│               │ 0x01 │ 0x08 │
│ Dot2│ Dot5│               │ 0x02 │ 0x10 │
│ Dot3│ Dot6│               │ 0x04 │ 0x20 │
│ Dot7│ Dot8│               │ 0x40 │ 0x80 │
└─────┴─────┘               └──────┴──────┘

Colonne gauche : bits {0x01, 0x02, 0x04, 0x40}
Colonne droite : bits {0x08, 0x10, 0x20, 0x80}
```

### Table de mapping Rust

```rust
/// Bits Braille par (colonne, ligne) dans le caractère.
/// braille_dots[col][row] donne le bit à activer.
const BRAILLE_DOTS: [[u32; 4]; 2] = [
    [0x01, 0x02, 0x04, 0x40],  // colonne gauche (col=0)
    [0x08, 0x10, 0x20, 0x80],  // colonne droite (col=1)
];
```

### Algorithme de rendu du graphique Braille

```rust
/// Rend des valeurs (0..100) en graphique Braille multi-lignes.
/// Chaque caractère Braille = 2 colonnes × 4 lignes de la grille de dots.
/// Les valeurs sont alignées à droite (les plus récentes à droite).
pub fn render_braille_chart(values: &[f64], width: usize, rows: usize) -> Vec<String> {
    if width == 0 || rows == 0 || values.is_empty() {
        return Vec::new();
    }

    let dot_rows = rows * 4;       // lignes totales dans la grille de dots
    let dot_cols = width * 2;      // colonnes totales dans la grille de dots

    // Initialiser la grille Braille (tous les caractères = U+2800, vide)
    let mut grid: Vec<Vec<u32>> = vec![vec![0x2800; width]; rows];

    // Aligner les valeurs à droite
    let start_idx = if values.len() > dot_cols {
        values.len() - dot_cols
    } else {
        0
    };

    for (i, &v) in values[start_idx..].iter().enumerate() {
        let dot_col = i + (dot_cols - values[start_idx..].len().min(dot_cols));
        let v = v.clamp(0.0, 100.0);

        // Mapper la valeur vers une ligne de dot (0 = haut, dot_rows-1 = bas)
        let dot_row = (dot_rows - 1) as f64 - (v / 100.0 * (dot_rows - 1) as f64);
        let dot_row = (dot_row as usize).min(dot_rows - 1);

        // Convertir coordonnées dot → cellule caractère + offset
        let char_col = dot_col / 2;
        let char_row = dot_row / 4;
        let sub_col = dot_col % 2;
        let sub_row = dot_row % 4;

        if char_col < width && char_row < rows {
            grid[char_row][char_col] |= BRAILLE_DOTS[sub_col][sub_row];
        }
    }

    // Convertir la grille en chaînes
    grid.iter()
        .map(|row| row.iter().map(|&code| char::from_u32(code).unwrap_or(' ')).collect())
        .collect()
}
```

### Exemple de rendu

```
Valeurs : [10, 30, 60, 80, 95, 70, 40, 20]
Grille 4×2 (8 colonnes dot × 8 lignes dot)

Ligne dot:  0  ─────────── 100%
            1  ────────
            2  ──────          ⠈⠑
            3  ────              ⠠⠄
            4  ──
            5
            6
            7  ─────────── 0%
```

### Calcul de hauteur du graphique

```
hauteur_disponible = chart_h - 2 (bordures) - 2 (titre + barre de progression) - 2 (sparklines)
si hauteur_disponible >= 4 :
    rows_braille = hauteur_disponible / 2  (chaque caractère = 4 dot rows visuellement ~2 lignes)
sinon :
    pas de rendu Braille (fallback aux sparklines simples)
```

---

## T6.8 — Panneau de logs scrollable

### Structure Go d'origine

```go
type LogsModel struct {
    viewport    viewport.Model   // composant Bubbles
    entries     []string
    autoScroll  bool
    width       int
    height      int
    algoNames   []string
}
```

### Spécification Rust

```rust
/// Panneau de logs scrollable avec auto-défilement.
pub struct LogsModel {
    /// Entrées de log formatées (avec styles ANSI).
    entries: Vec<String>,
    /// Offset de défilement vertical (0 = tout en haut).
    scroll_offset: usize,
    /// Auto-scroll activé (suit les dernières entrées).
    auto_scroll: bool,
    /// Dimensions du panneau.
    width: u16,
    height: u16,
    /// Noms des algorithmes pour mapper index → nom.
    algo_names: Vec<String>,
}

const MAX_LOG_ENTRIES: usize = 10_000;
```

### Algorithme de scrolling

```rust
impl LogsModel {
    /// Nombre de lignes visibles dans le viewport.
    fn visible_lines(&self) -> usize {
        (self.height as usize).saturating_sub(2)  // -2 pour les bordures
    }

    /// Nombre total de lignes (peut dépasser entries.len() si du wrapping).
    fn total_lines(&self) -> usize {
        self.entries.len()
    }

    /// Offset maximal de défilement.
    fn max_scroll(&self) -> usize {
        self.total_lines().saturating_sub(self.visible_lines())
    }

    /// Ajouter une entrée avec gestion du cap et auto-scroll.
    pub fn add_entry(&mut self, entry: String) {
        self.entries.push(entry);
        self.trim_entries();
        if self.auto_scroll {
            self.scroll_offset = self.max_scroll();
        }
    }

    /// Tronquer au-delà de MAX_LOG_ENTRIES.
    fn trim_entries(&mut self) {
        if self.entries.len() > MAX_LOG_ENTRIES {
            let excess = self.entries.len() - MAX_LOG_ENTRIES;
            self.entries.drain(..excess);
            self.scroll_offset = self.scroll_offset.saturating_sub(excess);
        }
    }

    /// Défilement vers le haut.
    pub fn scroll_up(&mut self, lines: usize) {
        self.scroll_offset = self.scroll_offset.saturating_sub(lines);
        self.auto_scroll = false;
    }

    /// Défilement vers le bas.
    pub fn scroll_down(&mut self, lines: usize) {
        self.scroll_offset = (self.scroll_offset + lines).min(self.max_scroll());
        // Réactiver l'auto-scroll si on atteint le bas
        if self.scroll_offset >= self.max_scroll() {
            self.auto_scroll = true;
        }
    }

    /// Page up (hauteur du viewport).
    pub fn page_up(&mut self) {
        self.scroll_up(self.visible_lines());
    }

    /// Page down.
    pub fn page_down(&mut self) {
        self.scroll_down(self.visible_lines());
    }
}
```

### Rendu avec ratatui

```rust
impl LogsModel {
    pub fn draw(&self, frame: &mut Frame, area: Rect) {
        let inner = area.inner(Margin::new(1, 1));  // espace pour la bordure

        // Extraire les lignes visibles
        let start = self.scroll_offset;
        let end = (start + inner.height as usize).min(self.entries.len());
        let visible: Vec<Line> = self.entries[start..end]
            .iter()
            .map(|e| Line::raw(e.as_str()))
            .collect();

        let paragraph = Paragraph::new(visible)
            .block(Block::default()
                .borders(Borders::ALL)
                .border_style(Style::default().fg(Color::Rgb(0xFF, 0x66, 0x00)))
                .border_type(BorderType::Rounded));

        frame.render_widget(paragraph, area);
    }
}
```

### Comportement de l'auto-scroll

```
État initial : auto_scroll = true, scroll_offset = 0

Ajout d'entrée :
    entries.push(entry)
    SI auto_scroll : scroll_offset = max_scroll()

Scroll up (↑ / k) :
    scroll_offset -= 1
    auto_scroll = false    ← désactivé manuellement

Scroll down (↓ / j) :
    scroll_offset += 1
    SI scroll_offset >= max_scroll() : auto_scroll = true  ← réactivé

Reset :
    entries.clear()
    scroll_offset = 0
    auto_scroll = true
```

---

## T6.9 — Pipeline de collecte de métriques système

### Architecture Go d'origine

```go
// sysmon/sysmon.go
func Sample() Stats {
    cpuPcts, _ := cpu.Percent(0, false)    // gopsutil
    vmem, _ := mem.VirtualMemory()          // gopsutil
    return Stats{CPUPercent: cpuPcts[0], MemPercent: vmem.UsedPercent}
}
```

Le sampling est déclenché par `TickMsg` toutes les 500ms (pas 1s), via `sampleSysStatsCmd()`.

### Spécification Rust

```rust
// Crate sysinfo (équivalent de gopsutil)
use sysinfo::System;

pub struct SysMonitor {
    sys: System,
}

impl SysMonitor {
    pub fn new() -> Self {
        let mut sys = System::new_all();
        sys.refresh_all();  // premier échantillonnage
        Self { sys }
    }

    /// Collecte un snapshot CPU et mémoire.
    pub fn sample(&mut self) -> SysStats {
        self.sys.refresh_cpu_usage();
        self.sys.refresh_memory();

        let cpu_percent = self.sys.global_cpu_usage() as f64;
        let total_mem = self.sys.total_memory() as f64;
        let used_mem = self.sys.used_memory() as f64;
        let mem_percent = if total_mem > 0.0 {
            (used_mem / total_mem) * 100.0
        } else {
            0.0
        };

        SysStats { cpu_percent, mem_percent }
    }
}

pub struct SysStats {
    pub cpu_percent: f64,  // 0.0..100.0
    pub mem_percent: f64,  // 0.0..100.0
}
```

### Pipeline de collecte

```
┌─────────────────────────────────────────────────────────────┐
│                    Tâche tokio périodique                     │
│                                                              │
│  async fn metrics_sampler(tx: Sender<AppMessage>,            │
│                           cancel: CancellationToken) {       │
│      let mut sys = SysMonitor::new();                        │
│      let mut interval = tokio::time::interval(               │
│          Duration::from_millis(500));                         │
│                                                              │
│      loop {                                                  │
│          tokio::select! {                                    │
│              _ = interval.tick() => {                         │
│                  // Métriques système                         │
│                  let sys_stats = sys.sample();                │
│                  tx.send(AppMessage::SysStats(SysStatsMsg {  │
│                      cpu_percent: sys_stats.cpu_percent,      │
│                      mem_percent: sys_stats.mem_percent,      │
│                  })).ok();                                    │
│                                                              │
│                  // Métriques processus (RSS, threads)        │
│                  let proc_info = get_process_info();          │
│                  tx.send(AppMessage::MemStats(MemStatsMsg {  │
│                      alloc: proc_info.rss,                    │
│                      heap_sys: proc_info.vms,                 │
│                      num_gc: 0,  // N/A en Rust               │
│                      pause_total_ns: 0,                       │
│                      num_threads: proc_info.threads,          │
│                  })).ok();                                    │
│              }                                               │
│              _ = cancel.cancelled() => break,                 │
│          }                                                   │
│      }                                                       │
│  }                                                           │
└─────────────────────────────────────────────────────────────┘
```

### Différences Go vs Rust pour les métriques runtime

| Métrique     | Go                                | Rust                                                         |
| ------------- | --------------------------------- | ------------------------------------------------------------ |
| Heap utilisé | `runtime.MemStats.Alloc`        | RSS processus via `sysinfo`                                |
| Heap système | `runtime.MemStats.HeapSys`      | VMS processus                                                |
| Cycles GC     | `runtime.MemStats.NumGC`        | 0 (pas de GC)                                                |
| Pause GC      | `runtime.MemStats.PauseTotalNs` | 0 (pas de GC)                                                |
| Goroutines    | `runtime.NumGoroutine()`        | Nombre de threads (via `sysinfo` ou `/proc/self/status`) |

### Mapping crate

| Dépendance Go                        | Crate Rust           | Usage                |
| ------------------------------------- | -------------------- | -------------------- |
| `github.com/shirou/gopsutil/v4/cpu` | `sysinfo`          | CPU% global          |
| `github.com/shirou/gopsutil/v4/mem` | `sysinfo`          | MEM% et RSS          |
| `runtime.ReadMemStats()`            | `sysinfo::Process` | Métriques processus |

---

## T6.10 — TUI Bridge (canal de progrès → messages ratatui)

### Architecture du Bridge

Le bridge convertit les événements de l'orchestration (progrès, résultats, erreurs) en messages `AppMessage` envoyés au thread principal du TUI via un canal `mpsc`.

### Équivalent Go

```go
// bridge.go
type TUIProgressReporter struct { ref *programRef }
type TUIResultPresenter struct { ref *programRef }
```

### Spécification Rust

```rust
/// Reporter de progrès qui transmet les mises à jour au TUI via un canal.
pub struct TuiProgressReporter {
    tx: mpsc::UnboundedSender<AppMessage>,
}

impl ProgressReporter for TuiProgressReporter {
    fn report_progress(&self, update: ProgressUpdate) {
        // Calculer ETA et progrès moyen
        // (logique portée depuis format::ProgressWithETA)
        self.tx.send(AppMessage::Progress(ProgressMsg {
            calculator_index: update.calculator_index,
            value: update.value,
            average_progress: update.average_progress,
            eta: update.eta,
        })).ok();
    }

    fn report_done(&self) {
        self.tx.send(AppMessage::ProgressDone).ok();
    }
}

/// Présentateur de résultats qui transmet les résultats au TUI.
pub struct TuiResultPresenter {
    tx: mpsc::UnboundedSender<AppMessage>,
}

impl ResultPresenter for TuiResultPresenter {
    fn present_comparison_table(&self, results: &[CalculationResult]) {
        self.tx.send(AppMessage::ComparisonResults(ComparisonResultsMsg {
            results: results.to_vec(),
        })).ok();
    }

    fn present_result(&self, result: &CalculationResult, n: u64,
                      verbose: bool, details: bool, show_value: bool) {
        self.tx.send(AppMessage::FinalResult(FinalResultMsg {
            result: result.clone(),
            n,
            verbose,
            details,
            show_value,
        })).ok();
    }

    fn handle_error(&self, err: &dyn std::error::Error, duration: Duration) -> i32 {
        self.tx.send(AppMessage::Error(ErrorMsg {
            err: err.to_string(),
            duration,
        })).ok();
        // Retourner le code d'erreur approprié
        exit_code_from_error(err)
    }

    fn format_duration(&self, d: Duration) -> String {
        format_execution_duration(d)
    }
}
```

### Diagramme complet du flux de données

```
┌───────────────────────────────────────────────────────────────────────┐
│                        Thread de calcul (tokio::spawn)                │
│                                                                       │
│  orchestration::execute_calculations()                                │
│       │                                                               │
│       ├── Calculator::calculate()                                     │
│       │       │                                                       │
│       │       ├── progress_channel.send(ProgressUpdate)               │
│       │       │       │                                               │
│       │       │       ▼                                               │
│       │       │  TuiProgressReporter::report_progress()               │
│       │       │       │                                               │
│       │       │       ▼                                               │
│       │       │  tx.send(AppMessage::Progress(...))  ──────┐          │
│       │       │                                            │          │
│       │       └── (résultat)                               │          │
│       │                                                    │          │
│       ├── analyze_comparison_results()                     │          │
│       │       │                                            │          │
│       │       ├── TuiResultPresenter::present_*()          │          │
│       │       │       │                                    │          │
│       │       │       ▼                                    │          │
│       │       │  tx.send(AppMessage::FinalResult(...)) ────┤          │
│       │       │                                            │          │
│       └── tx.send(AppMessage::CalculationComplete) ────────┤          │
│                                                            │          │
└────────────────────────────────────────────────────────────┤──────────┘
                                                             │
                                                             ▼
┌────────────────────────────────────────────────────────────┐
│                  mpsc::UnboundedReceiver                    │
│               (thread principal / boucle TUI)              │
│                                                            │
│  while let Ok(msg) = message_rx.try_recv() {              │
│      match msg {                                           │
│          Progress(p) → {                                   │
│              logs.add_progress_entry(p);                    │
│              chart.add_data_point(p);                       │
│              metrics.update_progress(p);                    │
│          }                                                 │
│          FinalResult(r) → logs.add_final_result(r);        │
│          ComparisonResults(c) → logs.add_results(c);       │
│          CalculationComplete(c) → {                        │
│              if c.generation == self.generation {           │
│                  self.done = true; ...                      │
│              }                                             │
│          }                                                 │
│          MemStats(m) → metrics.update_mem_stats(m);        │
│          SysStats(s) → chart.update_sys_stats(s);          │
│          Indicators(i) → metrics.update_indicators(i);     │
│          ...                                               │
│      }                                                     │
│  }                                                         │
│                                                            │
│  terminal.draw(|f| app.draw(f));  // rendu ratatui          │
└────────────────────────────────────────────────────────────┘
```

### Gestion du calcul asynchrone des indicateurs

Le calcul des indicateurs post-résultat (bits/s, digits/s, etc.) est effectué de manière asynchrone pour ne pas bloquer le thread UI :

```rust
// Lors de la réception de FinalResult, si result n'est pas None :
if msg.result.result.is_some() {
    let tx = self.message_tx.clone();
    let result = msg.result.clone();
    let n = msg.n;
    tokio::spawn(async move {
        let indicators = metrics::compute(&result.result.unwrap(), n, result.duration);
        tx.send(AppMessage::Indicators(IndicatorsMsg { indicators })).ok();
    });
}
```

### Résumé des dépendances crate pour le TUI

| Crate                      | Version | Usage                                                 |
| -------------------------- | ------- | ----------------------------------------------------- |
| `ratatui`                | ≥ 0.28 | Framework TUI (Layout, Frame, Widget)                 |
| `crossterm`              | ≥ 0.28 | Backend terminal (events, raw mode, alternate screen) |
| `tokio`                  | ≥ 1.0  | Runtime async (spawn, channels, timers)               |
| `sysinfo`                | ≥ 0.31 | Métriques système (CPU%, MEM%, RSS)                 |
| `chrono`                 | ≥ 0.4  | Horodatage (calibration profiles)                     |
| `serde` / `serde_json` | ≥ 1.0  | Sérialisation profils calibration                    |
| `num-cpus`               | ≥ 1.16 | Détection nombre de CPUs                             |
| `tokio-util`             | ≥ 0.7  | `CancellationToken`                                 |

---

## Résumé des correspondances Phase 5-6

| Tâche | Fichier Go source                                      | Module Rust cible                | Complexité |
| ------ | ------------------------------------------------------ | -------------------------------- | ----------- |
| T5.1   | `fibonacci/dynamic_threshold.go`                     | `fibonacci::dynamic_threshold` | Moyenne     |
| T5.2   | `fibonacci/dynamic_threshold.go` (significantChange) | `fibonacci::dynamic_threshold` | Faible      |
| T5.3   | `fibonacci/threshold_types.go`                       | `fibonacci::threshold_types`   | Faible      |
| T5.4   | `fibonacci/dynamic_threshold.go` (analyze*)          | `fibonacci::dynamic_threshold` | Moyenne     |
| T5.5   | `calibration/profile.go`                             | `calibration::profile`         | Faible      |
| T5.6   | `calibration/calibration.go`                         | `calibration::mod`             | Élevée    |
| T5.7   | `calibration/adaptive.go`                            | `calibration::adaptive`        | Faible      |
| T5.8   | `calibration/microbench.go`                          | `calibration::microbench`      | Moyenne     |
| T6.1   | `tui/model.go`                                       | `tui::app`                     | Élevée    |
| T6.2   | `tui/model.go` (generation)                          | `tui::app`                     | Faible      |
| T6.3   | `tui/bridge.go` (programRef)                         | `tui::bridge` (mpsc)           | Faible      |
| T6.4   | `tui/messages.go`                                    | `tui::messages`                | Faible      |
| T6.5   | `tui/model.go` (layoutPanels)                        | `tui::layout`                  | Moyenne     |
| T6.6   | `tui/sparkline.go`                                   | `tui::sparkline`               | Faible      |
| T6.7   | `tui/sparkline.go` (RenderBrailleChart)              | `tui::braille`                 | Moyenne     |
| T6.8   | `tui/logs.go`                                        | `tui::logs`                    | Moyenne     |
| T6.9   | `sysmon/sysmon.go`                                   | `sysmon::mod`                  | Faible      |
| T6.10  | `tui/bridge.go`                                      | `tui::bridge`                  | Moyenne     |

# Phase 7 — Intégration, Tests & Finalisation

> **Portage FibGo (Go) → FibRust (Rust)**
> Phase 7 couvre les tâches T7.1 à T7.8 : cartographie exhaustive des fichiers, contrats de traits, diagrammes de flux de données, catalogue de cas limites, carte de propagation d'erreurs, spécification FFI/GMP, scénarios de tests d'intégration et structure documentaire.

---

## Table des matières

- [T7.1 — Cartographie fichier par fichier Go → Rust](#t71--cartographie-fichier-par-fichier-go--rust)
- [T7.2 — Contrats de Traits avec Pré/Postconditions](#t72--contrats-de-traits-avec-préconditions)
- [T7.3 — Diagrammes de Flux de Données (Rust)](#t73--diagrammes-de-flux-de-données-rust)
- [T7.4 — Catalogue de Cas Limites](#t74--catalogue-de-cas-limites)
- [T7.5 — Carte de Propagation des Erreurs](#t75--carte-de-propagation-des-erreurs)
- [T7.6 — Spécification FFI (GMP / rug)](#t76--spécification-ffi-gmp--rug)
- [T7.7 — Scénarios de Tests d&#39;Intégration](#t77--scénarios-de-tests-dintégration)
- [T7.8 — Structure Documentaire du Projet Rust](#t78--structure-documentaire-du-projet-rust)

---

## T7.1 — Cartographie fichier par fichier Go → Rust

### 7.1.1 Architecture Crate Rust

Le workspace Cargo contient **7 crates** :

| Crate                     | Type    | Description                                                                                        |
| ------------------------- | ------- | -------------------------------------------------------------------------------------------------- |
| `fibcalc-core`          | `lib` | Algorithmes Fibonacci, stratégies, observateurs, seuils dynamiques, arène mémoire, contrôle GC |
| `fibcalc-bigfft`        | `lib` | Multiplication FFT, nombres de Fermat, cache de transformées, allocateurs, pools                  |
| `fibcalc-orchestration` | `lib` | Exécution parallèle, sélection de calculatrices, analyse de résultats                          |
| `fibcalc-cli`           | `lib` | Sortie CLI, présentateur, barre de progression, ETA, complétion shell                            |
| `fibcalc-tui`           | `lib` | Dashboard TUI Ratatui (modèle Elm), panneaux, graphiques, sparklines                              |
| `fibcalc-calibration`   | `lib` | Calibration, benchmarks, profils adaptatifs, micro-benchmarks                                      |
| `fibcalc`               | `bin` | Point d'entrée binaire, application, configuration, gestion des erreurs                           |

### 7.1.2 Table de correspondance exhaustive

#### Crate `fibcalc` (binaire principal)

| Fichier Go                                    | Crate Rust  | Fichier Rust                             | Priorité | Notes de migration                                           |
| --------------------------------------------- | ----------- | ---------------------------------------- | --------- | ------------------------------------------------------------ |
| `cmd/fibcalc/main.go`                       | `fibcalc` | `src/main.rs`                          | P0        | Point d'entrée `fn main()`, appel à `app::run()`       |
| `cmd/fibcalc/main_test.go`                  | `fibcalc` | `tests/main_test.rs`                   | P1        | Tests d'intégration du binaire                              |
| `cmd/generate-golden/main.go`               | `fibcalc` | `src/bin/generate_golden.rs`           | P2        | Binaire séparé pour données de référence                |
| `cmd/generate-golden/main_test.go`          | `fibcalc` | `tests/generate_golden_test.rs`        | P2        | Tests du générateur golden                                 |
| `internal/app/app.go`                       | `fibcalc` | `src/app.rs`                           | P0        | `Application` struct, `new()`, `run()`, dispatch modes |
| `internal/app/version.go`                   | `fibcalc` | `src/version.rs`                       | P0        | Constantes de version via `build.rs` ou `env!()`         |
| `internal/app/doc.go`                       | `fibcalc` | —                                       | —        | Pas d'équivalent Rust (doc dans `//!` du `lib.rs`)      |
| `internal/app/app_test.go`                  | `fibcalc` | `tests/app_test.rs`                    | P1        | Tests d'intégration                                         |
| `internal/app/version_test.go`              | `fibcalc` | `tests/version_test.rs`                | P2        | Tests version                                                |
| `internal/config/config.go`                 | `fibcalc` | `src/config.rs`                        | P0        | `AppConfig`, parsing via `clap`                          |
| `internal/config/env.go`                    | `fibcalc` | `src/config/env.rs`                    | P1        | Variables d'environnement `FIBCALC_*`                      |
| `internal/config/usage.go`                  | `fibcalc` | `src/config/usage.rs`                  | P2        | Aide personnalisée (intégrée dans `clap`)               |
| `internal/config/doc.go`                    | `fibcalc` | —                                       | —        | Doc module Rust                                              |
| `internal/config/config_test.go`            | `fibcalc` | `tests/config_test.rs`                 | P1        | Tests config                                                 |
| `internal/config/config_exhaustive_test.go` | `fibcalc` | `tests/config_exhaustive_test.rs`      | P2        | Tests exhaustifs config                                      |
| `internal/config/config_extra_test.go`      | `fibcalc` | `tests/config_extra_test.rs`           | P2        | Tests supplémentaires                                       |
| `internal/config/env_test.go`               | `fibcalc` | `tests/config_env_test.rs`             | P1        | Tests env vars                                               |
| `internal/errors/errors.go`                 | `fibcalc` | `src/errors.rs`                        | P0        | `FibError` enum avec `thiserror`                         |
| `internal/errors/handler.go`                | `fibcalc` | `src/errors/handler.rs`                | P0        | `HandleCalculationError`, codes de sortie                  |
| `internal/errors/doc.go`                    | `fibcalc` | —                                       | —        | Doc module                                                   |
| `internal/errors/errors_test.go`            | `fibcalc` | `src/errors.rs` (tests inline)         | P1        | `#[cfg(test)] mod tests`                                   |
| `internal/errors/handler_test.go`           | `fibcalc` | `src/errors/handler.rs` (tests inline) | P1        | Tests handler                                                |
| `internal/format/duration.go`               | `fibcalc` | `src/format/duration.rs`               | P1        | Formatage durées                                            |
| `internal/format/numbers.go`                | `fibcalc` | `src/format/numbers.rs`                | P1        | Formatage nombres                                            |
| `internal/format/progress_eta.go`           | `fibcalc` | `src/format/progress_eta.rs`           | P1        | Affichage ETA                                                |
| `internal/format/progress_eta_test.go`      | `fibcalc` | `src/format/progress_eta.rs` (tests)   | P2        | Tests ETA                                                    |
| `internal/metrics/indicators.go`            | `fibcalc` | `src/metrics/indicators.rs`            | P2        | Indicateurs performance (bits/s, digits/s)                   |
| `internal/metrics/memory.go`                | `fibcalc` | `src/metrics/memory.rs`                | P2        | `MemoryCollector`, `MemorySnapshot`                      |
| `internal/metrics/indicators_test.go`       | `fibcalc` | `src/metrics/indicators.rs` (tests)    | P2        | Tests indicateurs                                            |
| `internal/metrics/memory_test.go`           | `fibcalc` | `src/metrics/memory.rs` (tests)        | P2        | Tests mémoire                                               |
| `internal/parallel/errors.go`               | `fibcalc` | `src/parallel/errors.rs`               | P1        | `ErrorCollector` → Rust `Result` + `rayon`            |
| `internal/parallel/doc.go`                  | `fibcalc` | —                                       | —        | Doc module                                                   |
| `internal/parallel/errors_test.go`          | `fibcalc` | `src/parallel/errors.rs` (tests)       | P1        | Tests ErrorCollector                                         |
| `internal/sysmon/sysmon.go`                 | `fibcalc` | `src/sysmon.rs`                        | P3        | Monitoring CPU/mémoire via `sysinfo`                      |
| `internal/sysmon/sysmon_test.go`            | `fibcalc` | `src/sysmon.rs` (tests)                | P3        | Tests sysmon                                                 |
| `internal/testutil/ansi.go`                 | `fibcalc` | `src/testutil.rs`                      | P2        | Nettoyage ANSI pour assertions test                          |
| `internal/testutil/doc.go`                  | `fibcalc` | —                                       | —        | Doc module                                                   |
| `internal/testutil/ansi_test.go`            | `fibcalc` | `src/testutil.rs` (tests)              | P2        | Tests ANSI                                                   |
| `internal/ui/colors.go`                     | `fibcalc` | `src/ui/colors.rs`                     | P1        | Couleurs terminales,`NO_COLOR`                             |
| `internal/ui/themes.go`                     | `fibcalc` | `src/ui/themes.rs`                     | P1        | Thèmes (`ColorTheme`)                                     |
| `internal/ui/doc.go`                        | `fibcalc` | —                                       | —        | Doc module                                                   |
| `internal/ui/themes_test.go`                | `fibcalc` | `src/ui/themes.rs` (tests)             | P2        | Tests thèmes                                                |

#### Crate `fibcalc-core`

| Fichier Go                                    | Crate Rust       | Fichier Rust                         | Priorité | Notes de migration                                                                                                |
| --------------------------------------------- | ---------------- | ------------------------------------ | --------- | ----------------------------------------------------------------------------------------------------------------- |
| `internal/fibonacci/calculator.go`          | `fibcalc-core` | `src/calculator.rs`                | P0        | Trait `Calculator`, `FibCalculator` décorateur, `calculateSmall()`                                         |
| `internal/fibonacci/strategy.go`            | `fibcalc-core` | `src/strategy.rs`                  | P0        | Traits `Multiplier`, `DoublingStepExecutor`; `AdaptiveStrategy`, `FFTOnlyStrategy`, `KaratsubaStrategy` |
| `internal/fibonacci/observer.go`            | `fibcalc-core` | `src/observer.rs`                  | P0        | Trait `ProgressObserver`, `ProgressSubject` avec `Freeze()`                                                 |
| `internal/fibonacci/observers.go`           | `fibcalc-core` | `src/observers.rs`                 | P1        | `ChannelObserver`, `LoggingObserver`, `NoOpObserver`                                                        |
| `internal/fibonacci/registry.go`            | `fibcalc-core` | `src/registry.rs`                  | P0        | Trait `CalculatorFactory`, `DefaultFactory` avec `RwLock<HashMap>`                                          |
| `internal/fibonacci/generator.go`           | `fibcalc-core` | `src/generator.rs`                 | P1        | Trait `SequenceGenerator`                                                                                       |
| `internal/fibonacci/generator_iterative.go` | `fibcalc-core` | `src/generator_iterative.rs`       | P1        | `IterativeGenerator` implémentation                                                                            |
| `internal/fibonacci/fastdoubling.go`        | `fibcalc-core` | `src/fastdoubling.rs`              | P0        | `OptimizedFastDoubling`, `CalculationState`, `statePool` → pools Rust                                      |
| `internal/fibonacci/matrix.go`              | `fibcalc-core` | `src/matrix.rs`                    | P0        | `MatrixExponentiation`, `matrixState` pool                                                                    |
| `internal/fibonacci/fft_based.go`           | `fibcalc-core` | `src/fft_based.rs`                 | P0        | `FFTBasedCalculator`                                                                                            |
| `internal/fibonacci/doubling_framework.go`  | `fibcalc-core` | `src/doubling_framework.rs`        | P0        | `DoublingFramework`, `ExecuteDoublingLoop()`                                                                  |
| `internal/fibonacci/matrix_framework.go`    | `fibcalc-core` | `src/matrix_framework.rs`          | P0        | `MatrixFramework`, `ExecuteMatrixLoop()`                                                                      |
| `internal/fibonacci/matrix_ops.go`          | `fibcalc-core` | `src/matrix_ops.rs`                | P0        | Multiplications matricielles, Strassen                                                                            |
| `internal/fibonacci/matrix_types.go`        | `fibcalc-core` | `src/matrix_types.rs`              | P0        | Types `Matrix`, `MatrixState`                                                                                 |
| `internal/fibonacci/fft.go`                 | `fibcalc-core` | `src/fft_wrappers.rs`              | P0        | `mulFFT`, `sqrFFT`, `smartMultiply`, `smartSquare`, `executeDoublingStepFFT`                            |
| `internal/fibonacci/common.go`              | `fibcalc-core` | `src/common.rs`                    | P0        | `taskSemaphore` → Rayon, `executeTasks` générique, pools                                                   |
| `internal/fibonacci/constants.go`           | `fibcalc-core` | `src/constants.rs`                 | P0        | Constantes de seuils,`pub const`                                                                                |
| `internal/fibonacci/options.go`             | `fibcalc-core` | `src/options.rs`                   | P0        | `Options` struct, `normalizeOptions()`                                                                        |
| `internal/fibonacci/progress.go`            | `fibcalc-core` | `src/progress.rs`                  | P0        | `ProgressUpdate`, `ProgressCallback`, `CalcTotalWork()`, `PrecomputePowers4()`                            |
| `internal/fibonacci/dynamic_threshold.go`   | `fibcalc-core` | `src/dynamic_threshold.rs`         | P1        | `DynamicThresholdManager` avec ring buffer                                                                      |
| `internal/fibonacci/threshold_types.go`     | `fibcalc-core` | `src/threshold_types.rs`           | P1        | `IterationMetric`, `ThresholdStats`, `DynamicThresholdConfig`                                               |
| `internal/fibonacci/arena.go`               | `fibcalc-core` | `src/arena.rs`                     | P1        | `CalculationArena` → bump allocator Rust                                                                       |
| `internal/fibonacci/gc_control.go`          | `fibcalc-core` | `src/gc_control.rs`                | P2        | Pas de GC en Rust — stub ou métriques mémoire                                                                  |
| `internal/fibonacci/memory_budget.go`       | `fibcalc-core` | `src/memory_budget.rs`             | P1        | `MemoryEstimate`, `EstimateMemoryUsage()`, `ParseMemoryLimit()`                                             |
| `internal/fibonacci/modular.go`             | `fibcalc-core` | `src/modular.rs`                   | P1        | `FastDoublingMod` pour `--last-digits`                                                                        |
| `internal/fibonacci/testing.go`             | `fibcalc-core` | `src/testing.rs`                   | P1        | `MockCalculator`, `TestFactory` (cfg(test) ou pub)                                                            |
| `internal/fibonacci/doc.go`                 | `fibcalc-core` | `src/lib.rs`                       | —        | Documentation `//!` module racine                                                                               |
| `internal/fibonacci/calculator_gmp.go`      | `fibcalc-core` | `src/calculator_gmp.rs`            | P2        | Feature `gmp`, `rug::Integer`, `#[cfg(feature = "gmp")]`                                                    |
| Tous les `*_test.go` fibonacci              | `fibcalc-core` | `tests/*.rs` + `src/*.rs` inline | P1-P2     | 28 fichiers de test →`#[cfg(test)]` + `tests/`                                                               |

#### Crate `fibcalc-bigfft`

| Fichier Go                           | Crate Rust         | Fichier Rust             | Priorité | Notes de migration                                                   |
| ------------------------------------ | ------------------ | ------------------------ | --------- | -------------------------------------------------------------------- |
| `internal/bigfft/fft.go`           | `fibcalc-bigfft` | `src/fft.rs`           | P0        | API publique `Mul`, `Sqr`, `MulTo`, `SqrTo`                  |
| `internal/bigfft/fft_core.go`      | `fibcalc-bigfft` | `src/fft_core.rs`      | P0        | Noyau FFT, transformées directe/inverse                             |
| `internal/bigfft/fft_recursion.go` | `fibcalc-bigfft` | `src/fft_recursion.rs` | P0        | Récursion FFT, parallélisme interne                                |
| `internal/bigfft/fft_poly.go`      | `fibcalc-bigfft` | `src/fft_poly.rs`      | P0        | Opérations polynomiales pour FFT                                    |
| `internal/bigfft/fft_cache.go`     | `fibcalc-bigfft` | `src/fft_cache.rs`     | P1        | Cache LRU thread-safe pour transformées                             |
| `internal/bigfft/fermat.go`        | `fibcalc-bigfft` | `src/fermat.rs`        | P0        | Arithmétique nombres de Fermat                                      |
| `internal/bigfft/pool.go`          | `fibcalc-bigfft` | `src/pool.rs`          | P1        | Pools `BigInt`, recyclage objets                                   |
| `internal/bigfft/pool_warming.go`  | `fibcalc-bigfft` | `src/pool_warming.rs`  | P2        | Pré-chauffage des pools                                             |
| `internal/bigfft/bump.go`          | `fibcalc-bigfft` | `src/bump.rs`          | P0        | `BumpAllocator` O(1)                                               |
| `internal/bigfft/allocator.go`     | `fibcalc-bigfft` | `src/allocator.rs`     | P0        | Trait `TempAllocator`, `PoolAllocator`, `BumpAllocatorAdapter` |
| `internal/bigfft/memory_est.go`    | `fibcalc-bigfft` | `src/memory_est.rs`    | P1        | Estimation mémoire FFT                                              |
| `internal/bigfft/scan.go`          | `fibcalc-bigfft` | `src/scan.rs`          | P1        | Utilitaires de scan                                                  |
| `internal/bigfft/arith_amd64.go`   | `fibcalc-bigfft` | `src/arith_amd64.rs`   | P1        | `go:linkname` → FFI directe ou crate `num-bigint`               |
| `internal/bigfft/arith_generic.go` | `fibcalc-bigfft` | `src/arith_generic.rs` | P0        | Implémentations portables                                           |
| `internal/bigfft/arith_decl.go`    | `fibcalc-bigfft` | `src/arith_decl.rs`    | P1        | Déclarations d'arithmétique vectorielle                            |
| `internal/bigfft/cpu_amd64.go`     | `fibcalc-bigfft` | `src/cpu_detect.rs`    | P2        | Détection CPU AVX2/AVX-512 via `std::arch`                        |
| `internal/bigfft/doc.go`           | `fibcalc-bigfft` | `src/lib.rs`           | —        | Documentation `//!`                                                |
| Tous les `*_test.go` bigfft        | `fibcalc-bigfft` | `tests/*.rs` + inline  | P1-P2     | 12 fichiers de test                                                  |

#### Crate `fibcalc-orchestration`

| Fichier Go                                         | Crate Rust                | Fichier Rust                    | Priorité | Notes de migration                                                                   |
| -------------------------------------------------- | ------------------------- | ------------------------------- | --------- | ------------------------------------------------------------------------------------ |
| `internal/orchestration/orchestrator.go`         | `fibcalc-orchestration` | `src/orchestrator.rs`         | P0        | `ExecuteCalculations()`, `AnalyzeComparisonResults()` via `tokio` ou `rayon` |
| `internal/orchestration/interfaces.go`           | `fibcalc-orchestration` | `src/interfaces.rs`           | P0        | Traits `ProgressReporter`, `ResultPresenter`, `NullProgressReporter`           |
| `internal/orchestration/calculator_selection.go` | `fibcalc-orchestration` | `src/calculator_selection.rs` | P0        | `GetCalculatorsToRun()`                                                            |
| `internal/orchestration/doc.go`                  | `fibcalc-orchestration` | `src/lib.rs`                  | —        | Documentation module                                                                 |
| Tous les `*_test.go` orchestration               | `fibcalc-orchestration` | `tests/*.rs`                  | P1        | 3 fichiers de test                                                                   |

#### Crate `fibcalc-cli`

| Fichier Go                       | Crate Rust      | Fichier Rust            | Priorité | Notes de migration                                |
| -------------------------------- | --------------- | ----------------------- | --------- | ------------------------------------------------- |
| `internal/cli/output.go`       | `fibcalc-cli` | `src/output.rs`       | P0        | `Display*`, `Format*`, `Write*`, `Print*` |
| `internal/cli/presenter.go`    | `fibcalc-cli` | `src/presenter.rs`    | P0        | `CLIResultPresenter`, `CLIProgressReporter`   |
| `internal/cli/calculate.go`    | `fibcalc-cli` | `src/calculate.rs`    | P1        | Helpers d'affichage résultats                    |
| `internal/cli/progress_eta.go` | `fibcalc-cli` | `src/progress_eta.rs` | P1        | Calcul ETA, affichage progression                 |
| `internal/cli/completion.go`   | `fibcalc-cli` | `src/completion.rs`   | P2        | Complétion shell via `clap_complete`           |
| `internal/cli/provider.go`     | `fibcalc-cli` | `src/provider.rs`     | P1        | Fournisseurs progress reporter et config display  |
| `internal/cli/ui.go`           | `fibcalc-cli` | `src/ui.rs`           | P1        | Helpers UI (spinners via `indicatif`)           |
| `internal/cli/ui_display.go`   | `fibcalc-cli` | `src/ui_display.rs`   | P1        | Affichage formaté                                |
| `internal/cli/ui_format.go`    | `fibcalc-cli` | `src/ui_format.rs`    | P1        | Formatage UI                                      |
| `internal/cli/doc.go`          | `fibcalc-cli` | `src/lib.rs`          | —        | Documentation module                              |
| Tous les `*_test.go` cli       | `fibcalc-cli` | `tests/*.rs` + inline | P1-P2     | 8 fichiers de test                                |

#### Crate `fibcalc-tui`

| Fichier Go                    | Crate Rust      | Fichier Rust            | Priorité | Notes de migration                              |
| ----------------------------- | --------------- | ----------------------- | --------- | ----------------------------------------------- |
| `internal/tui/model.go`     | `fibcalc-tui` | `src/model.rs`        | P1        | `Model` Elm architecture → Ratatui           |
| `internal/tui/bridge.go`    | `fibcalc-tui` | `src/bridge.rs`       | P1        | `TUIProgressReporter`, `TUIResultPresenter` |
| `internal/tui/header.go`    | `fibcalc-tui` | `src/header.rs`       | P2        | Panneau header                                  |
| `internal/tui/footer.go`    | `fibcalc-tui` | `src/footer.rs`       | P2        | Panneau footer                                  |
| `internal/tui/logs.go`      | `fibcalc-tui` | `src/logs.rs`         | P2        | Panneau logs scrollable                         |
| `internal/tui/metrics.go`   | `fibcalc-tui` | `src/metrics.rs`      | P2        | Panneau métriques runtime                      |
| `internal/tui/chart.go`     | `fibcalc-tui` | `src/chart.rs`        | P2        | Graphique de progression                        |
| `internal/tui/sparkline.go` | `fibcalc-tui` | `src/sparkline.rs`    | P2        | Visualisation sparkline                         |
| `internal/tui/styles.go`    | `fibcalc-tui` | `src/styles.rs`       | P2        | Styles Ratatui                                  |
| `internal/tui/keymap.go`    | `fibcalc-tui` | `src/keymap.rs`       | P2        | Raccourcis clavier                              |
| `internal/tui/messages.go`  | `fibcalc-tui` | `src/messages.rs`     | P2        | Types de messages Elm                           |
| `internal/tui/doc.go`       | `fibcalc-tui` | `src/lib.rs`          | —        | Documentation module                            |
| Tous les `*_test.go` tui    | `fibcalc-tui` | `tests/*.rs` + inline | P2        | 10 fichiers de test                             |

#### Crate `fibcalc-calibration`

| Fichier Go                              | Crate Rust              | Fichier Rust            | Priorité | Notes de migration                 |
| --------------------------------------- | ----------------------- | ----------------------- | --------- | ---------------------------------- |
| `internal/calibration/calibration.go` | `fibcalc-calibration` | `src/calibration.rs`  | P2        | Mode calibration complet           |
| `internal/calibration/runner.go`      | `fibcalc-calibration` | `src/runner.rs`       | P2        | Exécuteur de benchmarks           |
| `internal/calibration/adaptive.go`    | `fibcalc-calibration` | `src/adaptive.rs`     | P1        | Estimation adaptative des seuils   |
| `internal/calibration/profile.go`     | `fibcalc-calibration` | `src/profile.rs`      | P2        | Profil de calibration (serde JSON) |
| `internal/calibration/io.go`          | `fibcalc-calibration` | `src/io.rs`           | P2        | Persistance profil                 |
| `internal/calibration/microbench.go`  | `fibcalc-calibration` | `src/microbench.rs`   | P2        | Micro-benchmarks                   |
| `internal/calibration/doc.go`         | `fibcalc-calibration` | `src/lib.rs`          | —        | Documentation module               |
| Tous les `*_test.go` calibration      | `fibcalc-calibration` | `tests/*.rs` + inline | P2        | 6 fichiers de test                 |

#### Fichiers E2E et Données de Test

| Fichier Go                                            | Crate Rust       | Fichier Rust                       | Priorité | Notes de migration                      |
| ----------------------------------------------------- | ---------------- | ---------------------------------- | --------- | --------------------------------------- |
| `test/e2e/cli_e2e_test.go`                          | `fibcalc`      | `tests/e2e/cli_e2e_test.rs`      | P1        | Tests E2E via `assert_cmd`            |
| `internal/fibonacci/testdata/fibonacci_golden.json` | `fibcalc-core` | `testdata/fibonacci_golden.json` | P0        | Données de référence (copie directe) |

### 7.1.3 Résumé des priorités

| Priorité    | Nombre de fichiers | Description                                                      |
| ------------ | ------------------ | ---------------------------------------------------------------- |
| **P0** | 35                 | Noyau algorithmique, interfaces, framework, point d'entrée      |
| **P1** | 38                 | Observateurs, calibration adaptative, formatage, tests critiques |
| **P2** | 30                 | TUI, calibration complète, complétion shell, GMP               |
| **P3** | 2                  | Monitoring système                                              |

---

## T7.2 — Contrats de Traits avec Pré/Postconditions

### 7.2.1 Trait `Calculator`

```rust
/// Calculatrice Fibonacci publique.
///
/// Ce trait est l'abstraction principale consommée par la couche d'orchestration
/// pour interagir avec différents algorithmes de calcul Fibonacci.
pub trait Calculator: Send + Sync {
    /// Calcule le n-ième nombre de Fibonacci.
    ///
    /// # Préconditions
    /// - `ctx` n'est pas annulé au moment de l'appel
    /// - `progress_tx` est un émetteur valide (ou None pour ignorer la progression)
    /// - `calc_index` ≥ 0
    /// - `opts` est normalisé (les seuils à 0 seront remplacés par les défauts)
    ///
    /// # Postconditions
    /// - Si `Ok(result)` : `result` == F(n) mathématiquement exact
    /// - Si `Ok(result)` et n ≤ 93 : le résultat tient dans un u64
    /// - Si `Ok(result)` : le dernier `progress` envoyé est 1.0
    /// - Si `Err(FibError::Cancelled)` : le contexte a été annulé
    /// - Si `Err(FibError::Timeout)` : le délai a expiré
    ///
    /// # Invariants
    /// - Thread-safe : peut être appelé concurremment depuis plusieurs threads
    /// - Aucune mutation d'état partagé entre appels
    /// - La mémoire intermédiaire est libérée après retour
    fn calculate(
        &self,
        ctx: &CancellationToken,
        progress_tx: Option<mpsc::Sender<ProgressUpdate>>,
        calc_index: usize,
        n: u64,
        opts: &Options,
    ) -> Result<BigUint, FibError>;

    /// Retourne le nom d'affichage de l'algorithme.
    ///
    /// # Postconditions
    /// - La chaîne retournée est non vide
    /// - La chaîne est déterministe (même valeur à chaque appel)
    fn name(&self) -> &str;
}
```

### 7.2.2 Trait `Multiplier`

```rust
/// Interface étroite pour les opérations de multiplication et de mise au carré.
///
/// Les consommateurs qui n'ont besoin que de Multiply/Square doivent dépendre
/// de Multiplier plutôt que du trait plus large DoublingStepExecutor.
pub trait Multiplier: Send + Sync {
    /// Calcule x × y.
    ///
    /// # Préconditions
    /// - `x` et `y` sont des entiers non-négatifs valides
    /// - `opts.fft_threshold` ≥ 0
    ///
    /// # Postconditions
    /// - `Ok(result)` == x × y mathématiquement exact
    /// - Le résultat est non-négatif
    ///
    /// # Invariants
    /// - Pas de mutation de `x` ou `y`
    /// - Thread-safe
    fn multiply(&self, x: &BigUint, y: &BigUint, opts: &Options) -> Result<BigUint, FibError>;

    /// Calcule x².
    ///
    /// # Préconditions
    /// - `x` est un entier non-négatif valide
    ///
    /// # Postconditions
    /// - `Ok(result)` == x² mathématiquement exact
    /// - Optimisé par rapport à multiply(x, x) (symétrie exploitée)
    fn square(&self, x: &BigUint, opts: &Options) -> Result<BigUint, FibError>;

    /// Nom descriptif de la stratégie.
    fn name(&self) -> &str;
}
```

### 7.2.3 Trait `DoublingStepExecutor`

```rust
/// Extension de Multiplier avec une exécution de pas de doublement optimisée.
///
/// Les consommateurs qui ont besoin du pas de doublement complet (combinant
/// plusieurs multiplications avec des optimisations algorithmiques comme
/// la réutilisation de transformées FFT) doivent dépendre de ce trait.
pub trait DoublingStepExecutor: Multiplier {
    /// Exécute un pas de doublement complet :
    /// F(2k) = F(k) × (2·F(k+1) - F(k))
    /// F(2k+1) = F(k+1)² + F(k)²
    ///
    /// # Préconditions
    /// - `state.fk` == F(k) pour un certain k ≥ 0
    /// - `state.fk1` == F(k+1) = F(k) + F(k-1)
    /// - `state.t1`, `state.t2`, `state.t3` sont des tampons valides
    /// - `ctx` n'est pas annulé
    ///
    /// # Postconditions
    /// - `state.fk` est mis à jour (via rotation de pointeurs) pour contenir F(2k)
    /// - `state.fk1` est mis à jour pour contenir F(2k+1)
    /// - Les tampons temporaires contiennent des valeurs intermédiaires valides
    ///
    /// # Invariants
    /// - Si `in_parallel` est true, les 3 multiplications sont exécutées en parallèle
    ///   sur des destinations disjointes
    /// - Aucune allocation dynamique dans le chemin critique (réutilisation des tampons)
    fn execute_step(
        &self,
        ctx: &CancellationToken,
        state: &mut CalculationState,
        opts: &Options,
        in_parallel: bool,
    ) -> Result<(), FibError>;
}
```

### 7.2.4 Trait `ProgressObserver`

```rust
/// Observateur d'événements de progression.
///
/// Les implémentations reçoivent des notifications quand la progression du calcul
/// change, permettant un traitement découplé pour l'UI, le logging, les métriques, etc.
pub trait ProgressObserver: Send + Sync {
    /// Appelé quand la progression change.
    ///
    /// # Préconditions
    /// - `calc_index` correspond à un calculateur actif
    /// - `progress` ∈ [0.0, 1.0]
    ///
    /// # Postconditions
    /// - L'observateur a traité la mise à jour (ou l'a ignorée si throttled)
    /// - Aucun blocage : l'appel retourne rapidement (< 1ms)
    ///
    /// # Invariants
    /// - Thread-safe : peut être appelé depuis n'importe quel thread
    /// - Ne doit jamais paniquer
    /// - Les appels successifs ont `progress` monotonement croissant (ou égal)
    fn update(&self, calc_index: usize, progress: f64);
}
```

### 7.2.5 Trait `SequenceGenerator`

```rust
/// Générateur de séquence Fibonacci itératif/streaming.
///
/// Contrairement à Calculator qui calcule un seul F(n), SequenceGenerator
/// produit des termes consécutifs pour les cas d'utilisation en streaming.
pub trait SequenceGenerator {
    /// Avance le générateur et retourne le prochain nombre de Fibonacci.
    ///
    /// # Préconditions
    /// - `ctx` n'est pas annulé
    ///
    /// # Postconditions
    /// - Le premier appel retourne F(0) = 0
    /// - Le k-ième appel retourne F(k-1)
    /// - `self.index()` est incrémenté de 1
    /// - `self.current()` retourne la valeur retournée
    fn next(&mut self, ctx: &CancellationToken) -> Result<BigUint, FibError>;

    /// Retourne le nombre de Fibonacci courant sans avancer.
    ///
    /// # Postconditions
    /// - Retourne None si `next()` n'a jamais été appelé
    /// - Sinon retourne la même valeur que le dernier `next()`
    fn current(&self) -> Option<&BigUint>;

    /// Retourne l'indice du nombre de Fibonacci courant.
    ///
    /// # Postconditions
    /// - Retourne 0 si `next()` n'a jamais été appelé
    /// - Sinon retourne l'indice du dernier terme produit
    fn index(&self) -> u64;

    /// Réinitialise le générateur à F(0).
    ///
    /// # Postconditions
    /// - `self.index()` == 0
    /// - `self.current()` == None
    /// - Le prochain `next()` retournera F(0)
    fn reset(&mut self);

    /// Avance directement au n-ième nombre de Fibonacci.
    ///
    /// # Préconditions
    /// - `ctx` n'est pas annulé
    ///
    /// # Postconditions
    /// - `self.index()` == n
    /// - Le résultat == F(n)
    /// - Plus efficace que d'appeler next() n fois pour de grands sauts
    fn skip(&mut self, ctx: &CancellationToken, n: u64) -> Result<BigUint, FibError>;
}
```

### 7.2.6 Trait `CalculatorFactory`

```rust
/// Fabrique de calculatrices avec registre et cache.
pub trait CalculatorFactory: Send + Sync {
    /// Crée une nouvelle instance de calculatrice par nom.
    ///
    /// # Préconditions
    /// - `name` est une chaîne non vide
    ///
    /// # Postconditions
    /// - Si `Ok(calc)` : `calc.name()` identifie l'algorithme
    /// - Si `Err(UnknownCalculator)` : `name` n'est pas enregistré
    ///
    /// # Invariants
    /// - Chaque appel crée une nouvelle instance (pas de cache)
    fn create(&self, name: &str) -> Result<Box<dyn Calculator>, FibError>;

    /// Retourne une instance existante (ou la crée et la cache).
    ///
    /// # Postconditions
    /// - Les appels successifs avec le même `name` retournent la même instance logique
    fn get(&self, name: &str) -> Result<Arc<dyn Calculator>, FibError>;

    /// Retourne la liste triée des noms de calculatrices enregistrées.
    ///
    /// # Postconditions
    /// - La liste est triée alphabétiquement
    /// - La liste contient au moins les entrées par défaut ["fast", "fft", "matrix"]
    fn list(&self) -> Vec<String>;

    /// Enregistre un nouveau type de calculatrice.
    ///
    /// # Postconditions
    /// - Si un calculateur du même nom existait, il est remplacé
    /// - Le cache pour ce nom est invalidé
    fn register(&self, name: &str, creator: Box<dyn Fn() -> Box<dyn Calculator> + Send + Sync>);

    /// Retourne toutes les calculatrices (initialisation lazy).
    fn get_all(&self) -> HashMap<String, Arc<dyn Calculator>>;
}
```

### 7.2.7 Trait `ProgressReporter`

```rust
/// Interface pour l'affichage de la progression du calcul.
///
/// Découple la couche d'orchestration de la couche de présentation.
pub trait ProgressReporter: Send + Sync {
    /// Démarre l'affichage de la progression à partir du canal.
    ///
    /// # Préconditions
    /// - `progress_rx` est un récepteur valide
    /// - `num_calculators` > 0
    ///
    /// # Postconditions
    /// - Bloque jusqu'à la fermeture du canal
    /// - Tous les messages reçus sont affichés (ou drainés pour NullProgressReporter)
    ///
    /// # Invariants
    /// - N'écrit que dans `out`, aucune mutation d'état externe
    fn display_progress(
        &self,
        progress_rx: mpsc::Receiver<ProgressUpdate>,
        num_calculators: usize,
        out: &mut dyn Write,
    );
}
```

### 7.2.8 Trait `ResultPresenter`

```rust
/// Interface pour la présentation des résultats de calcul.
pub trait ResultPresenter: Send + Sync {
    /// Affiche le tableau comparatif des résultats.
    ///
    /// # Préconditions
    /// - `results` contient au moins un élément
    /// - `results` est trié par durée (succès d'abord)
    ///
    /// # Postconditions
    /// - Un tableau formaté est écrit dans `out`
    fn present_comparison_table(&self, results: &[CalculationResult], out: &mut dyn Write);

    /// Affiche le résultat final du calcul.
    fn present_result(
        &self,
        result: &CalculationResult,
        n: u64,
        verbose: bool,
        details: bool,
        show_value: bool,
        out: &mut dyn Write,
    );

    /// Formate une durée pour l'affichage.
    ///
    /// # Postconditions
    /// - La chaîne retournée est lisible par l'humain (ex: "1.23s", "45.6ms")
    fn format_duration(&self, d: Duration) -> String;

    /// Gère les erreurs de calcul et retourne un code de sortie approprié.
    ///
    /// # Postconditions
    /// - Le code retourné ∈ {0, 1, 2, 3, 4, 130}
    fn handle_error(&self, err: &FibError, duration: Duration, out: &mut dyn Write) -> i32;
}
```

### 7.2.9 Trait `TempAllocator`

```rust
/// Abstraction pour l'allocation de tampons Fermat temporaires.
///
/// Permet à l'algorithme FFT de fonctionner avec différentes stratégies
/// d'allocation (pool, bump allocator) sans duplication de code.
pub trait TempAllocator: Send + Sync {
    /// Alloue un tampon Fermat temporaire de taille n+1.
    ///
    /// # Préconditions
    /// - `n` > 0
    ///
    /// # Postconditions
    /// - Le tampon retourné a exactement `n + 1` éléments
    /// - Tous les éléments sont initialisés à zéro
    /// - Le `Guard` doit être conservé jusqu'à la fin d'utilisation
    ///
    /// # Invariants
    /// - Thread-safe
    /// - Pour BumpAllocator : la libération est no-op (libération en bloc)
    /// - Pour PoolAllocator : la libération retourne au pool
    fn alloc_fermat_temp(&self, n: usize) -> (Fermat, AllocGuard);

    /// Alloue K nombres de Fermat, chacun de taille n+1.
    ///
    /// # Préconditions
    /// - `k` > 0, `n` > 0
    ///
    /// # Postconditions
    /// - Retourne exactement `k` tranches Fermat
    /// - Chaque tranche a `n + 1` éléments zéro
    fn alloc_fermat_slice(&self, k: usize, n: usize) -> (Vec<Fermat>, Vec<Word>, AllocGuard);
}
```

---

## T7.3 — Diagrammes de Flux de Données (Rust)

### 7.3.1 DFD 1 — Flux CLI (Argument Parsing → Calcul → Sortie)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        CRATE: fibcalc (bin)                             │
│                                                                         │
│  ┌──────────┐    ┌──────────────┐    ┌──────────────┐                  │
│  │  main()  │───▶│ AppConfig    │───▶│ Application  │                  │
│  │          │    │ (clap parse) │    │ ::new()      │                  │
│  └──────────┘    └──────────────┘    └──────┬───────┘                  │
│       │               │                      │                          │
│       │         env::FIBCALC_*          ┌────▼────┐                    │
│       │               │                │ dispatch │                    │
│       │               ▼                │  match   │                    │
│       │         ┌───────────┐          └─┬──┬──┬──┘                    │
│       │         │ Calibration│            │  │  │                      │
│       │         │ ::load()   │            │  │  │                      │
│       │         └───────────┘            │  │  │                      │
│       │                                   │  │  └─▶ run_completion()   │
│       │                                   │  └───▶ run_tui()           │
│       │                                   └──────▶ run_calculate()     │
└───────┼──────────────────────────────────────────────────────────────────┘
        │
        ▼
┌──────────────────────────────────────────────────────────────────────────┐
│                  CRATE: fibcalc-orchestration                           │
│                                                                         │
│  ┌────────────────────┐     ┌────────────────────────┐                 │
│  │ get_calculators    │────▶│ ExecuteCalculations()  │                 │
│  │   _to_run()        │     │                        │                 │
│  │                    │     │  ┌──── rayon scope ───┐│                 │
│  │ factory.list()     │     │  │ calc[0].calculate()││                 │
│  │ factory.get(name)  │     │  │ calc[1].calculate()││                 │
│  └────────────────────┘     │  │ calc[2].calculate()││                 │
│                              │  └────────────────────┘│                 │
│                              │         │               │                │
│                              │    mpsc::channel        │                │
│                              │    ProgressUpdate       │                │
│                              │         │               │                │
│                              │    ┌────▼────┐          │                │
│                              │    │Reporter │          │                │
│                              │    │display()│          │                │
│                              │    └─────────┘          │                │
│                              └────────────────────────┘                 │
│                                       │                                 │
│  ┌────────────────────────────────────▼──────────────────────────────┐  │
│  │ AnalyzeComparisonResults()                                        │  │
│  │  • sort par durée                                                 │  │
│  │  • vérifier cohérence (BigUint::eq)                              │  │
│  │  • présenter via ResultPresenter                                  │  │
│  └───────────────────────────────────────────────────────────────────┘  │
└──────────────────────────────────────────────────────────────────────────┘
        │
        ▼ Ownership: BigUint result is moved, not cloned
┌──────────────────────────────────────────────────────────────────────────┐
│                       CRATE: fibcalc-cli                                │
│                                                                         │
│  ┌─────────────────┐   ┌──────────────────┐  ┌────────────────────┐   │
│  │ CLIResultPresent│   │ DisplayQuietResult│  │ WriteResultToFile │   │
│  │   ::present()   │   │   (stdout)        │  │   (fs::write)     │   │
│  └─────────────────┘   └──────────────────┘  └────────────────────┘   │
└──────────────────────────────────────────────────────────────────────────┘
```

**Cycle de vie du ownership** :

1. `main()` : `AppConfig` owned par `Application`
2. `Application::run()` : `Options` emprunté (`&Options`) par les calculatrices
3. `calculate()` : `BigUint` créé, owned par `CalculationResult`
4. `AnalyzeComparisonResults()` : `&[CalculationResult]` emprunté
5. `PresentResult()` : `&BigUint` emprunté pour affichage
6. Retour de `main()` : tout est droppé automatiquement

### 7.3.2 DFD 2 — Flux TUI (Boucle événementielle → État → Rendu)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                        CRATE: fibcalc-tui                               │
│                                                                         │
│       ┌──────────┐          ┌────────────┐          ┌──────────┐       │
│       │ Terminal  │─────────▶│  Event     │─────────▶│  Model   │       │
│       │ (crossterm│  KeyEvent│  Dispatch  │  Msg     │  (state) │       │
│       │  events)  │─────────▶│            │─────────▶│          │       │
│       └──────────┘          └────────────┘          └────┬─────┘       │
│                                    ▲                      │             │
│                                    │                      │ update()    │
│                                    │                      ▼             │
│       ┌──────────┐          ┌──────┴──────┐         ┌──────────┐       │
│       │ Render   │◀─────────│  Command    │◀────────│ Updated  │       │
│       │ (ratatui │  View fn │  Pipeline   │  Cmd    │  Model   │       │
│       │  frame)  │◀─────────│             │◀────────│          │       │
│       └──────────┘          └─────────────┘         └──────────┘       │
│                                                                         │
│  ┌─────────────────────────────────────────────────────────────────┐    │
│  │                    Composants du Model                          │    │
│  │                                                                 │    │
│  │  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐  ┌────────┐  │    │
│  │  │ Header │  │  Logs  │  │Metrics │  │ Chart  │  │ Footer │  │    │
│  │  │ Panel  │  │ Panel  │  │ Panel  │  │ Panel  │  │ Panel  │  │    │
│  │  └────────┘  └────────┘  └────────┘  └────────┘  └────────┘  │    │
│  └─────────────────────────────────────────────────────────────────┘    │
│                                                                         │
│  Messages entrants (depuis fibcalc-orchestration via mpsc) :           │
│  ┌─────────────┐  ┌──────────────┐  ┌──────────────┐                  │
│  │ ProgressMsg │  │ ResultMsg    │  │ SystemMetrics│                  │
│  │ (f64, usize)│  │ (BigUint,Dur)│  │ (CPU,Mem)   │                  │
│  └─────────────┘  └──────────────┘  └──────────────┘                  │
└──────────────────────────────────────────────────────────────────────────┘
```

**Ownership TUI** :

- `Model` possède tous les panneaux par valeur
- Les messages sont `Send` pour traverser les frontières de threads
- `BigUint` dans `ResultMsg` est moved (pas cloné) depuis l'orchestration
- Le `Frame` de ratatui emprunte le `Model` en lecture seule pour le rendu

### 7.3.3 DFD 3 — Orchestration (Sélection → Exécution Parallèle → Analyse)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                  CRATE: fibcalc-orchestration                           │
│                                                                         │
│  AppConfig                                                              │
│     │                                                                   │
│     ▼                                                                   │
│  ┌──────────────────────┐                                              │
│  │ get_calculators_to_run│                                              │
│  │                       │                                              │
│  │ if algo == "all":     │                                              │
│  │   factory.list()      │── &CalculatorFactory ──▶ ["fast","fft",     │
│  │   factory.get(each)   │                          "matrix"]          │
│  │ else:                 │                                              │
│  │   factory.get(algo)   │                                              │
│  └──────────┬────────────┘                                              │
│             │ Vec<Arc<dyn Calculator>>                                  │
│             ▼                                                           │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ execute_calculations()                                       │      │
│  │                                                               │      │
│  │  let (tx, rx) = mpsc::channel(N * 50);                       │      │
│  │                                                               │      │
│  │  rayon::scope(|s| {                                          │      │
│  │    for (i, calc) in calculators.iter().enumerate() {         │      │
│  │      s.spawn(move |_| {                                      │      │
│  │        let start = Instant::now();                           │      │
│  │        let result = calc.calculate(&ctx, tx.clone(), i, n); │      │
│  │        results[i] = CalculationResult { ... };               │      │
│  │      });                                                     │      │
│  │    }                                                         │      │
│  │  });                                                          │      │
│  │                                                               │      │
│  │  drop(tx); // ferme le canal                                 │      │
│  │  reporter_handle.join();                                      │      │
│  └──────────────────────┬───────────────────────────────────────┘      │
│                          │ Vec<CalculationResult>                       │
│                          ▼                                              │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ analyze_comparison_results()                                  │      │
│  │                                                               │      │
│  │  results.sort_by(|a,b| {                                     │      │
│  │    match (a.err, b.err) {                                    │      │
│  │      (None, Some(_)) => Less,                                │      │
│  │      (Some(_), None) => Greater,                             │      │
│  │      _ => a.duration.cmp(&b.duration),                       │      │
│  │    }                                                          │      │
│  │  });                                                          │      │
│  │                                                               │      │
│  │  // Vérification de cohérence                                │      │
│  │  if results.iter()                                            │      │
│  │       .filter(|r| r.err.is_none())                           │      │
│  │       .any(|r| r.result != first.result) {                   │      │
│  │    return ExitCode::Mismatch(3);                             │      │
│  │  }                                                            │      │
│  └──────────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────────────┘
```

### 7.3.4 DFD 4 — Algorithme (Sélection Stratégie → Framework → Multiplication)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                       CRATE: fibcalc-core                               │
│                                                                         │
│  FibCalculator::calculate()                                             │
│     │                                                                   │
│     ├── n ≤ 93? ──▶ calculate_small(n) ──▶ return BigUint              │
│     │                                                                   │
│     ▼                                                                   │
│  ┌──────────────────────────────────────────────────────────────┐      │
│  │ GCController::begin()  [Rust: no-op ou mémoire tracking]    │      │
│  └────────────────────────────┬─────────────────────────────────┘      │
│                                │                                        │
│  ┌─────────────────────────────▼────────────────────────────────┐      │
│  │ CalculateCore dispatch (selon implémentation)                │      │
│  │                                                               │      │
│  │  OptimizedFastDoubling:                                      │      │
│  │    state = StatePool::acquire()                              │      │
│  │    arena = CalculationArena::new(n)                          │      │
│  │    strategy = AdaptiveStrategy                               │      │
│  │    framework = DoublingFramework::new(strategy)              │      │
│  │    framework.execute_doubling_loop(ctx, n, opts, state)      │      │
│  │    StatePool::release(state)                                 │      │
│  │                                                               │      │
│  │  FFTBasedCalculator:                                         │      │
│  │    strategy = FFTOnlyStrategy                                │      │
│  │    framework = DoublingFramework::new(strategy)              │      │
│  │    framework.execute_doubling_loop(ctx, n, opts, state)      │      │
│  │                                                               │      │
│  │  MatrixExponentiation:                                       │      │
│  │    state = MatrixStatePool::acquire()                        │      │
│  │    framework = MatrixFramework::new()                        │      │
│  │    framework.execute_matrix_loop(ctx, n, opts, state)        │      │
│  └────────────────────────────┬─────────────────────────────────┘      │
│                                │                                        │
│  ┌─────────────────────────────▼────────────────────────────────┐      │
│  │ DoublingFramework::execute_doubling_loop()                   │      │
│  │                                                               │      │
│  │  for i in (0..num_bits).rev() {                              │      │
│  │    // Décision parallélisme                                  │      │
│  │    let parallel = should_parallelize(opts, fk_bits, fk1_bits)│      │
│  │                                                               │      │
│  │    // Dispatch vers stratégie                                │      │
│  │    strategy.execute_step(ctx, state, opts, parallel)?;       │      │
│  │      │                                                       │      │
│  │      ├── AdaptiveStrategy:                                   │      │
│  │      │     fk1_bits > fft_threshold?                         │      │
│  │      │       oui → execute_doubling_step_fft()               │      │
│  │      │       non → execute_doubling_step_mul()               │      │
│  │      │                                                       │      │
│  │      ├── FFTOnlyStrategy:                                    │      │
│  │      │     → execute_doubling_step_fft() toujours            │      │
│  │      │                                                       │      │
│  │      └── KaratsubaStrategy:                                  │      │
│  │            → execute_doubling_step_mul() toujours            │      │
│  │                                                               │      │
│  │    // Post-multiplication                                    │      │
│  │    state.t3 <<= 1; state.t3 -= &state.t2;  // F(2k)       │      │
│  │    state.t1 += &state.t2;                    // F(2k+1)     │      │
│  │    swap pointers: fk↔t3, fk1↔t1, t1↔t2                    │      │
│  │                                                               │      │
│  │    // Addition step si bit == 1                              │      │
│  │    if (n >> i) & 1 == 1 { ... }                              │      │
│  │                                                               │      │
│  │    // Seuils dynamiques (optionnel)                          │      │
│  │    dtm.record_iteration(bits, dur, used_fft, used_parallel);│      │
│  │    report_step_progress(reporter, ...);                      │      │
│  │  }                                                            │      │
│  │                                                               │      │
│  │  // Zero-copy: "voler" fk du state                           │      │
│  │  let result = std::mem::replace(&mut state.fk, BigUint::ZERO)│      │
│  │  return Ok(result)                                            │      │
│  └──────────────────────────────────────────────────────────────┘      │
└──────────────────────────────────────────────────────────────────────────┘
```

### 7.3.5 DFD 5 — Pipeline FFT (Paramètres → Transformée → Multiplication → Inverse)

```
┌──────────────────────────────────────────────────────────────────────────┐
│                     CRATE: fibcalc-bigfft                               │
│                                                                         │
│  Entrée: &BigUint x, &BigUint y   (ou x pour squaring)                │
│                                                                         │
│  ┌──────────────────────────────────────────────────┐                  │
│  │ 1. Sélection des paramètres FFT                  │                  │
│  │    get_fft_params(target_words)                   │                  │
│  │    → (k: usize, m: usize)                        │                  │
│  │    k = log2(nombre de coefficients)               │                  │
│  │    m = taille en mots de chaque coefficient       │                  │
│  │    Choix basé sur: target_words = 2*max_words + 2 │                  │
│  └────────────────────┬─────────────────────────────┘                  │
│                        │                                                │
│  ┌─────────────────────▼────────────────────────────┐                  │
│  │ 2. Décomposition en polynômes Fermat              │                  │
│  │    poly_from_int(x, k, m) → Poly<Fermat>         │                  │
│  │    poly_from_int(y, k, m) → Poly<Fermat>         │                  │
│  │                                                    │                  │
│  │    Allocation via TempAllocator:                   │                  │
│  │    ├── BumpAllocator: O(1), zéro fragmentation   │                  │
│  │    └── PoolAllocator: sync::Pool<Vec<Word>>       │                  │
│  └────────────────────┬─────────────────────────────┘                  │
│                        │                                                │
│  ┌─────────────────────▼────────────────────────────┐                  │
│  │ 3. Transformée NTT (Number Theoretic Transform)   │                  │
│  │    poly_x.transform(n) → PolValues                │                  │
│  │    poly_y.transform(n) → PolValues                │                  │
│  │                                                    │                  │
│  │    Récursion FFT avec parallélisme:               │                  │
│  │    if depth < log2(num_cpus) {                    │                  │
│  │      rayon::join(fft_even, fft_odd)               │                  │
│  │    } else {                                       │                  │
│  │      fft_even(); fft_odd(); // séquentiel         │                  │
│  │    }                                              │                  │
│  │                                                    │                  │
│  │    Cache LRU (optionnel):                          │                  │
│  │    if bits > min_cache_bits {                      │                  │
│  │      cache.get_or_insert(key, || transform())     │                  │
│  │    }                                              │                  │
│  └────────────────────┬─────────────────────────────┘                  │
│                        │                                                │
│  ┌─────────────────────▼────────────────────────────┐                  │
│  │ 4. Multiplication point à point                   │                  │
│  │    values_x.mul(&values_y) → PolValues            │                  │
│  │    ou values_x.sqr() → PolValues (squaring)       │                  │
│  │                                                    │                  │
│  │    Pour chaque i in 0..K:                         │                  │
│  │      result[i] = fermat_mul(x[i], y[i])          │                  │
│  │    Arithmétique modulo 2^(64*m) + 1               │                  │
│  └────────────────────┬─────────────────────────────┘                  │
│                        │                                                │
│  ┌─────────────────────▼────────────────────────────┐                  │
│  │ 5. Transformée inverse                            │                  │
│  │    result_values.inv_transform() → Poly<Fermat>   │                  │
│  │    poly.set_m(m)                                  │                  │
│  │    poly.int_to_biguint(dest) → BigUint            │                  │
│  │                                                    │                  │
│  │    Reconstruction: propager les retenues entre     │                  │
│  │    coefficients, assembler le résultat final       │                  │
│  └────────────────────┬─────────────────────────────┘                  │
│                        │                                                │
│  Sortie: BigUint = x × y (ownership transféré)                         │
└──────────────────────────────────────────────────────────────────────────┘
```

---

## T7.4 — Catalogue de Cas Limites

### Table de 55 cas limites

| #  | Composant     | Cas Limite                                     | Traitement Attendu                                                              | Priorité |
| -- | ------------- | ---------------------------------------------- | ------------------------------------------------------------------------------- | --------- |
| 1  | Calculator    | n = 0                                          | Retourne `BigUint::from(0u32)`, pas de progression envoyée au-delà de 1.0   | P0        |
| 2  | Calculator    | n = 1                                          | Retourne `BigUint::from(1u32)`, chemin rapide itératif                       | P0        |
| 3  | Calculator    | n = 2                                          | Retourne `BigUint::from(1u32)`, chemin rapide itératif                       | P0        |
| 4  | Calculator    | n = 93 (MaxFibU64)                             | Dernier Fibonacci tenant dans u64 : 12200160415121876738                        | P0        |
| 5  | Calculator    | n = 94                                         | Premier Fibonacci nécessitant BigUint, bascule vers algorithme complet         | P0        |
| 6  | Calculator    | n = u64::MAX                                   | Estimation mémoire > RAM disponible → erreur mémoire ou validation de budget | P1        |
| 7  | Calculator    | Annulation contexte pendant calcul             | `Err(FibError::Cancelled)` retourné proprement, pas de fuite mémoire        | P0        |
| 8  | Calculator    | Timeout expiré                                | `Err(FibError::Timeout)`, code sortie 2                                       | P0        |
| 9  | Calculator    | Progression canal plein                        | Envoi non-bloquant, mise à jour droppée silencieusement                       | P1        |
| 10 | Calculator    | Progression canal None                         | Callback no-op, aucune panique                                                  | P0        |
| 11 | FastDoubling  | n = 2 (après fast-path)                       | Si n > 93 forcé, le framework gère correctement n = 2                         | P1        |
| 12 | FastDoubling  | Bit de poids fort = 0                          | Itération skip correcte, pas de division par zéro                             | P0        |
| 13 | FastDoubling  | n est puissance de 2                           | Uniquement des pas de doublement, aucun pas d'addition                          | P1        |
| 14 | FastDoubling  | n = 2^64 - 1                                   | 64 bits d'itération, vérifier que le compteur de bits est correct             | P1        |
| 15 | FastDoubling  | Tous les bits à 1                             | Maximum de pas d'addition, stress test mémoire                                 | P1        |
| 16 | Matrix        | n = 1 (exponent = 0)                           | Matrice identité retournée, F(1) = 1                                          | P0        |
| 17 | Matrix        | n = 2 (exponent = 1)                           | Une seule multiplication matricielle                                            | P0        |
| 18 | Matrix        | Strassen threshold = 0                         | Strassen désactivé, multiplication standard 8-mul                             | P1        |
| 19 | FFTBased      | n très petit (100)                            | FFT sur petits opérandes — surcoût acceptable, résultat correct             | P1        |
| 20 | FFTBased      | Erreur FFT interne                             | Propagation correcte vers l'appelant                                            | P0        |
| 21 | Strategy      | fft_threshold = 0                              | FFT désactivé, math/big uniquement                                            | P1        |
| 22 | Strategy      | parallel_threshold = 0                         | Parallélisme désactivé, exécution séquentielle                             | P1        |
| 23 | Strategy      | fft_threshold = 1                              | FFT activé pour tous les opérandes > 1 bit                                    | P2        |
| 24 | Observer      | Enregistrement observateur nil/None            | No-op, pas de panique                                                           | P0        |
| 25 | Observer      | Désenregistrement observateur non enregistré | No-op silencieux                                                                | P1        |
| 26 | Observer      | Freeze() sans observateurs                     | Retourne callback no-op                                                         | P0        |
| 27 | Observer      | Freeze() avec 100 observateurs                 | Snapshot correct, pas de deadlock                                               | P2        |
| 28 | Observer      | progress > 1.0                                 | Clamped à 1.0 par ChannelObserver                                              | P1        |
| 29 | Observer      | progress négatif                              | Accepté (pas de clamp bas dans le code Go actuel) — à valider                | P2        |
| 30 | Factory       | Nom de calculatrice inconnu                    | `Err(FibError::UnknownCalculator("xyz"))`                                     | P0        |
| 31 | Factory       | Double enregistrement même nom                | Le nouveau remplace l'ancien, cache invalidé                                   | P1        |
| 32 | Factory       | Get concurrent depuis 10 threads               | Thread-safe via `RwLock`, pas de data race                                    | P0        |
| 33 | Factory       | Liste vide (aucun enregistrement)              | Retourne `Vec::new()`, pas de panique                                         | P1        |
| 34 | Generator     | next() sans reset préalable                   | Retourne F(0) = 0 au premier appel                                              | P0        |
| 35 | Generator     | skip(0)                                        | Retourne F(0), index = 0                                                        | P1        |
| 36 | Generator     | skip(u64::MAX)                                 | Même considérations mémoire que Calculator                                   | P2        |
| 37 | Generator     | reset() après 1000 next()                     | Retourne à l'état initial, F(0) au prochain next()                            | P1        |
| 38 | Config        | -n négatif (chaîne)                          | Erreur de parsing clap, message d'aide                                          | P0        |
| 39 | Config        | -n 0                                           | Valide, calcule F(0) = 0                                                        | P0        |
| 40 | Config        | --timeout 0s                                   | Erreur de validation : "timeout must be positive"                               | P0        |
| 41 | Config        | --timeout 1ns                                  | Timeout quasi-immédiat, probablement Err(Timeout)                              | P1        |
| 42 | Config        | --algo "inexistant"                            | Erreur ConfigError avec liste des algos valides                                 | P0        |
| 43 | Config        | --threshold -1                                 | Erreur : "threshold cannot be negative"                                         | P0        |
| 44 | Config        | Variables env invalides                        | Ignorées silencieusement ou erreur claire                                      | P1        |
| 45 | Orchestration | 0 calculatrices                                | Retourne Vec vide, pas de panique                                               | P1        |
| 46 | Orchestration | 1 calculatrice                                 | Chemin rapide sans errgroup/rayon                                               | P0        |
| 47 | Orchestration | Résultats incohérents                        | Code sortie 3 (ExitErrorMismatch)                                               | P0        |
| 48 | Orchestration | Toutes les calculatrices échouent             | Code sortie basé sur la première erreur                                       | P0        |
| 49 | Modular       | m = 0                                          | Erreur : "modulus must be positive"                                             | P0        |
| 50 | Modular       | m = 1                                          | Retourne 0 (tout mod 1 = 0)                                                     | P1        |
| 51 | Modular       | m négatif                                     | Erreur : "modulus must be positive"                                             | P0        |
| 52 | Memory        | Estimation pour n = 0                          | Retourne 0 pour tous les champs                                                 | P1        |
| 53 | Memory        | ParseMemoryLimit("")                           | Erreur : "empty memory limit"                                                   | P0        |
| 54 | Memory        | ParseMemoryLimit("abc")                        | Erreur de parsing                                                               | P0        |
| 55 | Memory        | Budget dépassé                               | Avertissement et code sortie 4 (ExitErrorConfig)                                | P0        |

---

## T7.5 — Carte de Propagation des Erreurs

### 7.5.1 Hiérarchie d'erreurs `FibError`

```rust
/// Erreur racine de l'application fibcalc.
///
/// Utilise `thiserror` pour la dérivation automatique de Display et Error.
#[derive(Debug, thiserror::Error)]
pub enum FibError {
    // ─── Erreurs de configuration (code sortie 4) ───
    #[error("Erreur de configuration : {message}")]
    Config { message: String },

    #[error("Algorithme inconnu : '{name}'. Algorithmes valides : {available}")]
    UnknownCalculator { name: String, available: String },

    #[error("Limite mémoire invalide : {0}")]
    InvalidMemoryLimit(String),

    #[error("Budget mémoire dépassé : estimé {estimated}, limite {limit}")]
    MemoryBudgetExceeded { estimated: String, limit: String },

    // ─── Erreurs de calcul (code sortie 1) ───
    #[error("Erreur de calcul : {0}")]
    Calculation(String),

    #[error("Erreur FFT : {0}")]
    FftError(String),

    #[error("Erreur de multiplication : {0}")]
    MultiplicationFailed(String),

    #[error("Erreur de multiplication matricielle : {0}")]
    MatrixMultiplicationFailed(String),

    #[error("Le modulus doit être positif")]
    InvalidModulus,

    // ─── Erreurs de temporisation (code sortie 2) ───
    #[error("Timeout : la limite d'exécution a été atteinte")]
    Timeout,

    // ─── Erreurs d'annulation (code sortie 130) ───
    #[error("Calcul annulé par l'utilisateur")]
    Cancelled,

    // ─── Erreurs de cohérence (code sortie 3) ───
    #[error("Incohérence détectée entre les résultats des algorithmes")]
    ResultMismatch,

    // ─── Erreurs I/O ───
    #[error("Erreur I/O : {0}")]
    Io(#[from] std::io::Error),

    // ─── Erreurs de sérialisation ───
    #[error("Erreur de sérialisation : {0}")]
    Serialization(#[from] serde_json::Error),

    // ─── Erreurs GMP/FFI (feature "gmp") ───
    #[cfg(feature = "gmp")]
    #[error("Erreur GMP : {0}")]
    GmpError(String),
}
```

### 7.5.2 Codes de sortie

| Code | Constante Rust          | Variantes `FibError`                                                                                  | Description                    |
| ---- | ----------------------- | ------------------------------------------------------------------------------------------------------- | ------------------------------ |
| 0    | `EXIT_SUCCESS`        | — (pas d'erreur)                                                                                       | Exécution réussie            |
| 1    | `EXIT_ERROR_GENERIC`  | `Calculation`, `FftError`, `MultiplicationFailed`, `MatrixMultiplicationFailed`, `Io`         | Erreur générique             |
| 2    | `EXIT_ERROR_TIMEOUT`  | `Timeout`                                                                                             | Dépassement du délai         |
| 3    | `EXIT_ERROR_MISMATCH` | `ResultMismatch`                                                                                      | Incohérence entre algorithmes |
| 4    | `EXIT_ERROR_CONFIG`   | `Config`, `UnknownCalculator`, `InvalidMemoryLimit`, `MemoryBudgetExceeded`, `InvalidModulus` | Erreur de configuration        |
| 130  | `EXIT_ERROR_CANCELED` | `Cancelled`                                                                                           | Annulation (SIGINT)            |

### 7.5.3 Carte de propagation

```
 Couche Algorithme (fibcalc-core)
 ═══════════════════════════════

 smartMultiply() ──FftError──▶ execute_doubling_step_mul() ──MultiplicationFailed──▶
                                                                                    │
 smartSquare()  ──FftError──▶ execute_doubling_step_mul() ──MultiplicationFailed──▶ │
                                                                                    │
 fermat_mul()   ──FftError──▶ fft_poly::transform()       ──FftError──────────────▶ │
                                                                                    │
 ctx.cancelled? ──Cancelled──▶ execute_doubling_loop()     ──Cancelled─────────────▶ │
                                                                                    ▼
                              DoublingFramework                                      │
                              ═══════════════                                        │
                                                                                    │
 execute_step() ────────────▶ ExecuteDoublingLoop()                                 │
   ├── AdaptiveStrategy       │  wrap: "doubling step failed at bit {i}/{n}: {e}"   │
   ├── FFTOnlyStrategy        │                                                     │
   └── KaratsubaStrategy      ▼                                                     │
                                                                                    │
                              FibCalculator::calculate()                             │
                              ══════════════════════════                             │
                              │  GCController::begin/end                             │
                              │  pool acquire/release                                │
                              ▼                                                     │
                                                                                    │
 Couche Orchestration (fibcalc-orchestration)                                       │
 ════════════════════════════════════════════                                        │
                                                                                    │
 ExecuteCalculations() ◀────────────────────────────────────────────────────────────┘
   │
   │  Chaque résultat stocké dans CalculationResult { err: Option<FibError> }
   │
   ▼
 AnalyzeComparisonResults()
   │
   ├── Aucun succès → HandleError(first_error)
   │     ├── Timeout    → code 2
   │     ├── Cancelled  → code 130
   │     └── Autre      → code 1
   │
   ├── Incohérence → ResultMismatch → code 3
   │
   └── Succès → code 0

 Couche Application (fibcalc bin)
 ════════════════════════════════

 Application::run()
   │
   ├── run_completion()  → Config error → code 4
   ├── run_calibration() → Io error → code 1
   ├── run_tui()         → propage le code d'orchestration
   └── run_calculate()
         │
         ├── ParseMemoryLimit() → InvalidMemoryLimit → code 4
         ├── MemoryBudgetExceeded → code 4
         └── execute + analyze → code {0, 1, 2, 3, 130}

 main()
   │
   ├── Application::new() échec
   │     ├── clap parse error → affiche aide, code 4
   │     └── flag::ErrHelp → code 0
   │
   └── Application::run() → std::process::exit(code)
```

### 7.5.4 Conversion `FibError` → Code de sortie

```rust
impl FibError {
    /// Convertit l'erreur en code de sortie approprié.
    pub fn exit_code(&self) -> i32 {
        match self {
            FibError::Config { .. }
            | FibError::UnknownCalculator { .. }
            | FibError::InvalidMemoryLimit(_)
            | FibError::MemoryBudgetExceeded { .. }
            | FibError::InvalidModulus => EXIT_ERROR_CONFIG,  // 4

            FibError::Timeout => EXIT_ERROR_TIMEOUT,          // 2
            FibError::Cancelled => EXIT_ERROR_CANCELED,       // 130
            FibError::ResultMismatch => EXIT_ERROR_MISMATCH,  // 3

            _ => EXIT_ERROR_GENERIC,                          // 1
        }
    }
}
```

---

## T7.6 — Spécification FFI (GMP / `rug` Feature)

### 7.6.1 Feature Cargo

```toml
# Cargo.toml de fibcalc-core
[features]
default = []
gmp = ["dep:rug"]

[dependencies]
rug = { version = "1.24", optional = true, features = ["integer"] }
```

**Correspondance build tags Go → Cargo features** :

| Go Build Tag                  | Cargo Feature                      | Effet                 |
| ----------------------------- | ---------------------------------- | --------------------- |
| `//go:build gmp`            | `#[cfg(feature = "gmp")]`        | Compile le module GMP |
| `//go:build !gmp` (défaut) | `#[cfg(not(feature = "gmp"))]`   | Module GMP absent     |
| `//go:build amd64`          | `#[cfg(target_arch = "x86_64")]` | Optimisations amd64   |

### 7.6.2 Inventaire des blocs `unsafe`

Le portage Rust vise à **minimiser** l'utilisation de `unsafe`. Voici l'inventaire exhaustif des blocs `unsafe` nécessaires :

| # | Fichier Rust                           | Bloc `unsafe`                   | Raison                                                         | Preuve de sécurité                                                                                                                                                                                             |
| - | -------------------------------------- | --------------------------------- | -------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 1 | `fibcalc-core/src/calculator_gmp.rs` | `rug::Integer::from_raw()`      | Conversion FFI depuis pointeur mpz_t brut                      | `rug` gère la lifetime ; le pointeur est valide car créé par `rug::Integer::new()`. Aucune aliasing possible car `rug::Integer` est `!Sync`.                                                          |
| 2 | `fibcalc-core/src/calculator_gmp.rs` | `rug::Integer::as_raw()`        | Accès au pointeur interne pour optimisation                   | En lecture seule, pas de mutation. Le `rug::Integer` reste vivant pendant tout l'accès.                                                                                                                       |
| 3 | `fibcalc-bigfft/src/arith_amd64.rs`  | `std::arch::x86_64::_mm256_*`   | Instructions SIMD AVX2 pour arithmétique vectorielle          | Les tampons sont alignés à 32 octets via `repr(align(32))`. Les longueurs sont vérifiées avant l'appel. Le runtime vérifie la disponibilité AVX2 via `is_x86_feature_detected!()`.                     |
| 4 | `fibcalc-bigfft/src/arith_amd64.rs`  | `std::arch::x86_64::_mm512_*`   | Instructions SIMD AVX-512 (optionnel)                          | Même garanties que AVX2. Feature gate supplémentaire via `is_x86_feature_detected!("avx512f")`.                                                                                                              |
| 5 | `fibcalc-bigfft/src/bump.rs`         | `slice::from_raw_parts_mut()`   | Allocation bump O(1) depuis buffer pré-alloué                | Le buffer est alloué via `Vec<Word>` avec capacité suffisante. L'offset est vérifié contre la capacité avant chaque allocation. Pas d'aliasing : chaque allocation retourne une tranche disjointe.        |
| 6 | `fibcalc-bigfft/src/fermat.rs`       | Arithmétique sur pointeurs bruts | Opérations bit-à-bit sur mots machine pour nombres de Fermat | Les tailles sont vérifiées. Les opérations sont des add/sub/shift sur des tranches `&mut [Word]` avec bounds checking désactivé pour performance (`get_unchecked`). Les indices sont validés en amont. |

### 7.6.3 Documentation `rug::Integer`

```rust
/// Module GMP : calculatrice Fibonacci utilisant la bibliothèque GMP via `rug`.
///
/// # Prérequis système
/// - Linux : `sudo apt-get install libgmp-dev`
/// - macOS : `brew install gmp`
/// - Windows : MinGW avec libgmp ou WSL
///
/// # Activation
/// ```bash
/// cargo build --features gmp
/// cargo test --features gmp
/// ```
///
/// # Architecture
/// L'utilisation directe de `rug::Integer` (plutôt qu'un trait BigInt abstrait)
/// est un choix architectural délibéré : l'indirection d'interface annulerait
/// les gains de performance de GMP.
///
/// # Sécurité mémoire
/// - `rug::Integer` gère sa propre mémoire via l'allocateur GMP
/// - Aucun `unsafe` direct : `rug` encapsule toutes les opérations FFI
/// - La conversion vers `num_bigint::BigUint` se fait via sérialisation
///   en octets (`rug::Integer::to_digits::<u8>(Order::Msf)`)
///
/// # Performance
/// - Excelle pour N > 100_000_000 où les routines assembleur de GMP
///   surpassent `num-bigint`
/// - Pour les petits N, le surcoût CGO/FFI peut rendre `num-bigint` plus rapide
#[cfg(feature = "gmp")]
pub mod gmp {
    use rug::Integer;
    use num_bigint::BigUint;

    /// Calculatrice GMP utilisant l'algorithme Fast Doubling.
    pub struct GmpCalculator;

    impl GmpCalculator {
        /// Convertit un rug::Integer en num_bigint::BigUint.
        ///
        /// # Sécurité
        /// Aucun `unsafe` : utilise la sérialisation en octets.
        fn to_biguint(g: &Integer) -> BigUint {
            let bytes = g.to_digits::<u8>(rug::integer::Order::Msf);
            BigUint::from_bytes_be(&bytes)
        }
    }
}
```

### 7.6.4 Matrice de compatibilité

| Plateforme     | `num-bigint` (défaut) | `rug` (feature gmp) | SIMD amd64         |
| -------------- | ------------------------ | --------------------- | ------------------ |
| Linux x86_64   | Oui                      | Oui (libgmp-dev)      | Oui (AVX2/AVX-512) |
| Linux aarch64  | Oui                      | Oui (libgmp-dev)      | Non (NEON futur)   |
| macOS x86_64   | Oui                      | Oui (brew gmp)        | Oui (AVX2)         |
| macOS aarch64  | Oui                      | Oui (brew gmp)        | Non                |
| Windows x86_64 | Oui                      | Partiel (MinGW)       | Oui (AVX2)         |
| WASM           | Oui                      | Non                   | Non                |

---

## T7.7 — Scénarios de Tests d'Intégration

### Table de 25 scénarios E2E

| ID     | Description               | Setup            | Exécution                                      | Vérification                              | Résultat Attendu             | Code |
| ------ | ------------------------- | ---------------- | ----------------------------------------------- | ------------------------------------------ | ----------------------------- | ---- |
| E2E-01 | Calcul basique F(10)      | Binaire compilé | `fibcalc -n 10 -c`                            | stdout contient "F(10) = 55"               | F(10) = 55 affiché           | 0    |
| E2E-02 | Affichage aide            | —               | `fibcalc --help`                              | stdout contient "usage" (insensible casse) | Aide affichée                | 0    |
| E2E-03 | Comparaison tous algos    | —               | `fibcalc -n 100 --algo all -c`                | stdout contient "F(100)"                   | Résultats cohérents         | 0    |
| E2E-04 | Mode silencieux           | —               | `fibcalc -n 10 --quiet -c`                    | stdout contient "55", pas de bannière     | Sortie minimale               | 0    |
| E2E-05 | Timeout très court       | —               | `fibcalc -n 10000000 --timeout 1ms`           | Code sortie non-zéro                      | Timeout ou erreur             | 2    |
| E2E-06 | F(0) valide               | —               | `fibcalc -n 0 -c`                             | stdout contient "F(0)"                     | F(0) = 0                      | 0    |
| E2E-07 | F(1000) grand nombre      | —               | `fibcalc -n 1000 -c`                          | stdout contient "F(1000)"                  | Résultat correct             | 0    |
| E2E-08 | Flag version              | —               | `fibcalc --version`                           | stdout contient "fibcalc"                  | Version affichée             | 0    |
| E2E-09 | Algo spécifique "fast"   | —               | `fibcalc -n 500 --algo fast -c`               | stdout contient "Fast Doubling"            | Algo fast utilisé            | 0    |
| E2E-10 | Algo spécifique "matrix" | —               | `fibcalc -n 500 --algo matrix -c`             | stdout contient "Matrix"                   | Algo matrix utilisé          | 0    |
| E2E-11 | Algo spécifique "fft"    | —               | `fibcalc -n 500 --algo fft -c`                | stdout contient "FFT"                      | Algo FFT utilisé             | 0    |
| E2E-12 | Algo invalide             | —               | `fibcalc --algo xyz`                          | stderr contient "unrecognized" ou erreur   | Message d'erreur              | 4    |
| E2E-13 | Sortie fichier            | TempDir          | `fibcalc -n 100 -c -o {tmp}/result.txt`       | Fichier créé, contient le résultat      | Fichier écrit                | 0    |
| E2E-14 | Mode verbose              | —               | `fibcalc -n 100 -v -c`                        | stdout contient la valeur complète        | Valeur complète affichée    | 0    |
| E2E-15 | Mode détails             | —               | `fibcalc -n 100 -d -c`                        | stdout contient métriques performance     | Détails affichés            | 0    |
| E2E-16 | Last digits               | —               | `fibcalc -n 1000000 --last-digits 10`         | stdout contient 10 chiffres                | Derniers 10 chiffres corrects | 0    |
| E2E-17 | Complétion bash          | —               | `fibcalc --completion bash`                   | stdout est un script bash valide           | Script complétion            | 0    |
| E2E-18 | Complétion zsh           | —               | `fibcalc --completion zsh`                    | stdout est un script zsh valide            | Script complétion            | 0    |
| E2E-19 | Variable env FIBCALC_N    | `FIBCALC_N=42` | `fibcalc -c`                                  | stdout contient "F(42)"                    | Env var respectée            | 0    |
| E2E-20 | NO_COLOR respecté        | `NO_COLOR=1`   | `fibcalc -n 10 -c`                            | Pas de codes ANSI dans stdout              | Pas de couleurs               | 0    |
| E2E-21 | Memory limit suffisant    | —               | `fibcalc -n 1000 --memory-limit 1G -c`        | Calcul réussi                             | Sous la limite                | 0    |
| E2E-22 | Memory limit insuffisant  | —               | `fibcalc -n 1000000000 --memory-limit 1K`     | Message "exceeds limit"                    | Budget dépassé              | 4    |
| E2E-23 | GC control aggressive     | —               | `fibcalc -n 10000 --gc-control aggressive -c` | Calcul réussi                             | Mode GC respecté             | 0    |
| E2E-24 | Calibration mode          | —               | `fibcalc --calibrate` (timeout 30s)           | stdout contient résultats calibration     | Calibration exécutée        | 0    |
| E2E-25 | Signal SIGINT             | —               | `fibcalc -n 10000000` + SIGINT après 100ms   | Code sortie 130                            | Annulation propre             | 130  |

### 7.7.1 Framework de test recommandé

```rust
// tests/e2e/cli_e2e_test.rs
use assert_cmd::Command;
use predicates::prelude::*;
use tempfile::TempDir;

#[test]
fn e2e_01_basic_calculation() {
    Command::cargo_bin("fibcalc")
        .unwrap()
        .args(&["-n", "10", "-c"])
        .env("NO_COLOR", "1")
        .assert()
        .success()
        .stdout(predicate::str::contains("F(10) = 55"));
}

#[test]
fn e2e_05_timeout() {
    Command::cargo_bin("fibcalc")
        .unwrap()
        .args(&["-n", "10000000", "--timeout", "1ms"])
        .env("NO_COLOR", "1")
        .assert()
        .failure()
        .code(2);
}

#[test]
fn e2e_13_output_file() {
    let tmp = TempDir::new().unwrap();
    let path = tmp.path().join("result.txt");
    Command::cargo_bin("fibcalc")
        .unwrap()
        .args(&["-n", "100", "-c", "-o", path.to_str().unwrap()])
        .env("NO_COLOR", "1")
        .assert()
        .success();
    assert!(path.exists());
    let content = std::fs::read_to_string(&path).unwrap();
    assert!(!content.is_empty());
}
```

### 7.7.2 Tests de propriété (proptest)

```rust
// tests/property_tests.rs
use proptest::prelude::*;

proptest! {
    /// F(n) calculé par fast doubling == F(n) calculé par matrix
    #[test]
    fn fast_doubling_equals_matrix(n in 0u64..10_000) {
        let fast = calculate_fast_doubling(n);
        let matrix = calculate_matrix(n);
        prop_assert_eq!(fast, matrix);
    }

    /// F(n+1) = F(n) + F(n-1) pour n ≥ 2
    #[test]
    fn fibonacci_identity(n in 2u64..5_000) {
        let fn_minus_1 = calculate(n - 1);
        let fn_val = calculate(n);
        let fn_plus_1 = calculate(n + 1);
        prop_assert_eq!(fn_plus_1, fn_val + fn_minus_1);
    }

    /// La progression est monotonement croissante
    #[test]
    fn progress_monotonic(n in 100u64..50_000) {
        let updates = collect_progress_updates(n);
        for window in updates.windows(2) {
            prop_assert!(window[1].value >= window[0].value);
        }
    }
}
```

### 7.7.3 Tests de golden file

```rust
// tests/golden_test.rs
use serde::Deserialize;

#[derive(Deserialize)]
struct GoldenEntry {
    n: u64,
    value: String,
}

#[test]
fn golden_file_validation() {
    let data = include_str!("../../testdata/fibonacci_golden.json");
    let entries: Vec<GoldenEntry> = serde_json::from_str(data).unwrap();

    for entry in entries {
        let result = calculate(entry.n);
        assert_eq!(
            result.to_string(),
            entry.value,
            "Mismatch for F({})",
            entry.n
        );
    }
}
```

### 7.7.4 Tests fuzz (cargo-fuzz)

```rust
// fuzz/fuzz_targets/fuzz_fast_doubling.rs
#![no_main]
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: u64| {
    if data > 1_000_000 {
        return; // Limiter la taille pour le fuzzing
    }
    let result = fibcalc_core::calculate_fast_doubling(data);
    // Vérifier l'identité F(n)² + F(n+1)² = F(2n+1)
    if data > 0 {
        let fn_sq = &result * &result;
        let fn1 = fibcalc_core::calculate_fast_doubling(data + 1);
        let fn1_sq = &fn1 * &fn1;
        let f2n1 = fibcalc_core::calculate_fast_doubling(2 * data + 1);
        assert_eq!(fn_sq + fn1_sq, f2n1);
    }
});
```

---

## T7.8 — Structure Documentaire du Projet Rust

### 7.8.1 Arborescence `docs/`

```
docs/
├── BUILD.md                              # Instructions de compilation
├── CALIBRATION.md                        # Guide de calibration
├── PERFORMANCE.md                        # Benchmarks et optimisations
├── TESTING.md                            # Guide de test
├── TUI_GUIDE.md                          # Guide du dashboard TUI
├── MIGRATION.md                          # [NOUVEAU] Guide de migration Go → Rust
├── UNSAFE_AUDIT.md                       # [NOUVEAU] Audit des blocs unsafe
│
├── algorithms/
│   ├── BIGFFT.md                         # Algorithme BigFFT
│   ├── COMPARISON.md                     # Comparaison des algorithmes
│   ├── FAST_DOUBLING.md                  # Algorithme Fast Doubling
│   ├── FFT.md                            # Multiplication FFT
│   ├── GMP.md                            # Intégration GMP/rug
│   ├── MATRIX.md                         # Exponentiation matricielle
│   └── PROGRESS_BAR_ALGORITHM.md         # Algorithme barre de progression
│
├── architecture/
│   ├── README.md                         # Vue d'ensemble architecture
│   ├── CRATE_DEPENDENCIES.md             # [NOUVEAU] Graphe dépendances crates
│   ├── OWNERSHIP_MODEL.md                # [NOUVEAU] Modèle de propriété Rust
│   ├── component-diagram.mermaid         # Diagramme de composants
│   ├── container-diagram.mermaid         # Diagramme de conteneurs
│   ├── dependency-graph.mermaid          # Graphe de dépendances
│   ├── system-context.mermaid            # Contexte système
│   │
│   ├── flows/
│   │   ├── cli-flow.mermaid              # Flux CLI
│   │   ├── config-flow.mermaid           # Flux configuration
│   │   ├── fastdoubling.mermaid          # Flux Fast Doubling
│   │   ├── fft-pipeline.mermaid          # Pipeline FFT
│   │   ├── matrix.mermaid                # Flux Matrix
│   │   └── tui-flow.mermaid              # Flux TUI
│   │
│   ├── patterns/
│   │   ├── interface-hierarchy.mermaid   # Hiérarchie des traits
│   │   └── TRAIT_CONTRACTS.md            # [NOUVEAU] Contrats de traits formels
│   │
│   └── validation/
│       └── validation-report.md          # Rapport de validation
│
└── api/                                  # [NOUVEAU] Documentation API
    ├── README.md                         # Index de l'API
    └── RUSTDOC_CONFIG.md                 # Configuration rustdoc
```

### 7.8.2 Contenu attendu par fichier

| Fichier                   | Contenu                                                          | Source de migration               |
| ------------------------- | ---------------------------------------------------------------- | --------------------------------- |
| `BUILD.md`              | Instructions `cargo build`, features, cross-compilation, PGO   | Adaptation depuis Go `BUILD.md` |
| `CALIBRATION.md`        | `--calibrate`, `--auto-calibrate`, profils JSON              | Migration directe                 |
| `PERFORMANCE.md`        | Benchmarks `criterion`, flamegraphs, comparaison Go/Rust       | Adaptation + nouveaux benchmarks  |
| `TESTING.md`            | `cargo test`, `cargo fuzz`, proptest, golden files, coverage | Adaptation majeure                |
| `TUI_GUIDE.md`          | Ratatui, raccourcis, personnalisation                            | Adaptation pour Ratatui           |
| `MIGRATION.md`          | Guide détaillé du portage, décisions architecturales, pièges | **Nouveau**                 |
| `UNSAFE_AUDIT.md`       | Liste complète des `unsafe`, preuves de sécurité            | **Nouveau**                 |
| `algorithms/*.md`       | Identiques au Go avec exemples Rust                              | Adaptation code samples           |
| `CRATE_DEPENDENCIES.md` | Graphe inter-crate, dépendances externes, versions              | **Nouveau**                 |
| `OWNERSHIP_MODEL.md`    | Patterns de propriété, borrowing, lifetimes clés              | **Nouveau**                 |
| `TRAIT_CONTRACTS.md`    | Spécification formelle (reprise de T7.2)                        | **Nouveau**                 |
| `api/RUSTDOC_CONFIG.md` | Configuration `#![doc]`, exemples, `doc-cfg`                 | **Nouveau**                 |

### 7.8.3 Outillage de génération documentaire

| Outil                     | Usage                              | Configuration                                                |
| ------------------------- | ---------------------------------- | ------------------------------------------------------------ |
| **rustdoc**         | Documentation API automatique      | `cargo doc --no-deps --all-features`                       |
| **mdBook**          | Site de documentation narrative    | `docs/book.toml` + `docs/src/SUMMARY.md`                 |
| **mermaid-cli**     | Rendu des diagrammes Mermaid       | `mmdc -i input.mermaid -o output.svg`                      |
| **cargo-tarpaulin** | Couverture de code                 | `cargo tarpaulin --all-features --out Html`                |
| **cargo-criterion** | Benchmarks avec rapports HTML      | `cargo criterion`                                          |
| **cargo-fuzz**      | Fuzzing continu                    | `cargo fuzz run fuzz_fast_doubling`                        |
| **clippy**          | Linting Rust                       | `cargo clippy --all-features --all-targets -- -D warnings` |
| **cargo-deny**      | Audit licences et vulnérabilités | `cargo deny check`                                         |

### 7.8.4 Configuration `book.toml` (mdBook)

```toml
[book]
title = "FibRust Documentation"
authors = ["FibGo Migration Team"]
language = "fr"
multilingual = false
src = "src"

[build]
build-dir = "book"

[preprocessor.mermaid]
command = "mdbook-mermaid"

[output.html]
default-theme = "rust"
preferred-dark-theme = "ayu"
git-repository-url = "https://github.com/agbru/fibrust"
edit-url-template = "https://github.com/agbru/fibrust/edit/main/docs/{path}"

[output.html.search]
enable = true
```

### 7.8.5 Configuration `rustdoc`

```rust
// fibcalc-core/src/lib.rs
#![doc = include_str!("../../README.md")]
#![doc(
    html_logo_url = "https://raw.githubusercontent.com/agbru/fibrust/main/docs/logo.svg",
    html_favicon_url = "https://raw.githubusercontent.com/agbru/fibrust/main/docs/favicon.ico"
)]
#![cfg_attr(docsrs, feature(doc_cfg))]
#![warn(missing_docs)]
#![warn(rustdoc::missing_doc_code_examples)]
```

### 7.8.6 CI/CD Documentation

```yaml
# .github/workflows/docs.yml
name: Documentation
on:
  push:
    branches: [main]
  pull_request:

jobs:
  rustdoc:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@nightly
      - run: cargo doc --all-features --no-deps
        env:
          RUSTDOCFLAGS: "--cfg docsrs -D warnings"
      - uses: actions/upload-pages-artifact@v3
        with:
          path: target/doc

  mdbook:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: peaceiris/actions-mdbook@v2
      - run: mdbook build docs/
      - uses: actions/upload-pages-artifact@v3
        with:
          path: docs/book

  deploy:
    needs: [rustdoc, mdbook]
    runs-on: ubuntu-latest
    if: github.ref == 'refs/heads/main'
    permissions:
      pages: write
      id-token: write
    environment:
      name: github-pages
    steps:
      - uses: actions/deploy-pages@v4
```

---

## Annexe A — Dépendances Cargo recommandées

| Crate Rust                           | Remplace (Go)                     | Usage                                 |
| ------------------------------------ | --------------------------------- | ------------------------------------- |
| `num-bigint` + `num-traits`      | `math/big`                      | Arithmétique grande précision       |
| `rug` (feature gmp)                | `github.com/ncw/gmp`            | Liaison GMP                           |
| `rayon`                            | `golang.org/x/sync/errgroup`    | Parallélisme work-stealing           |
| `tokio`                            | goroutines + channels             | Runtime async (optionnel, pour TUI)   |
| `clap` + `clap_complete`         | `flag`                          | Parsing CLI + complétion shell       |
| `ratatui` + `crossterm`          | `bubbletea` + `lipgloss`      | TUI framework                         |
| `indicatif`                        | `github.com/briandowns/spinner` | Spinners et barres de progression CLI |
| `tracing` + `tracing-subscriber` | `github.com/rs/zerolog`         | Logging structuré                    |
| `sysinfo`                          | `github.com/shirou/gopsutil/v4` | Métriques système                   |
| `thiserror` + `anyhow`           | `errors` + `fmt.Errorf`       | Gestion d'erreurs                     |
| `serde` + `serde_json`           | `encoding/json`                 | Sérialisation                        |
| `proptest`                         | `github.com/leanovate/gopter`   | Tests basés sur les propriétés     |
| `criterion`                        | `testing.B`                     | Benchmarks                            |
| `assert_cmd` + `predicates`      | `os/exec` (tests E2E)           | Tests d'intégration binaire          |
| `tempfile`                         | `t.TempDir()`                   | Répertoires temporaires pour tests   |

## Annexe B — Checklist de validation du portage

- [ ] Tous les 97 fichiers Go source ont un équivalent Rust identifié
- [ ] Les 9 traits Rust couvrent toutes les interfaces Go
- [ ] Les 5 DFD documentent les flux de données avec types Rust concrets
- [ ] Les 55 cas limites ont des tests correspondants
- [ ] La hiérarchie `FibError` couvre tous les chemins d'erreur Go
- [ ] Les 6 blocs `unsafe` sont documentés avec preuves de sécurité
- [ ] Les 25 scénarios E2E sont implémentés et passent
- [ ] La documentation couvre les 24 fichiers existants + 7 nouveaux
- [ ] Les golden files sont partagés entre Go et Rust
- [ ] `cargo clippy` passe sans warning
- [ ] `cargo test --all-features` passe à 100%
- [ ] La couverture de code ≥ 75%
- [ ] `cargo deny check` ne signale aucune vulnérabilité

---

# Résumé & Dépendances critiques

## Récapitulatif des phases

| Phase           | Tâches         | Focus                                                         | Lignes          |
| --------------- | --------------- | ------------------------------------------------------------- | --------------- |
| Phase 1         | 12 (T1.1-T1.12) | Fondations, exigences, évaluation dépendances, risques      | ~1500           |
| Phase 2         | 18 (T2.1-T2.18) | Algorithmes détaillés (Fast Doubling, Matrix, FFT, Modular) | ~1700           |
| Phase 3         | 10 (T3.1-T3.10) | Observer, progression, modèle géométrique                  | ~1200           |
| Phase 4         | 12 (T4.1-T4.12) | Mémoire (arena, pool, bump), concurrence, annulation         | ~1200           |
| Phase 5         | 8 (T5.1-T5.8)   | Seuils dynamiques, calibration, profils                       | ~1000           |
| Phase 6         | 10 (T6.1-T6.10) | TUI (layout, messages, sparklines, bridge)                    | ~1000           |
| Phase 7         | 8 (T7.1-T7.8)   | Intégration (migration map, DFD, edge cases, tests)          | ~1600           |
| **Total** | **78**    | **PRD complet pour le portage Go → Rust**              | **~9300** |

## Graphe de dépendances critiques

```
Phase 1 (fondations) ──→ Phase 2 (algorithmes) ──→ Phase 7.1 (migration map)
Phase 1 ──→ Phase 3 (observer) ──→ Phase 6.10 (TUI bridge)
Phase 1 ──→ Phase 4 (mémoire) ──→ Phase 7.2 (contrats de traits)
Phase 2 ──→ Phase 5 (seuils dynamiques)
Phase 6 (TUI) ──→ Phase 7.3 (DFD)
Toutes les phases ──→ Phase 7 (intégration finale)
```

## Chemin critique d'implémentation recommandé

1. **Sprint 1** : fibcalc-core (types de base, traits Calculator/Multiplier, fast path u64)
2. **Sprint 2** : fibcalc-bigfft (arithmétique Fermat, FFT core, pools, bump allocator)
3. **Sprint 3** : Algorithmes complets (Fast Doubling, Matrix, FFT-based) avec tests golden
4. **Sprint 4** : fibcalc-orchestration (exécution parallèle, comparaison de résultats)
5. **Sprint 5** : fibcalc-cli (CLI output, progress, calibration)
6. **Sprint 6** : fibcalc-tui (dashboard ratatui, sparklines, métriques)
7. **Sprint 7** : Intégration, optimisation, validation croisée Go/Rust

## Structure Cargo Workspace cible

```
fibcalc-rs/
├── Cargo.toml                    # Workspace root
├── CLAUDE.md                     # AI assistant guidance (Rust)
├── crates/
│   ├── fibcalc-core/             # Types, traits, algorithmes
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── calculator.rs     # Calculator trait + FibCalculator decorator
│   │       ├── strategy.rs       # Multiplier, DoublingStepExecutor traits
│   │       ├── fast_doubling.rs  # OptimizedFastDoubling
│   │       ├── matrix.rs         # MatrixExponentiation
│   │       ├── fft_based.rs      # FFTBasedCalculator
│   │       ├── doubling_framework.rs
│   │       ├── matrix_framework.rs
│   │       ├── matrix_ops.rs
│   │       ├── observer.rs       # ProgressSubject, ProgressObserver
│   │       ├── progress.rs       # ProgressUpdate, geometric model
│   │       ├── options.rs        # Options, constants
│   │       ├── dynamic_threshold.rs
│   │       ├── modular.rs        # FastDoublingMod
│   │       ├── generator.rs      # SequenceGenerator + IterativeGenerator
│   │       ├── arena.rs          # CalculationArena (bumpalo)
│   │       ├── memory_budget.rs
│   │       └── registry.rs       # CalculatorFactory
│   ├── fibcalc-bigfft/           # FFT multiplication engine
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── fft.rs            # Public API (Mul, Sqr)
│   │       ├── fft_core.rs       # Core FFT operations
│   │       ├── fft_recursion.rs  # Parallel recursion
│   │       ├── fft_poly.rs       # Polynomial operations
│   │       ├── fermat.rs         # Fermat arithmetic
│   │       ├── fft_cache.rs      # LRU transform cache
│   │       ├── pool.rs           # Object pooling
│   │       ├── bump.rs           # Bump allocator
│   │       └── scan.rs           # Decimal string parsing
│   ├── fibcalc-orchestration/    # Concurrent execution
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── orchestrator.rs
│   │       └── calculator_selection.rs
│   ├── fibcalc-cli/              # CLI presentation
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── output.rs
│   │       ├── presenter.rs
│   │       ├── progress_eta.rs
│   │       └── completion.rs
│   ├── fibcalc-tui/              # TUI dashboard
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs
│   │       ├── app.rs            # Main event loop (Elm-like)
│   │       ├── bridge.rs
│   │       ├── chart.rs
│   │       ├── sparkline.rs
│   │       ├── logs.rs
│   │       ├── metrics.rs
│   │       └── styles.rs
│   └── fibcalc-calibration/      # Auto-tuning
│       ├── Cargo.toml
│       └── src/
│           ├── lib.rs
│           ├── calibration.rs
│           ├── adaptive.rs
│           ├── profile.rs
│           └── microbench.rs
├── src/
│   └── main.rs                   # Binary entry point
├── tests/
│   ├── golden/                   # Golden file tests
│   └── e2e/                      # End-to-end tests
└── docs/                         # Documentation mirror
```

## Critères de validation finale

Le portage Rust sera considéré comme **complet** lorsque :

1. ✅ Tous les 100+ fichiers Go ont une correspondance Rust fonctionnelle
2. ✅ Les résultats sont identiques à Go pour N ∈ {0, 1, 93, 1K, 10K, 100K, 1M, 10M, 100M}
3. ✅ Les performances Rust sont dans la marge de 5% des baselines Go (ou meilleures)
4. ✅ La couverture de tests dépasse 75%
5. ✅ Les 5 fuzz targets sont portés et fonctionnels
6. ✅ Le TUI ratatui est visuellement équivalent au TUI Bubble Tea
7. ✅ La calibration automatique fonctionne sur les 5 triples cibles
8. ✅ Le binaire stripped est < 5 MB
9. ✅ Le temps de démarrage est < 50 ms
10. ✅ Tous les 20+ scénarios d'intégration E2E passent

---

*Document généré automatiquement par une équipe de 5 agents Claude Code travaillant en parallèle.*
*Total : 78 tâches complétées sur 7 phases.*
