mod build;
mod model;
mod render;

pub(crate) use build::AgentWhoInput;

pub(crate) fn render_agent_who(
    store: &crate::state::Store,
    input: AgentWhoInput<'_>,
) -> anyhow::Result<String> {
    let aggregation = crate::who_aggregation::WhoAggregation::load(store, input.now)?;
    Ok(render::render_agent_who(&build::build_agent_who(
        store,
        &aggregation,
        input,
    )))
}

#[cfg(test)]
mod tests;
