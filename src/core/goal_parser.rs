use crate::core::lifecycle::LifecyclePhase;
use std::str::FromStr;

/// Goal parser - parses Maven goals and maps them to lifecycle phases
pub struct GoalParser;

/// Parsed goal information
#[derive(Debug, Clone)]
pub enum Goal {
    /// Lifecycle phase goal (e.g., "compile", "package")
    Phase(LifecyclePhase),
    /// Plugin goal (e.g., "compiler:compile", "surefire:test")
    Plugin {
        group_id: Option<String>,
        artifact_id: String,
        goal: String,
    },
}

impl GoalParser {
    pub fn new() -> Self {
        Self
    }

    /// Parse a goal string into a Goal
    pub fn parse(&self, goal_str: &str) -> Result<Goal, String> {
        // Check if it's a lifecycle phase
        if let Ok(phase) = LifecyclePhase::from_str(goal_str) {
            return Ok(Goal::Phase(phase));
        }

        // Check if it's a plugin goal (format: groupId:artifactId:goal or artifactId:goal)
        if goal_str.contains(':') {
            let parts: Vec<&str> = goal_str.split(':').collect();
            
            match parts.len() {
                2 => {
                    // artifactId:goal
                    Ok(Goal::Plugin {
                        group_id: None,
                        artifact_id: parts[0].to_string(),
                        goal: parts[1].to_string(),
                    })
                }
                3 => {
                    // groupId:artifactId:goal
                    Ok(Goal::Plugin {
                        group_id: Some(parts[0].to_string()),
                        artifact_id: parts[1].to_string(),
                        goal: parts[2].to_string(),
                    })
                }
                _ => Err(format!("Invalid goal format: {goal_str}")),
            }
        } else {
            Err(format!("Unknown goal: {goal_str}"))
        }
    }

    /// Parse multiple goals
    pub fn parse_goals(&self, goals: &[String]) -> Result<Vec<Goal>, String> {
        goals.iter()
            .map(|g| self.parse(g))
            .collect()
    }

    /// Get target phase from goals (highest phase if multiple phases specified)
    pub fn get_target_phase(&self, goals: &[String]) -> Option<LifecyclePhase> {
        let all_phases = LifecyclePhase::all();
        let mut target_phase: Option<&LifecyclePhase> = None;

        for goal_str in goals {
            if let Ok(Goal::Phase(phase)) = self.parse(goal_str) {
                // Find the phase in the lifecycle
                if let Some(pos) = all_phases.iter().position(|p| p == &phase) {
                    let phase_ref = &all_phases[pos];
                    match target_phase {
                        None => target_phase = Some(phase_ref),
                        Some(current) => {
                            // Keep the later phase
                            if let Some(current_pos) = all_phases.iter().position(|p| p == current) {
                                if pos > current_pos {
                                    target_phase = Some(phase_ref);
                                }
                            }
                        }
                    }
                }
            }
        }

        target_phase.cloned()
    }
}

impl Default for GoalParser {
    fn default() -> Self {
        Self::new()
    }
}

