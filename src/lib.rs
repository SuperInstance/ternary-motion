#![forbid(unsafe_code)]
//! Ternary motion — the full kinematics of {-1,0,+1} change.
//! 
//! Position is where it IS. Velocity is where it's GOING.
//! Acceleration is whether it's speeding up or settling.
//! Rhythm is the pattern of change over time.
//! Phase is where in the cycle it sits.
//! Groove is whether it's locked in or fighting the system.

// ============================================================
// Core types — the full state of a moving ternary agent
// ============================================================

/// The complete motion state of a single ternary agent.
#[derive(Debug, Clone)]
pub struct MotionState {
    pub position: i8,           // Current value: -1, 0, +1
    pub velocity: f64,          // Rate of change: negative = heading toward -1, positive = toward +1
    pub acceleration: f64,      // Change in velocity: is it speeding up or settling?
    pub jerk: f64,              // Change in acceleration: sudden shifts
    pub ticks_in_state: u64,    // How long at current position (dwell time)
    pub transitions: u64,       // Total state changes over lifetime
    pub phase: f64,             // Where in the oscillation cycle [0, 2π)
    pub angular_velocity: f64,  // How fast the phase is advancing
}

impl MotionState {
    pub fn new(position: i8) -> Self {
        Self {
            position, velocity: 0.0, acceleration: 0.0, jerk: 0.0,
            ticks_in_state: 0, transitions: 0, phase: 0.0, angular_velocity: 0.0,
        }
    }

    /// Update motion state from a new position reading.
    pub fn update(&mut self, new_position: i8) {
        let new_velocity = (new_position - self.position) as f64;
        let new_acceleration = new_velocity - self.velocity;
        let new_jerk = new_acceleration - self.acceleration;

        self.jerk = new_jerk;
        self.acceleration = new_acceleration;
        self.velocity = new_velocity;

        if new_position != self.position {
            self.transitions += 1;
            self.ticks_in_state = 0;
        } else {
            self.ticks_in_state += 1;
        }
        self.position = new_position;

        // Update phase based on position mapping
        self.phase = match self.position {
            -1 => std::f64::consts::PI,           // Bottom of cycle
            0 => std::f64::consts::PI / 2.0,      // Quarter way
            _ => 0.0,                                // Top of cycle
        };
        self.angular_velocity = self.velocity.abs();
    }

    /// Is this agent settled (not moving)?
    pub fn is_settled(&self) -> bool {
        self.velocity.abs() < 0.01 && self.acceleration.abs() < 0.01 && self.ticks_in_state > 5
    }

    /// Is this agent oscillating (changing direction regularly)?
    pub fn is_oscillating(&self) -> bool {
        self.velocity.abs() > 0.5 && self.transitions > 3
    }

    /// Is this agent accelerating toward a new state?
    pub fn is_breaking_free(&self) -> bool {
        self.acceleration.abs() > 0.5
    }

    /// Energy of motion (kinetic + potential).
    pub fn kinetic_energy(&self) -> f64 {
        0.5 * self.velocity.powi(2) + 0.5 * self.acceleration.abs()
    }

    /// Direction label.
    pub fn direction(&self) -> &'static str {
        if self.velocity > 0.3 { "rising" }
        else if self.velocity < -0.3 { "falling" }
        else if self.position == 0 { "at_spindle" }
        else { "holding" }
    }
}

// ============================================================
// Population motion — tracking a whole system
// ============================================================

/// Motion analysis for an entire population.
pub struct PopulationMotion {
    pub agents: Vec<MotionState>,
    pub tick: u64,
}

impl PopulationMotion {
    pub fn new(positions: &[i8]) -> Self {
        Self {
            agents: positions.iter().map(|&p| MotionState::new(p)).collect(),
            tick: 0,
        }
    }

    /// Feed new positions and update all motion states.
    pub fn update(&mut self, new_positions: &[i8]) {
        for (i, &pos) in new_positions.iter().enumerate() {
            if i < self.agents.len() {
                self.agents[i].update(pos);
            }
        }
        self.tick += 1;
    }

    // ---- Population-level metrics ----

    /// Average velocity — which direction the population is heading.
    pub fn mean_velocity(&self) -> f64 {
        if self.agents.is_empty() { return 0.0; }
        self.agents.iter().map(|a| a.velocity).sum::<f64>() / self.agents.len() as f64
    }

    /// Average speed (absolute velocity) — how fast things are changing.
    pub fn mean_speed(&self) -> f64 {
        if self.agents.is_empty() { return 0.0; }
        self.agents.iter().map(|a| a.velocity.abs()).sum::<f64>() / self.agents.len() as f64
    }

    /// Average acceleration — is the system speeding up or settling?
    pub fn mean_acceleration(&self) -> f64 {
        if self.agents.is_empty() { return 0.0; }
        self.agents.iter().map(|a| a.acceleration).sum::<f64>() / self.agents.len() as f64
    }

    /// Total kinetic energy of the system.
    pub fn total_energy(&self) -> f64 {
        self.agents.iter().map(|a| a.kinetic_energy()).sum()
    }

    /// Fraction of agents that are settled (not moving).
    pub fn settled_fraction(&self) -> f64 {
        if self.agents.is_empty() { return 0.0; }
        self.agents.iter().filter(|a| a.is_settled()).count() as f64 / self.agents.len() as f64
    }

    /// Fraction of agents that are oscillating.
    pub fn oscillating_fraction(&self) -> f64 {
        if self.agents.is_empty() { return 0.0; }
        self.agents.iter().filter(|a| a.is_oscillating()).count() as f64 / self.agents.len() as f64
    }

    /// Rhythm coherence — are agents changing in sync?
    /// High coherence = agents transitioning at the same time.
    pub fn rhythm_coherence(&self) -> f64 {
        if self.agents.is_empty() || self.tick < 2 { return 0.0; }
        let velocities: Vec<f64> = self.agents.iter().map(|a| a.velocity).collect();
        let mean = velocities.iter().sum::<f64>() / velocities.len() as f64;
        let variance = velocities.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / velocities.len() as f64;
        // Low variance = high coherence (all moving together)
        1.0 / (1.0 + variance)
    }

    /// Phase distribution — how many agents at each phase of the cycle.
    pub fn phase_distribution(&self) -> PhaseDistribution {
        let mut dist = PhaseDistribution::default();
        for a in &self.agents {
            match a.position {
                -1 => dist.neg_count += 1,
                0 => dist.zero_count += 1,
                _ => dist.pos_count += 1,
            }
            if a.is_oscillating() { dist.oscillating += 1; }
            if a.is_settled() { dist.settled += 1; }
            if a.is_breaking_free() { dist.breaking_free += 1; }
        }
        dist.total = self.agents.len();
        dist
    }

    /// Groove alignment — how well the population's rhythm matches a target BPM.
    /// Measures whether transitions happen on-beat.
    pub fn groove_alignment(&self, beat_interval: u64) -> f64 {
        if self.tick == 0 { return 0.0; }
        let on_beat = self.agents.iter()
            .filter(|a| a.transitions > 0 && self.tick % beat_interval == 0)
            .count();
        let total_active = self.agents.iter().filter(|a| a.transitions > 0).count();
        if total_active == 0 { return 1.0; }
        on_beat as f64 / total_active as f64
    }

    /// Direction breakdown — how many going each way.
    pub fn direction_breakdown(&self) -> DirectionBreakdown {
        let mut bd = DirectionBreakdown::default();
        for a in &self.agents {
            match a.direction() {
                "rising" => bd.rising += 1,
                "falling" => bd.falling += 1,
                "at_spindle" => bd.at_spindle += 1,
                _ => bd.holding += 1,
            }
        }
        bd.total = self.agents.len();
        bd
    }

    /// Momentum — mass × velocity. Positive = system trending toward +1.
    pub fn momentum(&self) -> f64 {
        self.agents.iter().map(|a| a.velocity).sum()
    }

    /// Angular momentum — how much rotational force in the system.
    /// Agents oscillating between -1 and +1 create angular momentum.
    pub fn angular_momentum(&self) -> f64 {
        self.agents.iter().map(|a| a.angular_velocity * a.phase.sin()).sum()
    }

    /// System health from motion perspective.
    pub fn motion_health(&self) -> MotionHealth {
        let speed = self.mean_speed();
        let settled = self.settled_fraction();
        let energy = self.total_energy();
        let coherence = self.rhythm_coherence();

        if settled > 0.9 { MotionHealth::Frozen }
        else if speed > 0.5 && coherence > 0.5 { MotionHealth::Grooving }  // Moving together
        else if speed > 0.5 && coherence < 0.3 { MotionHealth::Chaotic }   // Moving randomly
        else if speed < 0.1 && settled > 0.5 { MotionHealth::Settling }    // Coming to rest
        else { MotionHealth::Transitioning }
    }
}

#[derive(Debug, Clone, Default)]
pub struct PhaseDistribution {
    pub neg_count: usize,
    pub zero_count: usize,
    pub pos_count: usize,
    pub oscillating: usize,
    pub settled: usize,
    pub breaking_free: usize,
    pub total: usize,
}

impl PhaseDistribution {
    pub fn neg_frac(&self) -> f64 { if self.total == 0 { 0.0 } else { self.neg_count as f64 / self.total as f64 } }
    pub fn zero_frac(&self) -> f64 { if self.total == 0 { 0.0 } else { self.zero_count as f64 / self.total as f64 } }
    pub fn pos_frac(&self) -> f64 { if self.total == 0 { 0.0 } else { self.pos_count as f64 / self.total as f64 } }
}

#[derive(Debug, Clone, Default)]
pub struct DirectionBreakdown {
    pub rising: usize,
    pub falling: usize,
    pub holding: usize,
    pub at_spindle: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MotionHealth {
    Frozen,       // Everything stopped
    Settling,     // Slowing down
    Transitioning, // Mixed
    Grooving,     // Moving in sync — the sweet spot
    Chaotic,      // Moving randomly
}

impl std::fmt::Display for MotionHealth {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MotionHealth::Frozen => write!(f, "🧊 FROZEN"),
            MotionHealth::Settling => write!(f, "😌 SETTLING"),
            MotionHealth::Transitioning => write!(f, "🔄 TRANSITIONING"),
            MotionHealth::Grooving => write!(f, "🎶 GROOVING"),
            MotionHealth::Chaotic => write!(f, "🌊 CHAOTIC"),
        }
    }
}

// ============================================================
// Rhythm extraction — find the patterns in change
// ============================================================

/// Extract rhythm from a ternary time series.
/// Returns (period, strength) — how often the pattern repeats and how strongly.
pub fn extract_rhythm(series: &[i8]) -> (usize, f64) {
    if series.len() < 4 { return (0, 0.0); }
    let mut best_period = 1;
    let mut best_strength = 0.0;
    let max_period = series.len() / 2;

    for period in 1..=max_period {
        let mut matches = 0;
        let mut total = 0;
        for i in 0..series.len() - period {
            if series[i] == series[i + period] { matches += 1; }
            total += 1;
        }
        let strength = if total > 0 { matches as f64 / total as f64 } else { 0.0 };
        if strength > best_strength {
            best_strength = strength;
            best_period = period;
        }
    }
    (best_period, best_strength)
}

/// Velocity profile — compute velocity at each step of a series.
pub fn velocity_profile(series: &[i8]) -> Vec<f64> {
    series.windows(2).map(|w| (w[1] - w[0]) as f64).collect()
}

/// Acceleration profile — compute acceleration at each step.
pub fn acceleration_profile(series: &[i8]) -> Vec<f64> {
    let velocities = velocity_profile(series);
    velocities.windows(2).map(|w| w[1] - w[0]).collect()
}

/// Jerk profile — sudden changes in acceleration.
pub fn jerk_profile(series: &[i8]) -> Vec<f64> {
    let accels = acceleration_profile(series);
    accels.windows(2).map(|w| w[1] - w[0]).collect()
}

/// Detect transitions — count how many times the series changes value.
pub fn transition_count(series: &[i8]) -> usize {
    series.windows(2).filter(|w| w[0] != w[1]).count()
}

/// Transition rhythm — the pattern of gaps between transitions.
pub fn transition_intervals(series: &[i8]) -> Vec<usize> {
    let mut intervals = Vec::new();
    let mut last_transition = 0;
    for (i, w) in series.windows(2).enumerate() {
        if w[0] != w[1] {
            intervals.push(i - last_transition);
            last_transition = i;
        }
    }
    intervals
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test] fn test_motion_state_creation() { let ms = MotionState::new(1); assert_eq!(ms.position, 1); assert_eq!(ms.velocity, 0.0); }
    #[test] fn test_motion_update() { let mut ms = MotionState::new(1); ms.update(0); assert_eq!(ms.position, 0); assert!(ms.velocity < 0.0); }
    #[test] fn test_motion_transition_count() { let mut ms = MotionState::new(1); ms.update(0); ms.update(-1); assert_eq!(ms.transitions, 2); }
    #[test] fn test_motion_dwell_time() { let mut ms = MotionState::new(1); ms.update(1); assert_eq!(ms.ticks_in_state, 1); }
    #[test] fn test_motion_settled() { let mut ms = MotionState::new(1); for _ in 0..10 { ms.update(1); } assert!(ms.is_settled()); }
    #[test] fn test_motion_oscillating() { let mut ms = MotionState::new(1); ms.update(-1); ms.update(1); ms.update(-1); ms.update(1); assert!(ms.is_oscillating()); }
    #[test] fn test_motion_direction_rising() { let mut ms = MotionState::new(-1); ms.update(1); assert_eq!(ms.direction(), "rising"); }
    #[test] fn test_motion_direction_falling() { let mut ms = MotionState::new(1); ms.update(-1); assert_eq!(ms.direction(), "falling"); }
    #[test] fn test_motion_direction_holding() { let mut ms = MotionState::new(1); ms.update(1); assert_eq!(ms.direction(), "holding"); }
    #[test] fn test_motion_kinetic_energy() { let ms = MotionState::new(0); assert!(ms.kinetic_energy() >= 0.0); }
    #[test] fn test_population_creation() { let pm = PopulationMotion::new(&[1,0,-1]); assert_eq!(pm.agents.len(), 3); }
    #[test] fn test_population_update() { let mut pm = PopulationMotion::new(&[1,0,-1]); pm.update(&[0,0,0]); assert_eq!(pm.tick, 1); }
    #[test] fn test_mean_velocity() { let mut pm = PopulationMotion::new(&[1,-1]); pm.update(&[-1,1]); assert!(pm.mean_velocity().abs() < 0.01); }
    #[test] fn test_mean_speed() { let mut pm = PopulationMotion::new(&[1,-1]); pm.update(&[-1,1]); assert!(pm.mean_speed() > 0.0); }
    #[test] fn test_total_energy() { let mut pm = PopulationMotion::new(&[1,0,-1]); pm.update(&[-1,0,1]); assert!(pm.total_energy() > 0.0); }
    #[test] fn test_settled_fraction() { let mut pm = PopulationMotion::new(&[1,1]); for _ in 0..10 { pm.update(&[1,1]); } assert!(pm.settled_fraction() > 0.5); }
    #[test] fn test_rhythm_coherence() { let mut pm = PopulationMotion::new(&[1,1,1]); pm.update(&[-1,-1,-1]); pm.update(&[1,1,1]); pm.update(&[-1,-1,-1]); assert!(pm.rhythm_coherence() > 0.3, "coherence={}", pm.rhythm_coherence()); }
    #[test] fn test_phase_distribution() { let pm = PopulationMotion::new(&[1,0,-1,1,0]); let d = pm.phase_distribution(); assert_eq!(d.pos_count, 2); assert_eq!(d.zero_count, 2); assert_eq!(d.neg_count, 1); }
    #[test] fn test_direction_breakdown() { let mut pm = PopulationMotion::new(&[1,-1,0]); pm.update(&[-1,1,0]); let bd = pm.direction_breakdown(); assert_eq!(bd.total, 3); }
    #[test] fn test_momentum() { let mut pm = PopulationMotion::new(&[1,0]); pm.update(&[1,1]); assert!(pm.momentum() > 0.0); }
    #[test] fn test_motion_health_grooving() { let mut pm = PopulationMotion::new(&[1,1,1,1]); pm.update(&[-1,-1,-1,-1]); pm.update(&[1,1,1,1]); let h = pm.motion_health(); assert!(matches!(h, MotionHealth::Chaotic | MotionHealth::Grooving | MotionHealth::Transitioning)); }
    #[test] fn test_motion_health_frozen() { let mut pm = PopulationMotion::new(&[1,1,1,1]); for _ in 0..20 { pm.update(&[1,1,1,1]); } assert_eq!(pm.motion_health(), MotionHealth::Frozen); }
    #[test] fn test_extract_rhythm_constant() { let (p, s) = extract_rhythm(&[1,1,1,1,1,1]); assert_eq!(p, 1); assert!(s > 0.9); }
    #[test] fn test_extract_rhythm_alternating() { let (p, s) = extract_rhythm(&[1,-1,1,-1,1,-1]); assert_eq!(p, 2); assert!(s > 0.9); }
    #[test] fn test_extract_rhythm_triple() { let (p, s) = extract_rhythm(&[1,0,-1,1,0,-1]); assert_eq!(p, 3); assert!(s > 0.9); }
    #[test] fn test_velocity_profile() { let vp = velocity_profile(&[1,0,-1]); assert_eq!(vp.len(), 2); assert!((vp[0] - (-1.0)).abs() < 0.01); }
    #[test] fn test_acceleration_profile() { let ap = acceleration_profile(&[1,0,-1,0]); assert_eq!(ap.len(), 2); }
    #[test] fn test_jerk_profile() { let jp = jerk_profile(&[1,0,-1,0,1]); assert!(jp.len() >= 2, "len={}", jp.len()); }
    #[test] fn test_transition_count() { assert_eq!(transition_count(&[1,1,0,0,-1]), 2); }
    #[test] fn test_transition_count_none() { assert_eq!(transition_count(&[1,1,1,1]), 0); }
    #[test] fn test_transition_intervals() { let iv = transition_intervals(&[1,0,0,-1]); assert_eq!(iv.len(), 2); }
    #[test] fn test_breaking_free() { let mut ms = MotionState::new(0); ms.velocity = -1.0; ms.update(1); assert!(ms.is_breaking_free()); }
    #[test] fn test_phase_distribution_fracs() { let pm = PopulationMotion::new(&[1,0,-1]); let d = pm.phase_distribution(); assert!((d.pos_frac() - 1.0/3.0).abs() < 0.01); }
    #[test] fn test_empty_rhythm() { assert_eq!(extract_rhythm(&[]), (0, 0.0)); }
    #[test] fn test_short_rhythm() { assert_eq!(extract_rhythm(&[1]), (0, 0.0)); }
}
