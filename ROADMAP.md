<!-- GENERATED from roadmap.yaml. Do not edit ROADMAP.md directly. -->

# College Football Simulator Roadmap

This roadmap records product direction, not delivery dates or release commitments. `roadmap.yaml` is authoritative; OpenSpec remains authoritative for detailed requirements and implementation evidence.

## Lifecycle

`exploring` → `proposed` → `active` → `complete`. Any non-complete item can become `deferred`; deferred work returns through `exploring`.

| Order | ID | Feature | Theme | Status | Dependencies |
|---:|---|---|---|---|---|
| 1 | `SIM-01` | Statistical game simulation | simulation | `complete` | — |
| 2 | `DYNASTY-01` | Player and roster model | dynasty | `exploring` | SIM-01 |
| 3 | `DYNASTY-02` | Positions and depth charts | dynasty | `exploring` | DYNASTY-01 |
| 4 | `SEASON-01` | Schedules and weekly season loop | season | `exploring` | SIM-01 |
| 5 | `DYNASTY-03` | Development, fatigue, and injuries | dynasty | `exploring` | DYNASTY-01, SEASON-01 |
| 6 | `RECRUIT-01` | Recruiting | recruiting | `exploring` | DYNASTY-01 |
| 7 | `DYNASTY-04` | Coaching staff, schemes, and tactics | dynasty | `exploring` | SIM-01, DYNASTY-01 |
| 8 | `DYNASTY-05` | Transfers, graduation, and offseason progression | dynasty | `exploring` | DYNASTY-01, DYNASTY-03, RECRUIT-01 |
| 9 | `PROGRAM-01` | Prestige, facilities, finances, and expectations | program | `exploring` | SEASON-01, RECRUIT-01 |
| 10 | `POSTSEASON-01` | Rankings, bowls, conference championships, and playoff | postseason | `exploring` | SEASON-01 |
| 11 | `HISTORY-01` | Records, awards, and historical continuity | history | `exploring` | SEASON-01, POSTSEASON-01 |

## SIM-01 — Statistical game simulation

- **Status:** `complete`
- **Theme:** simulation
- **Dependencies:** None
- **Outcome:** Rated teams produce deterministic, calibrated possession-level games and auditable statistical summaries.
- **Excludes:**
  - Player-level and play-by-play simulation
  - Historical-era-specific rules
- **Evidence:** [capability `possession-game-simulation`](openspec/specs/possession-game-simulation/spec.md), [capability `simulation-calibration`](openspec/specs/simulation-calibration/spec.md), [archived change `2026-08-05-add-possession-game-simulation`](openspec/changes/archive/2026-08-05-add-possession-game-simulation/)

## DYNASTY-01 — Player and roster model

- **Status:** `exploring`
- **Theme:** dynasty
- **Dependencies:** SIM-01
- **Outcome:** Programs maintain multi-season player rosters with positions, attributes, eligibility, and stable identity.
- **Excludes:**
  - Recruiting and transfer acquisition workflows
  - Player-level game simulation
- **Evidence:** None yet

## DYNASTY-02 — Positions and depth charts

- **Status:** `exploring`
- **Theme:** dynasty
- **Dependencies:** DYNASTY-01
- **Outcome:** Managers assign eligible players to positional units and ordered depth charts that expose team strengths to simulation.
- **Excludes:**
  - In-game substitutions and tactical packages
  - Injury and fatigue effects
- **Evidence:** None yet

## SEASON-01 — Schedules and weekly season loop

- **Status:** `exploring`
- **Theme:** season
- **Dependencies:** SIM-01
- **Outcome:** A dynasty advances through scheduled weeks, simulates games, updates standings, and preserves deterministic season state.
- **Excludes:**
  - Rankings and postseason selection
  - Conference schedule optimization
- **Evidence:** None yet

## DYNASTY-03 — Development, fatigue, and injuries

- **Status:** `exploring`
- **Theme:** dynasty
- **Dependencies:** DYNASTY-01, SEASON-01
- **Outcome:** Player ability and availability evolve through training, games, recovery, fatigue, and injuries across a season.
- **Excludes:**
  - Medical staff employment simulation
  - Detailed rehabilitation minigames
- **Evidence:** None yet

## RECRUIT-01 — Recruiting

- **Status:** `exploring`
- **Theme:** recruiting
- **Dependencies:** DYNASTY-01
- **Outcome:** Programs scout and pursue prospects through a competitive, explainable recruiting cycle that produces future rosters.
- **Excludes:**
  - Transfer portal movement
  - Real-world prospect data ingestion
- **Evidence:** None yet

## DYNASTY-04 — Coaching staff, schemes, and tactics

- **Status:** `exploring`
- **Theme:** dynasty
- **Dependencies:** SIM-01, DYNASTY-01
- **Outcome:** Managers hire staff and choose schemes and tactical instructions that visibly modify player development and matchup performance.
- **Excludes:**
  - User-controlled play calling
  - Full playbook authoring
- **Evidence:** None yet

## DYNASTY-05 — Transfers, graduation, and offseason progression

- **Status:** `exploring`
- **Theme:** dynasty
- **Dependencies:** DYNASTY-01, DYNASTY-03, RECRUIT-01
- **Outcome:** Rosters transition coherently between seasons through graduation, departures, transfers, arrivals, and offseason development.
- **Excludes:**
  - Professional draft simulation
  - Athlete compensation negotiation
- **Evidence:** None yet

## PROGRAM-01 — Prestige, facilities, finances, and expectations

- **Status:** `exploring`
- **Theme:** program
- **Dependencies:** SEASON-01, RECRUIT-01
- **Outcome:** Program resources, reputation, leadership expectations, and investments create long-term constraints and strategic tradeoffs.
- **Excludes:**
  - Detailed university-wide accounting
  - Real-money monetization systems
- **Evidence:** None yet

## POSTSEASON-01 — Rankings, bowls, conference championships, and playoff

- **Status:** `exploring`
- **Theme:** postseason
- **Dependencies:** SEASON-01
- **Outcome:** Season performance feeds transparent rankings and configurable postseason qualification, seeding, and championship resolution.
- **Excludes:**
  - Exact reproduction of every historical postseason format
  - Human committee simulation in the initial version
- **Evidence:** None yet

## HISTORY-01 — Records, awards, and historical continuity

- **Status:** `exploring`
- **Theme:** history
- **Dependencies:** SEASON-01, POSTSEASON-01
- **Outcome:** The dynasty preserves season histories, records, awards, champions, and career milestones for long-term storytelling.
- **Excludes:**
  - Importing complete real-world historical archives
  - Media narrative generation
- **Evidence:** None yet
