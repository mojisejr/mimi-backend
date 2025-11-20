//! Prompt Management
//!
//! Loads and manages system prompts for different agent stages.

/// Load a prompt from the prompts directory
pub fn load_prompt(agent_name: &str) -> Result<String, String> {
    // TODO: Implement prompt loading logic
    // - Load from prompts/{agent_name}.md files
    // - Handle file not found errors
    // - Return prompt content
    Ok(format!("Prompt for agent: {}", agent_name))
}
