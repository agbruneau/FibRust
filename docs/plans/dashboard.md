# Score Optimization Dashboard

> Tracking execution of the implementation plan (92 → 97+)

**Started:** 2026-02-21
**Status:** 🔄 In Progress

---

## Phase 1 — Parallel Agents

### Agent 1 — Core/Performance (`feat/core-perf`)


| Task | Description                                          | Status    |
| ---- | ---------------------------------------------------- | --------- |
| 1.1  | Wire FFTBumpAllocator into FFT multiply path         | ⏳ Pending |
| 1.2  | Wire BigIntPool via PoolAllocator into FFT functions | ⏳ Pending |
| 1.3  | Add CPU Core Affinity for TUI vs Compute threads     | ⏳ Pending |
| 1.4  | Connect FFT memory estimation to budget checker      | ⏳ Pending |
| 1.5  | Final Agent 1 verification                           | ⏳ Pending |


### Agent 2 — Portability (`feat/portability`)


| Task | Description                              | Status    |
| ---- | ---------------------------------------- | --------- |
| 2.1  | Fix workspace rug declaration            | ⏳ Pending |
| 2.2  | Implement GmpCalculator                  | ⏳ Pending |
| 2.3  | Register GmpCalculator in DefaultFactory | ⏳ Pending |
| 2.4  | Add CI workflow for dual-build testing   | ⏳ Pending |
| 2.5  | Add feature flag documentation to lib.rs | ⏳ Pending |
| 2.6  | Final Agent 2 verification               | ⏳ Pending |


### Agent 3 — Documentation (`feat/documentation`)


| Task | Description                        | Status    |
| ---- | ---------------------------------- | --------- |
| 3.1  | Add workspace metadata inheritance | ⏳ Pending |
| 3.2  | Create INSTALLATION.md             | ⏳ Pending |
| 3.3  | Rewrite README.md                  | ⏳ Pending |
| 3.4  | Improve rustdoc coverage           | ⏳ Pending |
| 3.5  | Update CHANGELOG                   | ⏳ Pending |
| 3.6  | Final Agent 3 verification         | ⏳ Pending |


---

## Phase 2 — Validation & Integration

### Agent 4 — Validation (`feat/validation`)


| Task | Description                         | Status    |
| ---- | ----------------------------------- | --------- |
| 4.1  | Capture baseline benchmarks on main | ⏳ Pending |
| 4.2  | Run full test matrix                | ⏳ Pending |
| 4.3  | Write allocator integration test    | ⏳ Pending |
| 4.4  | Core affinity fallback test         | ⏳ Pending |
| 4.5  | Code review checklist               | ⏳ Pending |
| 4.6  | Self-assessment scoring             | ⏳ Pending |


---

## Final Steps


| Task | Description      | Status    |
| ---- | ---------------- | --------- |
| F.1  | Update README.md | ⏳ Pending |
| F.2  | Update CLAUDE.md | ⏳ Pending |


---

## Score Projection


| Category                  | Before     | Target     | Current |
| ------------------------- | ---------- | ---------- | ------- |
| Architecture & Modularité | 24/25      | 25/25      | —       |
| Complexité Algorithmique  | 25/25      | 25/25      | —       |
| Fiabilité & Tests         | 20/20      | 20/20      | —       |
| Performance & Mémoire     | 18/20      | 20/20      | —       |
| Documentation & Outillage | 5/10       | 9/10       | —       |
| **Total**                 | **92/100** | **99/100** | —       |


