pub mod schema;
pub mod sqlite;

use crate::error::Result;

/// Trait for database operations
/// Note: Storage implementations must be Send but not necessarily Sync
/// since rusqlite::Connection is Send but not Sync
pub trait Storage: Send {
    /// Get a reference to the underlying connection for raw queries
    fn connection(&self) -> &rusqlite::Connection;

    /// Run migrations
    fn migrate(&self) -> Result<()>;
}
