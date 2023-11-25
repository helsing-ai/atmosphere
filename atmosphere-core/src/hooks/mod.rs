use async_trait::async_trait;

use crate::{
    query::{Query, QueryResult},
    Bind, Result, Table,
};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HookStage {
    PreBind,
    PreExec,
    PostExec,
}

pub enum HookInput<'t, T: Table + Bind> {
    None,
    Row(&'t mut T),
    PrimaryKey(&'t T::PrimaryKey),
    QueryResult(QueryResult<'t, T>),
}

impl<'t, T: Table + Bind> From<QueryResult<'t, T>> for HookInput<'t, T> {
    fn from(value: QueryResult<'t, T>) -> Self {
        Self::QueryResult(value)
    }
}

#[async_trait]
pub trait Hook<T: Table + Bind + Sync>: Sync + Send {
    /// Stage
    fn stage(&self) -> HookStage;

    ///
    async fn apply(
        &self,
        #[allow(unused)] ctx: &Query<T>,
        #[allow(unused)] input: &HookInput<'_, T>,
    ) -> Result<()> {
        println!("{}", ctx.sql());
        Ok(())
    }
}

pub trait Hooks: Table + Bind {
    const HOOKS: &'static [&'static dyn Hook<Self>];
}

pub(crate) async fn execute<T: Hooks + Sync>(
    stage: HookStage,
    ctx: &Query<T>,
    input: HookInput<'_, T>,
) -> Result<()> {
    for hook in T::HOOKS {
        if hook.stage() != stage {
            continue;
        }

        hook.apply(ctx, &input).await?;
    }

    Ok(())
}
