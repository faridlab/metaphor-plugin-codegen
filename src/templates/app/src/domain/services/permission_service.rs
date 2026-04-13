//! Permission Service
//!
//! Handles permission checking and role-based access control across modules

use crate::domain::value_objects::{Permission, Role};
use std::collections::HashSet;
use uuid::Uuid;

/// Domain service for managing permissions and roles
pub struct PermissionService {
    // Dependencies would be injected here
}

impl PermissionService {
    pub fn new() -> Self {
        Self {}
    }

    /// Check if a user has a specific permission
    pub fn user_has_permission(
        &self,
        user_id: &Uuid,
        permission: &Permission,
        user_roles: &[Role],
    ) -> bool {
        // System roles have broader permissions
        if self.has_system_permission(user_roles, permission) {
            return true;
        }

        // Check module-specific permissions
        self.has_module_permission(user_roles, permission)
    }

    /// Check if user has permission based on their roles
    pub fn user_has_permission_for_resource(
        &self,
        user_id: &Uuid,
        action: &str,
        resource: &str,
        resource_id: Option<String>,
        user_roles: &[Role],
    ) -> bool {
        // Check global permission
        let global_permission = Permission::global(action.to_string(), resource.to_string());
        if self.user_has_permission(user_id, &global_permission, user_roles) {
            return true;
        }

        // Check resource-specific permission
        if let Some(res_id) = resource_id {
            let resource_permission =
                Permission::resource(action.to_string(), resource.to_string(), res_id);
            if self.user_has_permission(user_id, &resource_permission, user_roles) {
                return true;
            }
        }

        false
    }

    /// Get all permissions for a user based on their roles
    pub fn get_user_permissions(&self, user_roles: &[Role]) -> Vec<Permission> {
        let mut permissions = HashSet::new();

        for role in user_roles {
            let role_permissions = self.get_role_permissions(role);
            for permission in role_permissions {
                permissions.insert(permission);
            }
        }

        permissions.into_iter().collect()
    }

    /// Check if roles contain system-level permission
    fn has_system_permission(&self, roles: &[Role], permission: &Permission) -> bool {
        roles.iter().any(|role| {
            role.is_system_role() && self.role_has_permission(role, permission)
        })
    }

    /// Check if roles contain module-level permission
    fn has_module_permission(&self, roles: &[Role], permission: &Permission) -> bool {
        roles.iter().any(|role| self.role_has_permission(role, permission))
    }

    /// Check if a specific role has a permission
    fn role_has_permission(&self, role: &Role, permission: &Permission) -> bool {
        // In a real implementation, this would query role permissions
        // For now, using a simple heuristic based on role level
        match permission.action.as_str() {
            "read" | "view" => role.level >= 1,
            "create" | "write" => role.level >= 2,
            "update" | "edit" => role.level >= 3,
            "delete" | "remove" => role.level >= 4,
            "admin" | "manage" => role.level >= 5,
            _ => false,
        }
    }

    /// Get permissions associated with a role
    fn get_role_permissions(&self, role: &Role) -> Vec<Permission> {
        let mut permissions = Vec::new();

        // Basic permissions based on role level
        if role.level >= 1 {
            permissions.push(Permission::global("read".to_string(), "all".to_string()));
        }
        if role.level >= 2 {
            permissions.push(Permission::global("create".to_string(), "all".to_string()));
        }
        if role.level >= 3 {
            permissions.push(Permission::global("update".to_string(), "all".to_string()));
        }
        if role.level >= 4 {
            permissions.push(Permission::global("delete".to_string(), "all".to_string()));
        }
        if role.level >= 5 {
            permissions.push(Permission::global("admin".to_string(), "all".to_string()));
        }

        // Module-specific permissions
        if let Some(ref module) = role.module {
            permissions.push(Permission::module(
                "manage".to_string(),
                module.clone(),
                module.clone(),
            ));
        }

        permissions
    }

    /// Check if one role has higher privilege than another
    pub fn has_higher_privilege(&self, role_a: &Role, role_b: &Role) -> bool {
        // System roles have higher privilege than module roles
        match (role_a.is_system_role(), role_b.is_system_role()) {
            (true, false) => true,
            (false, true) => false,
            _ => role_a.has_higher_privilege_than(role_b),
        }
    }

    /// Get the highest privilege role for a user
    pub fn get_highest_privilege_role(&self, roles: &[Role]) -> Option<&Role> {
        roles.iter().max_by_key(|role| role.level)
    }

    /// Validate if a user can perform an action on another user
    pub fn can_manage_user(&self, actor_roles: &[Role], target_roles: &[Role]) -> bool {
        // Can manage users if you have admin permissions
        let has_admin = actor_roles.iter().any(|role| role.level >= 5);
        if has_admin {
            return true;
        }

        // Can manage users with lower privilege level
        let actor_max_level = actor_roles.iter().map(|r| r.level).max().unwrap_or(0);
        let target_max_level = target_roles.iter().map(|r| r.level).max().unwrap_or(0);

        actor_max_level > target_max_level
    }
}

impl Default for PermissionService {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::value_objects::PermissionScope;

    #[test]
    fn test_role_creation() {
        let role = Role::new("user".to_string(), 1);
        assert_eq!(role.name, "user");
        assert_eq!(role.level, 1);
        assert!(role.is_system_role());
    }

    #[test]
    fn test_module_role() {
        let role = Role::module_role("editor".to_string(), 2, "content".to_string());
        assert_eq!(role.name, "editor");
        assert_eq!(role.level, 2);
        assert!(role.is_module_role("content"));
        assert!(!role.is_module_role("users"));
    }

    #[test]
    fn test_permission_creation() {
        let global_perm = Permission::global("read".to_string(), "users".to_string());
        let module_perm = Permission::module("manage".to_string(), "content".to_string(), "cms".to_string());
        let resource_perm = Permission::resource("delete".to_string(), "post".to_string(), "123".to_string());

        assert!(matches!(global_perm.scope, PermissionScope::Global));
        assert!(matches!(module_perm.scope, PermissionScope::Module(_)));
        assert!(matches!(resource_perm.scope, PermissionScope::Resource(_)));
    }

    #[test]
    fn test_permission_service() {
        let service = PermissionService::new();
        let admin_role = Role::new("admin".to_string(), 5);
        let user_role = Role::new("user".to_string(), 1);
        let roles = vec![admin_role, user_role];

        let permissions = service.get_user_permissions(&roles);
        assert!(!permissions.is_empty());

        let read_permission = Permission::global("read".to_string(), "users".to_string());
        assert!(service.user_has_permission(&Uuid::new_v4(), &read_permission, &roles));
    }

    #[test]
    fn test_role_privilege_comparison() {
        let service = PermissionService::new();
        let admin_role = Role::new("admin".to_string(), 5);
        let user_role = Role::new("user".to_string(), 1);

        assert!(service.has_higher_privilege(&admin_role, &user_role));
        assert!(!service.has_higher_privilege(&user_role, &admin_role));
    }

    #[test]
    fn test_user_management_permissions() {
        let service = PermissionService::new();
        let admin_roles = vec![Role::new("admin".to_string(), 5)];
        let user_roles = vec![Role::new("user".to_string(), 1)];

        assert!(service.can_manage_user(&admin_roles, &user_roles));
        assert!(!service.can_manage_user(&user_roles, &admin_roles));
    }
}