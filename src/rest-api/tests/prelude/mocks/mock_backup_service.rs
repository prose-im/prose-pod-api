// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use service::{backups::backup_service::prelude::*, util::random_string_alphanumeric};

use super::prelude::*;

#[derive(Debug, Clone, Default)]
pub struct MockBackupService {
    pub state: Arc<RwLock<MockBackupServiceState>>,
}

#[derive(Debug, Default)]
pub struct MockBackupServiceState {
    pub backups: LinkedHashMap<BackupId, BackupMetadata>,
}

impl MockBackupService {
    #[allow(unused)]
    pub(crate) fn state(&self) -> RwLockReadGuard<'_, MockBackupServiceState> {
        self.state.read().unwrap()
    }

    #[allow(unused)]
    pub(crate) fn state_mut(&self) -> RwLockWriteGuard<'_, MockBackupServiceState> {
        self.state.write().unwrap()
    }
}

#[async_trait]
impl BackupServiceImpl for MockBackupService {
    async fn create(&self) -> Result<BackupMetadata, anyhow::Error> {
        let backup_id = random_string_alphanumeric(12);
        let backup = BackupMetadata {
            backup_id: backup_id.clone(),
        };
        self.state_mut().backups.insert(backup_id, backup.clone());
        Ok(backup)
    }

    async fn get(&self, backup_id: BackupId) -> Result<Option<BackupMetadata>, anyhow::Error> {
        Ok(self.state().backups.get(&backup_id).cloned())
    }

    async fn list(&self) -> Result<Vec<BackupMetadata>, anyhow::Error> {
        Ok(self.state().backups.keys().cloned().collect())
    }

    async fn delete(&self, backup_id: BackupId) -> Result<(), anyhow::Error> {
        self.state_mut().backups.remove(&backup_id);
        Ok(())
    }
}
