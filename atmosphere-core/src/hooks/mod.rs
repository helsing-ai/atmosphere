//! Atmosphere Hook System
//!
//! This module provides a system for defining and applying hooks at various stages of query
//! execution. Hooks allow for custom logic to be executed at predetermined points in the query
//! lifecycle, such as before binding, before execution, and after execution. This functionality is
//! essential for implementing side effects, validations, or augmentations to the standard query
//! process.
//!
//! # Concepts
//!
//! - `HookStage`: An enum representing different stages in the query lifecycle where hooks can be applied.
//! - `HookInput`: An enum representing different types of input that can be provided to hooks.
//! - `Hook`: A trait defining a hook with a specific stage and an application method.
//! - `Hooks`: A trait for associating a set of hooks with a table entity.
//! - `execute`: A function to execute the appropriate hooks for a given stage and context.
//!
//! The hooks system is a powerful tool for extending and customizing the behavior of database operations,
//! enabling developers to embed additional logic seamlessly within the query execution flow.

use async_trait::async_trait;

use crate::{
    Bind, Result, Table,
    query::{Query, QueryResult},
};

/// Enumerates different stages in the query lifecycle for hook application.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookStage {
    /// Represents the stage before query parameters are bound.
    PreBind,
    /// Indicates the stage before query execution.
    PreExec,
    /// Denotes the stage after the query has been executed.
    PostExec,
}

/// Represents different types of input that can be provided to hooks.
pub enum HookInput<'t, T: Table + Bind> {
    /// No input is provided to the hook.
    None,
    /// A mutable reference to a table row entity.
    Row(&'t mut T),
    /// A reference to the primary key of a table entity.
    PrimaryKey(&'t T::PrimaryKey),
    /// The result of a query operation.
    QueryResult(QueryResult<'t, T>),
}

impl<'t, T: Table + Bind> From<QueryResult<'t, T>> for HookInput<'t, T> {
    fn from(value: QueryResult<'t, T>) -> Self {
        Self::QueryResult(value)
    }
}

/// A trait defining a hook for query execution.
///
/// Implementors of this trait can define custom logic to be executed at a specific stage of the
/// query lifecycle. The trait provides a method to specify the stage at which the hook should be
/// applied and another method to implement the hook's logic.
#[async_trait]
pub trait Hook<T: Table + Bind + Sync>: Sync + Send {
    /// Returns the stage at which the hook should be applied.
    fn stage(&self) -> HookStage;

    /// Asynchronously applies the hook logic to a given query context and input.
    async fn apply(&self, ctx: &Query<T>, input: &mut HookInput<'_, T>) -> Result<()> {
        let _ = ctx;
        let _ = input;
        Ok(())
    }
}

/// A trait for associating a set of hooks with a table entity.
///
/// Implementors can define a static array of hooks that are associated with a table entity. These
/// hooks are invoked at their respective stages during the query execution process, enabling
/// custom behaviors or validations.
pub trait Hooks: Table + Bind {
    /// A static array of references to hooks associated with the implementing table entity.
    const HOOKS: &'static [&'static dyn Hook<Self>];
}

pub(crate) async fn execute<T: Hooks + Sync>(
    stage: HookStage,
    ctx: &Query<T>,
    mut input: HookInput<'_, T>,
) -> Result<()> {
    for hook in T::HOOKS {
        if hook.stage() != stage {
            continue;
        }

        hook.apply(ctx, &mut input).await?;
    }

    Ok(())
}
