# ternary-motion

**The full kinematics of ternary change. Where it is, where it's going, and how it's getting there.**

A ternary value `{-1, 0, +1}` tells you *where something is*. But that's a snapshot, not a story. To understand motion, you need velocity (which direction), acceleration (is it speeding up or slowing down), jerk (sudden changes), dwell time (how long it's been sitting still), phase (where in the oscillation cycle), and groove (is it locked in or fighting the beat).

This crate tracks all of that. A `MotionState` captures the complete kinematic state of a ternary agent — not just what it is, but its entire trajectory through state space. You feed it position readings, it computes everything else.

## What's Inside

- **`MotionState`** — the full kinematic snapshot:
  - `position` — current {-1, 0, +1}
  - `velocity` — rate of change (floating point, derived from transitions)
  - `acceleration` — change in velocity
  - `jerk` — change in acceleration (sudden shifts)
  - `ticks_in_state` — dwell time at current position
  - `transitions` — total state changes over lifetime
  - `phase` — position in oscillation cycle [0, 2π)
  - `angular_velocity` — how fast phase advances
- **`update(new_position)`** — feed a new reading, compute all derivatives
- **`MotionTracker`** — track multiple agents simultaneously
- **`rhythm_score()`** — how rhythmic is the motion? Regular transitions score high
- **`groove_lock(reference)`** — is this agent locked to a reference rhythm?

## Quick Example

```rust
use ternary_motion::*;

let mut motion = MotionState::new(0); // start at zero

// Feed position readings over time
motion.update(1);   // 0 → 1: velocity = +1, acceleration = +1
motion.update(1);   // 1 → 1: velocity = 0, acceleration = -1 (settling)
motion.update(-1);  // 1 → -1: velocity = -2, acceleration = -2, jerk = -1 (sudden!)
motion.update(0);   // -1 → 0: velocity = +1, acceleration = +3 (recovery)

println!("Position: {}", motion.position);       // 0
println!("Velocity: {:.1}", motion.velocity);    // +1.0
println!("Jerk: {:.1}", motion.jerk);            // big number — abrupt change
println!("Transitions: {}", motion.transitions);  // 3
println!("Phase: {:.2}π", motion.phase / PI);
```

## The Deeper Truth

**Velocity and acceleration reveal what position hides.** Two agents both at `+1` look identical — but one has been there for 100 ticks (stable, low velocity) while the other just arrived from `-1` (high velocity, changing fast). The kinematic state tells you *what's about to happen*, not just what's happening now.

The jerk is the most informative signal: high jerk means a sudden reversal. In trading, that's a regime change. In robotics, that's a collision. In conversation, that's a topic shift. Jerk detection is the earliest possible warning that something fundamental has changed.

The groove concept comes from music: is the agent's transition pattern aligned with a reference beat? Agents that are "in groove" transition on the beat. Agents "fighting" transition off-beat. The groove score quantifies this — and it turns out to be the most useful metric for predicting whether an agent will stay synchronized with a group.

**Use cases:**
- **Robotics** — track discrete state transitions with full kinematics
- **Trading** — position = current stance, velocity = momentum, jerk = regime change
- **Game AI** — detect when an NPC's behavior pattern shifts
- **Conversation analysis** — track topic transitions with velocity and jerk
- **Music** — groove detection for rhythmic alignment

## See Also

- **ternary-rhythm** — rhythm patterns and detection
- **ternary-phase** — phase relationships between agents
- **ternary-predict** — prediction using kinematic state
- **ternary-gauge** — simpler instrumentation (just statistics, no kinematics)
- **ternary-fib** — period-8 as the natural kinematic rhythm

## Install

```bash
cargo add ternary-motion
```

## License

MIT
