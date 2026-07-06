use std::sync::OnceLock;

use ureq::Agent;

static AGENT: OnceLock<Agent> = OnceLock::new();

pub fn agent() -> &'static Agent {
    AGENT.get_or_init(|| Agent::new_with_defaults())
}

pub fn set_agent(agent: Agent) -> Result<(), Agent> {
    AGENT.set(agent)
}
