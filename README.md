# Ternary Motion

Full kinematic analysis for ternary `{-1, 0, +1}` agent systems — tracking position, velocity, acceleration, jerk, rhythm, phase, and groove alignment across populations of agents that exist in exactly three states.

## Why It Matters

Many real-world systems — GPU kernel schedulers, consensus protocols, cellular automata, game-theoretic agents — operate on ternary state spaces. Yet most analysis tools treat these as simple labels, missing the **dynamics**: *how fast* agents are changing, *whether* the system is accelerating toward a transition, and *whether* the population is moving in sync.

Ternary Motion gives you the full kinematic toolkit. It answers questions like:

- "Is my system converging or about to flip?"
- "Are agents oscillating in lockstep, or chaotically?"
- "What's the rhythm — and does it align with the target cadence?"

This is critical for fleet orchestration, where detecting a **frozen** (deadlocked) vs. **chaotic** (thrashing) population early can prevent cascading failures.

## How It Works

### Motion Kinematics on a Discrete Lattice

For a ternary agent with position $x_t \in \{-1, 0, +1\}$ observed at discrete time $t$:

| Quantity | Definition | Range |
|---|---|---|
| **Velocity** | $v_t = x_t - x_{t-1}$ | $\{-2, -1, 0, +1, +2\}$ |
| **Acceleration** | $a_t = v_t - v_{t-1}$ | $\{-4, \ldots, +4\}$ |
| **Jerk** | $j_t = a_t - a_{t-1}$ | $\{-8, \ldots, +8\}$ |

**Kinetic energy** of a single agent:

$$E_k = \frac{1}{2}v^2 + \frac{1}{2}|a|$$

**Rhythm coherence** across a population of $N$ agents with velocities $\{v_i\}$:

$$C = \frac{1}{1 + \sigma^2_v}, \quad \text{where } \sigma^2_v = \frac{1}{N}\sum_{i=1}^{N}(v_i - \bar{v})^2$$

Low velocity variance → high coherence (all agents moving together). High variance → incoherent motion.

### Rhythm Extraction

Given a ternary time series $s = [s_0, s_1, \ldots, s_{n-1}]$, rhythm extraction finds the period $p$ that maximizes autocorrelation:

$$\text{strength}(p) = \frac{|\{i : s_i = s_{i+p}\}|}{n - p}$$

The algorithm scans $p \in [1, \lfloor n/2 \rfloor]$ and returns $(p^*, \text{strength}^*)$.

**Complexity:** $O(n^2)$ worst case for full scan, $O(n)$ for velocity/acceleration/jerk profile extraction.

### Groove Alignment

Groove measures whether transitions land on the beat:

$$G = \frac{|\{a : t \bmod B = 0 \wedge \text{transitions}(a) > 0\}|}{|\{a : \text{transitions}(a) > 0\}|}$$

where $B$ is the beat interval. $G = 1$ means every transition happens on-beat.

### System Health Classification

| State | Condition | Meaning |
|---|---|---|
| 🧊 Frozen | settled fraction > 0.9 | Deadlocked — nothing moving |
| 😌 Settling | speed < 0.1, settled > 0.5 | Converging to rest |
| 🔄 Transitioning | (default) | Mixed dynamics |
| 🎶 Grooving | speed > 0.5, coherence > 0.5 | Synchronized motion — optimal |
| 🌊 Chaotic | speed > 0.5, coherence < 0.3 | Random thrashing |

## Quick Start

```toml
[dependencies]
ternary-motion = "0.1"
```

```rust
use ternary_motion::{PopulationMotion, extract_rhythm};

// Track a population of 4 agents
let mut pop = PopulationMotion::new(&[1, 0, -1, 1]);

// Feed new positions over time
pop.update(&[-1, 0, 1, 0]);
pop.update(&[1, 0, -1, 1]);
pop.update(&[-1, 0, 1, 0]);

println!("Mean velocity: {:.3}", pop.mean_velocity());
println!("Rhythm coherence: {:.3}", pop.rhythm_coherence());
println!("System health: {}", pop.motion_health());

// Extract rhythm from a series
let (period, strength) = extract_rhythm(&[1, -1, 1, -1, 1, -1]);
assert_eq!(period, 2);
assert!(strength > 0.9);
```

## API

### `MotionState`
Single-agent kinematic state. Fields: `position`, `velocity`, `acceleration`, `jerk`, `ticks_in_state`, `transitions`, `phase`, `angular_velocity`.

Methods: `new(position)`, `update(new_position)`, `is_settled()`, `is_oscillating()`, `is_breaking_free()`, `kinetic_energy()`, `direction()`.

### `PopulationMotion`
Multi-agent tracker. Construct with `new(positions: &[i8])`, call `update(&[i8])` each tick.

Population metrics: `mean_velocity()`, `mean_speed()`, `mean_acceleration()`, `total_energy()`, `settled_fraction()`, `oscillating_fraction()`, `rhythm_coherence()`, `momentum()`, `angular_momentum()`, `groove_alignment(beat_interval)`, `motion_health()`.

### Free Functions
- `extract_rhythm(series: &[i8]) -> (usize, f64)` — autocorrelation-based period detection
- `velocity_profile(series: &[i8]) -> Vec<f64>` — first differences
- `acceleration_profile(series: &[i8]) -> Vec<f64>` — second differences
- `jerk_profile(series: &[i8]) -> Vec<f64>` — third differences
- `transition_count(series: &[i8]) -> usize` — count of state changes
- `transition_intervals(series: &[i8]) -> Vec<usize>` — gaps between transitions

## Architecture Notes

Ternary Motion is part of the SuperInstance ecosystem's **γ + η = C** framework:

- **γ (gamma)** — the *agent signal*: each agent's position in $\{-1, 0, +1\}$ encodes its strategic choice (bearish/neutral/bullish, deprioritize/normal/prioritize, etc.)
- **η (eta)** — the *environment response*: velocity, acceleration, and coherence describe how the environment reacts to agent decisions
- **C** — *coordination*: when γ and η are aligned (high groove alignment + high coherence), the system reaches the "grooving" state — the dynamical sweet spot for throughput

The crate is `#![forbid(unsafe_code)]` — pure safe Rust with zero dependencies.

## References

1. Strogatz, S. H. (2018). *Nonlinear Dynamics and Chaos*. CRC Press. — Rhythm, synchronization, and coupled oscillators.
2. Kantz, H., & Schreiber, T. (2004). *Nonlinear Time Series Analysis*. Cambridge University Press. — Lyapunov exponents and rhythm extraction.
3. Wolfram, S. (2002). *A New Kind of Science*. — Discrete dynamical systems on lattice state spaces.
4. Shader, M. (2021). "Ternary Cellular Automata and Their Dynamics." *J. Cellular Automata*, 16(3-4).

## License

MIT
