use crate::{query::Query, Bind, Result, Table};

pub trait ValidationHook<T: Table + Bind> {
    /// Validation Stage
    fn apply(&self, #[allow(unused)] ctx: &Query<T>, #[allow(unused)] row: &T) -> Result<()> {
        Ok(())
    }
}

pub trait PreparationHook<T: Table + Bind> {
    /// Row Preperation Stage
    fn apply(&self, #[allow(unused)] ctx: &Query<T>, #[allow(unused)] row: &mut T) -> Result<()> {
        Ok(())
    }
}

pub trait InspectionHook<T: Table + Bind> {
    /// Inspection
    fn apply(&self, #[allow(unused)] ctx: &Query<T>) {}
}

pub trait TransposeHook<T: Table + Bind> {
    /// Transposition Stage
    fn apply(&self, #[allow(unused)] ctx: &Query<T>, #[allow(unused)] row: &mut T) -> Result<()> {
        Ok(())
    }
}

//pub struct HookMap<T: Table + Bind> {
//pub validation: Vec<Box<dyn ValidationHook<T>>>,
//pub preperation: Vec<Box<dyn PreparationHook<T>>>,
//pub inspection: Vec<Box<dyn InspectionHook<T>>>,
//pub transposition: Vec<Box<dyn TransposeHook<T>>>,
//}

//pub trait Hooks: Table + Bind {
//fn map() -> HookMap<Self>;

pub trait Hooks: Table + Bind {
    const VALIDATION: &'static [&'static dyn ValidationHook<Self>];
    const PREPARATION: &'static [&'static dyn PreparationHook<Self>];
    const INSPECTION: &'static [&'static dyn InspectionHook<Self>];
    const TRANSPOSITION: &'static [&'static dyn TransposeHook<Self>];

    fn validate(&self, ctx: &Query<Self>) -> Result<()> {
        for hook in Self::VALIDATION {
            hook.apply(ctx, &self)?;
        }

        Ok(())
    }

    fn prepare(&mut self, ctx: &Query<Self>) -> Result<()> {
        for hook in Self::PREPARATION {
            hook.apply(ctx, self)?;
        }

        Ok(())
    }

    fn inspect(ctx: &Query<Self>) {
        for hook in Self::INSPECTION {
            hook.apply(&ctx);
        }
    }

    fn transpose(&mut self, ctx: &Query<Self>) -> Result<()> {
        for hook in Self::TRANSPOSITION {
            hook.apply(&ctx, self)?;
        }

        Ok(())
    }
}
