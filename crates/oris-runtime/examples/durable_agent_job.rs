//! Durable agent job: interrupt, restart, resume.
//!
//! This example shows the three things that make Oris a **runtime** (not just a library):
//!
//! 1. **Task can be interrupted** — Execution pauses at a node (e.g. for human approval).
//! 2. **Process can restart** — You can exit and run the program again later (e.g. next day).
//! 3. **State is not lost** — The same `thread_id` + checkpointer lets you resume from the last checkpoint.
//!
//! Flow: "research" step → "approval" step (interrupt) → continue after resume.
//!
//! Run:
//!   cargo run -p oris-runtime --example durable_agent_job
//!
//! The example runs two phases in one process to demonstrate resume; in production,
//! the second run would be a separate process or cron job using the same thread_id.

use oris_runtime::graph::{
    function_node, interrupt, Command, GraphError, InMemorySaver, MessagesState, RunnableConfig,
    StateGraph, StateOrCommand, END, START,
};
use oris_runtime::schemas::messages::Message;
use std::collections::HashMap;

const THREAD_ID: &str = "durable-job-1";

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Node 1: "research" — simulates doing work (e.g. fetch repo readme, summarize)
    let research_node = function_node("research", |_state: &MessagesState| async move {
        let mut update = HashMap::new();
        update.insert(
            "messages".to_string(),
            serde_json::to_value(vec![Message::new_ai_message(
                "Research done: Oris is a programmable execution runtime for AI agents in Rust.",
            )])?,
        );
        Ok(update)
    });

    // Node 2: "approval" — pauses for human decision; resume continues from here
    let approval_node = function_node("approval", |_state: &MessagesState| async move {
        let approved = interrupt(serde_json::json!({
            "question": "Proceed with this result? (simulated: say yes to continue)",
            "context": "Durable job checkpoint — you can restart the process and resume with the same thread_id."
        }))
        .await
        .map_err(GraphError::InterruptError)?;

        let status = if approved.as_bool().unwrap_or(false) {
            "approved"
        } else {
            "rejected"
        };
        let mut update = HashMap::new();
        update.insert(
            "messages".to_string(),
            serde_json::to_value(vec![Message::new_ai_message(format!(
                "Approval result: {}. Job complete.",
                status
            ))])?,
        );
        Ok(update)
    });

    let mut graph = StateGraph::<MessagesState>::new();
    graph.add_node("research", research_node)?;
    graph.add_node("approval", approval_node)?;
    graph.add_edge(START, "research");
    graph.add_edge("research", "approval");
    graph.add_edge("approval", END);

    let checkpointer = std::sync::Arc::new(InMemorySaver::new());
    let compiled = graph.compile_with_persistence(Some(checkpointer.clone()), None)?;
    let config = RunnableConfig::with_thread_id(THREAD_ID);

    // ----- Phase 1: run until interrupt -----
    println!("Phase 1: Start job (will stop at approval)");
    let initial_state =
        MessagesState::with_messages(vec![Message::new_human_message("Start durable job")]);
    let result = compiled
        .invoke_with_config_interrupt(StateOrCommand::State(initial_state), &config)
        .await?;

    if let Some(interrupts) = &result.interrupt {
        println!(
            "  -> Interrupted for approval ({} interrupt(s))",
            interrupts.len()
        );
        for (i, v) in interrupts.iter().enumerate() {
            println!("     [{}] {}", i + 1, v.value);
        }
        println!("\n  (In a real scenario you would exit here; later you resume with the same thread_id.)");
    }

    // ----- Phase 2: resume (simulates "next day" or another process) -----
    println!("\nPhase 2: Resume with same thread_id (simulated approval: true)");
    let resumed = compiled
        .invoke_with_config_interrupt(
            StateOrCommand::Command(Command::resume(serde_json::json!(true))),
            &config,
        )
        .await?;

    println!("  -> Final messages:");
    for msg in &resumed.state.messages {
        println!("     {}: {}", msg.message_type.to_string(), msg.content);
    }
    println!("\nDone. State was checkpointed; restarting with thread_id \"{}\" would resume from the last checkpoint.", THREAD_ID);

    Ok(())
}
