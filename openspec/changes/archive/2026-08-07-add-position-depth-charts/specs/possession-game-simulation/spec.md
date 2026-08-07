## MODIFIED Requirements

### Requirement: Current ratings influence matchup outcomes
The engine SHALL use the current `Team` offense, defense, special-teams, and overall ratings as baseline probability inputs. A caller MAY explicitly supply bounded roster-derived offense, defense, and special-teams strengths through the matchup modifier seam; when supplied, those modifiers SHALL combine with the baseline ratings deterministically. Increasing one team's relevant effective rating while holding all other inputs constant SHALL not reduce that team's expected aggregate performance for the affected dimension across the canonical seed set.

#### Scenario: Offensive strength improves expected production
- **WHEN** otherwise identical teams are compared over the canonical seed set and one team's baseline offense rating or roster-derived offense modifier is increased
- **THEN** that team's mean points or mean offensive efficiency does not decrease beyond the configured statistical tolerance

#### Scenario: Defensive strength suppresses opponents
- **WHEN** otherwise identical teams are compared over the canonical seed set and one team's baseline defense rating or roster-derived defense modifier is increased
- **THEN** its opponents' mean points or mean offensive efficiency does not increase beyond the configured statistical tolerance

#### Scenario: Special teams affects relevant events
- **WHEN** otherwise identical teams with different baseline special-teams ratings or roster-derived special-teams strengths are simulated over the canonical seed set
- **THEN** the stronger effective special-teams unit has no worse expected field-goal and field-position contribution beyond the configured statistical tolerance

#### Scenario: Neutral roster strength preserves baseline
- **WHEN** a valid depth chart derives neutral unit strengths and those strengths are adapted into matchup modifiers
- **THEN** the effective matchup inputs equal the aggregate `Team` baseline ratings

#### Scenario: Modifier composition is reproducible
- **WHEN** the same teams, roster-derived modifiers, configuration, and seed are simulated repeatedly
- **THEN** the ordered possessions, statistics, and final result are identical
