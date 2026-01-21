use nix::unistd::Uid;

use crate::errors::{AppError, Result};

/// Check that we're running as root
pub fn require_root() -> Result<()> {
    if !Uid::effective().is_root() {
        Err(AppError::RootPrivilegesRequired(
            env!("CARGO_PKG_NAME").into(),
        ))
    } else {
        Ok(())
    }
}
