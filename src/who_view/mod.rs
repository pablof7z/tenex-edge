mod build;
mod model;
mod render;

pub(crate) use build::AgentWhoInput;

pub(crate) fn render_agent_who(store: &crate::state::Store, input: AgentWhoInput<'_>) -> String {
    render::render_agent_who(&build::build_agent_who(store, input))
}

#[cfg(test)]
mod tests;
