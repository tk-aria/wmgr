/// SCM (Source Control Management) operations infrastructure
/// 
/// This module provides a unified interface for different SCM systems
/// including Git, SVN, and Perforce (P4).

pub mod scm_interface;
pub mod git_scm;
pub mod svn_scm;
pub mod p4_scm;
pub mod scm_factory;

pub use scm_interface::{ScmOperations, ScmError, CloneOptions, SyncOptions, StatusResult};
pub use scm_factory::ScmFactory;