use super::error::{TappError, TappResult};
use super::manifest::{Manifest, Permission};

pub struct PermissionChecker;

impl PermissionChecker {
    pub fn check(manifest: &Manifest, required: &Permission) -> TappResult<()> {
        if manifest.has_permission(required) {
            Ok(())
        } else {
            Err(TappError::PermissionDenied(format!(
                "App '{}' requires permission '{}' but it was not granted",
                manifest.id,
                required.as_str()
            )))
        }
    }

    pub fn check_any(manifest: &Manifest, required: &[Permission]) -> TappResult<()> {
        for perm in required {
            if manifest.has_permission(perm) {
                return Ok(());
            }
        }
        let perms: Vec<_> = required.iter().map(|p| p.as_str()).collect();
        Err(TappError::PermissionDenied(format!(
            "App '{}' requires one of permissions {:?} but none were granted",
            manifest.id, perms
        )))
    }

    pub fn check_all(manifest: &Manifest, required: &[Permission]) -> TappResult<()> {
        for perm in required {
            Self::check(manifest, perm)?;
        }
        Ok(())
    }

    pub fn can_read_fs(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::FsRead)
            || manifest.has_permission(&Permission::FsWrite)
            || manifest.has_permission(&Permission::FsSystem)
    }

    pub fn can_write_fs(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::FsWrite)
            || manifest.has_permission(&Permission::FsSystem)
    }

    pub fn can_use_network(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::NetworkFetch)
            || manifest.has_permission(&Permission::NetworkUnrestricted)
    }

    pub fn can_use_storage(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::StorageSession)
            || manifest.has_permission(&Permission::StoragePersistent)
    }

    pub fn can_use_persistent_storage(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::StoragePersistent)
    }

    pub fn can_interact_with_agent(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::AgentInject)
            || manifest.has_permission(&Permission::AgentTools)
            || manifest.has_permission(&Permission::AgentHooks)
            || manifest.has_permission(&Permission::AgentSpawn)
    }

    pub fn can_spawn_agent(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::AgentSpawn)
    }

    pub fn can_register_tools(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::AgentTools)
    }

    pub fn can_register_hooks(manifest: &Manifest) -> bool {
        manifest.has_permission(&Permission::AgentHooks)
    }
}
