use std::collections::HashMap;
use std::sync::Arc;

use async_trait::async_trait;

use super::Node;
use crate::graph::{
    compiled::CompiledGraph,
    error::GraphError,
    persistence::{config::RunnableConfig, store::StoreBox},
    state::State,
    StateUpdate,
};

/// Subgraph node - wraps a CompiledGraph as a node
///
/// This allows a graph to be used as a node in another graph.
/// The subgraph and parent graph share the same state type.
///
/// # Example
///
/// ```ignore
/// use oris_runtime::graph::{StateGraph, MessagesState, CompiledGraph};
/// let mut subgraph = StateGraph::<MessagesState>::new();
/// let compiled_subgraph = subgraph.compile().unwrap();
/// let mut parent = StateGraph::<MessagesState>::new();
/// parent.add_subgraph("subgraph_node", compiled_subgraph).unwrap();
/// ```
pub struct SubgraphNode<S: State + 'static> {
    subgraph: Arc<CompiledGraph<S>>,
    name: String,
}

impl<S: State + 'static> SubgraphNode<S> {
    /// Create a new subgraph node
    pub fn new(name: impl Into<String>, subgraph: CompiledGraph<S>) -> Self {
        Self {
            subgraph: Arc::new(subgraph),
            name: name.into(),
        }
    }

    /// Get the name of the subgraph node
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a reference to the subgraph
    pub fn subgraph(&self) -> &CompiledGraph<S> {
        &self.subgraph
    }
}

#[async_trait]
impl<S: State + 'static> Node<S> for SubgraphNode<S> {
    async fn invoke(&self, state: &S) -> Result<StateUpdate, GraphError> {
        // Execute the subgraph with the current state
        let final_state = self.subgraph.invoke(state.clone()).await?;

        // Convert final state to state update
        // For shared state, we merge the final state back
        let state_json =
            serde_json::to_value(final_state).map_err(GraphError::SerializationError)?;

        let mut update = HashMap::new();
        if let serde_json::Value::Object(map) = state_json {
            for (key, value) in map {
                update.insert(key, value);
            }
        }

        Ok(update)
    }

    async fn invoke_with_context(
        &self,
        state: &S,
        config: Option<&RunnableConfig>,
        _store: Option<StoreBox>,
    ) -> Result<StateUpdate, GraphError> {
        // Execute the subgraph with config and store
        // Note: subgraph will inherit checkpointer from parent if provided
        let final_state = if let Some(config) = config {
            self.subgraph
                .invoke_with_config(Some(state.clone()), config)
                .await?
        } else {
            self.subgraph.invoke(state.clone()).await?
        };

        // Convert final state to state update
        let state_json =
            serde_json::to_value(final_state).map_err(GraphError::SerializationError)?;

        let mut update = HashMap::new();
        if let serde_json::Value::Object(map) = state_json {
            for (key, value) in map {
                update.insert(key, value);
            }
        }

        Ok(update)
    }

    fn get_subgraph(&self) -> Option<Arc<CompiledGraph<S>>> {
        Some(self.subgraph.clone())
    }
}

/// Subgraph node with state transformation
///
/// This allows a subgraph with a different state type to be used
/// as a node in a parent graph. State transformation functions are
/// provided to convert between parent and subgraph state types.
///
/// # Example
///
/// See [StateGraph::add_subgraph_with_transform] for usage with [crate::graph::MessagesState].
pub struct SubgraphNodeWithTransform<ParentState: State + 'static, SubState: State + 'static> {
    subgraph: Arc<CompiledGraph<SubState>>,
    name: String,
    transform_in: Arc<dyn Fn(&ParentState) -> Result<SubState, GraphError> + Send + Sync>,
    transform_out: Arc<dyn Fn(&SubState) -> Result<StateUpdate, GraphError> + Send + Sync>,
}

impl<ParentState: State + 'static, SubState: State + 'static>
    SubgraphNodeWithTransform<ParentState, SubState>
{
    /// Create a new subgraph node with transformation
    pub fn new(
        name: impl Into<String>,
        subgraph: CompiledGraph<SubState>,
        transform_in: impl Fn(&ParentState) -> Result<SubState, GraphError> + Send + Sync + 'static,
        transform_out: impl Fn(&SubState) -> Result<StateUpdate, GraphError> + Send + Sync + 'static,
    ) -> Self {
        Self {
            subgraph: Arc::new(subgraph),
            name: name.into(),
            transform_in: Arc::new(transform_in),
            transform_out: Arc::new(transform_out),
        }
    }

    /// Get the name of the subgraph node
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Get a reference to the subgraph
    pub fn subgraph(&self) -> &CompiledGraph<SubState> {
        &self.subgraph
    }
}

#[async_trait]
impl<ParentState: State + 'static, SubState: State + 'static> Node<ParentState>
    for SubgraphNodeWithTransform<ParentState, SubState>
{
    async fn invoke(&self, state: &ParentState) -> Result<StateUpdate, GraphError> {
        // Transform parent state to subgraph state
        let sub_state = (self.transform_in)(state)?;

        // Execute the subgraph
        let final_sub_state = self.subgraph.invoke(sub_state).await?;

        // Transform subgraph state back to parent state update
        (self.transform_out)(&final_sub_state)
    }

    async fn invoke_with_context(
        &self,
        state: &ParentState,
        config: Option<&RunnableConfig>,
        _store: Option<StoreBox>,
    ) -> Result<StateUpdate, GraphError> {
        // Transform parent state to subgraph state
        let sub_state = (self.transform_in)(state)?;

        // Execute the subgraph with config
        let final_sub_state = if let Some(config) = config {
            self.subgraph
                .invoke_with_config(Some(sub_state), config)
                .await?
        } else {
            self.subgraph.invoke(sub_state).await?
        };

        // Transform subgraph state back to parent state update
        (self.transform_out)(&final_sub_state)
    }

    // Note: get_subgraph is not implemented for SubgraphNodeWithTransform
    // because it has a different state type. Streaming would need special handling.
}
