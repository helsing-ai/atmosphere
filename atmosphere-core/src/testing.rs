use crate::Entity;

/// Automate integration tests for database entities
pub trait Testable: Entity + Eq + Ord + Clone {
    fn mock() -> Self;
    fn patch(entity: Self) -> Self;
}

// TODO: provide helpers to autogenerate uuids, pks, strings, emails, etc – maybe reexport another
// crate?
