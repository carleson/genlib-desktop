//! Tjänster för Genlib Desktop
//!
//! Innehåller affärslogik som inte hör hemma i UI eller databas.

pub mod backup;
pub mod document_sync;
pub mod export;
pub mod family_tree;
pub mod restore;

pub use backup::{BackupInfo, BackupService};
pub use document_sync::DocumentSyncService;
pub use family_tree::{FamilyTree, FamilyTreeService, LinkType};
pub use restore::{RestorePreview, RestoreService};
