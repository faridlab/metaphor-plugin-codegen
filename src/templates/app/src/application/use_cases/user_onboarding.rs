//! User Onboarding Use Case
//!
//! Handles the complete user onboarding process across multiple modules:
//! - Creates user account (Sapiens module)
//! - Sends welcome email (Postman module)
//! - Creates user folders (Bucket module)
//! - Sets up default permissions

use crate::shared::error::{AppError, AppResult};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// User onboarding request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingRequest {
    pub email: String,
    pub password: String,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub roles: Vec<String>,
    pub send_welcome_email: bool,
    pub create_profile_folders: bool,
}

/// User onboarding response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserOnboardingResponse {
    pub user_id: Uuid,
    pub email: String,
    pub username: String,
    pub welcome_email_sent: bool,
    pub profile_folders_created: bool,
    pub default_permissions_set: bool,
    pub activation_required: bool,
    pub onboarding_steps: Vec<OnboardingStep>,
}

/// Onboarding step result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OnboardingStep {
    pub step: String,
    pub status: StepStatus,
    pub message: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StepStatus {
    Success,
    Failed,
    Skipped,
}

/// User onboarding use case
pub struct UserOnboardingUseCase {
    // Dependencies would be injected here:
    // - user_service: Arc<dyn sapiens::domain::UserService>,
    // - email_service: Arc<dyn postman::domain::EmailService>,
    // - file_service: Arc<dyn bucket::domain::FileService>,
    // - permission_service: Arc<crate::domain::services::PermissionService>,
    // - audit_service: Arc<crate::domain::services::AuditService>,
}

impl UserOnboardingUseCase {
    pub fn new() -> Self {
        Self {}
    }

    /// Execute the complete user onboarding process
    pub async fn execute(&self, request: UserOnboardingRequest) -> AppResult<UserOnboardingResponse> {
        let mut steps = Vec::new();
        let mut response = UserOnboardingResponse {
            user_id: Uuid::new_v4(), // Will be set after user creation
            email: request.email.clone(),
            username: request.username.clone(),
            welcome_email_sent: false,
            profile_folders_created: false,
            default_permissions_set: false,
            activation_required: true,
            onboarding_steps: steps,
        };

        // Step 1: Create user account
        let user_creation_result = self.create_user_account(&request).await;
        match user_creation_result {
            Ok(user_id) => {
                response.user_id = user_id;
                steps.push(OnboardingStep {
                    step: "Create User Account".to_string(),
                    status: StepStatus::Success,
                    message: "User account created successfully".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }
            Err(e) => {
                steps.push(OnboardingStep {
                    step: "Create User Account".to_string(),
                    status: StepStatus::Failed,
                    message: format!("Failed to create user account: {}", e),
                    timestamp: chrono::Utc::now(),
                });
                return Err(e);
            }
        }

        // Step 2: Set default permissions
        let permissions_result = self.set_default_permissions(&response.user_id, &request.roles).await;
        match permissions_result {
            Ok(_) => {
                response.default_permissions_set = true;
                steps.push(OnboardingStep {
                    step: "Set Default Permissions".to_string(),
                    status: StepStatus::Success,
                    message: "Default permissions assigned".to_string(),
                    timestamp: chrono::Utc::now(),
                });
            }
            Err(e) => {
                steps.push(OnboardingStep {
                    step: "Set Default Permissions".to_string(),
                    status: StepStatus::Failed,
                    message: format!("Failed to set permissions: {}", e),
                    timestamp: chrono::Utc::now(),
                });
                // Don't fail the entire process for permission issues
            }
        }

        // Step 3: Send welcome email (if requested)
        if request.send_welcome_email {
            let email_result = self.send_welcome_email(&response).await;
            match email_result {
                Ok(_) => {
                    response.welcome_email_sent = true;
                    steps.push(OnboardingStep {
                        step: "Send Welcome Email".to_string(),
                        status: StepStatus::Success,
                        message: "Welcome email sent successfully".to_string(),
                        timestamp: chrono::Utc::now(),
                    });
                }
                Err(e) => {
                    steps.push(OnboardingStep {
                        step: "Send Welcome Email".to_string(),
                        status: StepStatus::Failed,
                        message: format!("Failed to send welcome email: {}", e),
                        timestamp: chrono::Utc::now(),
                    });
                    // Don't fail the entire process for email issues
                }
            }
        } else {
            steps.push(OnboardingStep {
                step: "Send Welcome Email".to_string(),
                status: StepStatus::Skipped,
                message: "Welcome email disabled".to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        // Step 4: Create profile folders (if requested)
        if request.create_profile_folders {
            let folders_result = self.create_profile_folders(&response.user_id).await;
            match folders_result {
                Ok(_) => {
                    response.profile_folders_created = true;
                    steps.push(OnboardingStep {
                        step: "Create Profile Folders".to_string(),
                        status: StepStatus::Success,
                        message: "Profile folders created successfully".to_string(),
                        timestamp: chrono::Utc::now(),
                    });
                }
                Err(e) => {
                    steps.push(OnboardingStep {
                        step: "Create Profile Folders".to_string(),
                        status: StepStatus::Failed,
                        message: format!("Failed to create profile folders: {}", e),
                        timestamp: chrono::Utc::now(),
                    });
                    // Don't fail the entire process for folder creation issues
                }
            }
        } else {
            steps.push(OnboardingStep {
                step: "Create Profile Folders".to_string(),
                status: StepStatus::Skipped,
                message: "Profile folder creation disabled".to_string(),
                timestamp: chrono::Utc::now(),
            });
        }

        // Update response with all steps
        response.onboarding_steps = steps;

        Ok(response)
    }

    /// Create user account using Sapiens module
    async fn create_user_account(&self, request: &UserOnboardingRequest) -> AppResult<Uuid> {
        // This would integrate with the Sapiens module
        // For now, simulating user creation
        let user_id = Uuid::new_v4();

        // In a real implementation:
        // let create_user_command = sapiens::application::commands::CreateUserCommand {
        //     email: request.email.clone(),
        //     password: request.password.clone(),
        //     username: request.username.clone(),
        //     first_name: request.first_name.clone(),
        //     last_name: request.last_name.clone(),
        // };
        //
        // let user = self.user_service.create_user(create_user_command).await?;

        Ok(user_id)
    }

    /// Set default permissions for the new user
    async fn set_default_permissions(&self, user_id: &Uuid, roles: &[String]) -> AppResult<()> {
        // This would integrate with the permission service
        // For now, simulating permission setting

        // In a real implementation:
        // for role_name in roles {
        //     self.permission_service.assign_role_to_user(user_id, role_name).await?;
        // }

        Ok(())
    }

    /// Send welcome email using Postman module
    async fn send_welcome_email(&self, response: &UserOnboardingResponse) -> AppResult<()> {
        // This would integrate with the Postman module
        // For now, simulating email sending

        // In a real implementation:
        // let email_data = postman::domain::WelcomeEmailData {
        //     recipient_email: response.email.clone(),
        //     recipient_name: format!("{} {}", response.first_name, response.last_name),
        //     user_id: response.user_id,
        //     activation_token: self.generate_activation_token(&response.user_id),
        // };
        //
        // self.email_service.send_welcome_email(email_data).await?;

        Ok(())
    }

    /// Create user profile folders using Bucket module
    async fn create_profile_folders(&self, user_id: &Uuid) -> AppResult<()> {
        // This would integrate with the Bucket module
        // For now, simulating folder creation

        // In a real implementation:
        // let folders = vec![
        //     ("Documents", "User document storage"),
        //     ("Pictures", "User picture storage"),
        //     ("Profile", "User profile files"),
        // ];
        //
        // for (folder_name, description) in folders {
        //     let folder_request = bucket::application::commands::CreateFolderCommand {
        //         name: folder_name.to_string(),
        //         description: description.to_string(),
        //         parent_id: None,
        //         owner_id: *user_id,
        //         is_private: true,
        //     };
        //
        //     self.file_service.create_folder(folder_request).await?;
        // }

        Ok(())
    }

    /// Rollback onboarding in case of failure
    pub async fn rollback(&self, user_id: &Uuid, completed_steps: &[OnboardingStep]) -> AppResult<()> {
        // This would undo any completed steps in reverse order
        // In a real implementation, this would:
        // 1. Delete created folders
        // 2. Remove assigned permissions
        // 3. Delete user account
        // 4. Log the rollback attempt

        for step in completed_steps.iter().rev() {
            match step.step.as_str() {
                "Create Profile Folders" => {
                    // Delete created folders
                    self.delete_user_folders(user_id).await?;
                }
                "Set Default Permissions" => {
                    // Remove assigned permissions
                    self.remove_user_permissions(user_id).await?;
                }
                "Send Welcome Email" => {
                    // Can't rollback email, just log it
                    tracing::warn!("Cannot rollback welcome email for user {}", user_id);
                }
                "Create User Account" => {
                    // Delete user account
                    self.delete_user_account(user_id).await?;
                }
                _ => {}
            }
        }

        Ok(())
    }

    async fn delete_user_folders(&self, user_id: &Uuid) -> AppResult<()> {
        // Implementation for deleting user folders
        Ok(())
    }

    async fn remove_user_permissions(&self, user_id: &Uuid) -> AppResult<()> {
        // Implementation for removing user permissions
        Ok(())
    }

    async fn delete_user_account(&self, user_id: &Uuid) -> AppResult<()> {
        // Implementation for deleting user account
        Ok(())
    }
}

impl Default for UserOnboardingUseCase {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_onboarding_request() {
        let request = UserOnboardingRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            roles: vec!["user".to_string()],
            send_welcome_email: true,
            create_profile_folders: true,
        };

        assert_eq!(request.email, "test@example.com");
        assert!(request.send_welcome_email);
        assert_eq!(request.roles.len(), 1);
    }

    #[test]
    fn test_onboarding_step_creation() {
        let step = OnboardingStep {
            step: "Test Step".to_string(),
            status: StepStatus::Success,
            message: "Test completed".to_string(),
            timestamp: chrono::Utc::now(),
        };

        assert_eq!(step.step, "Test Step");
        assert!(matches!(step.status, StepStatus::Success));
    }

    #[tokio::test]
    async fn test_user_onboarding_use_case() {
        let use_case = UserOnboardingUseCase::new();
        let request = UserOnboardingRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            roles: vec!["user".to_string()],
            send_welcome_email: false, // Disable for testing
            create_profile_folders: false, // Disable for testing
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert_eq!(response.email, "test@example.com");
        assert_eq!(response.username, "testuser");
        assert!(response.default_permissions_set);
        assert!(!response.welcome_email_sent);
        assert!(!response.profile_folders_created);
        assert_eq!(response.onboarding_steps.len(), 4); // Created account, set permissions, 2 skipped
    }

    #[tokio::test]
    async fn test_user_onboarding_with_optional_steps() {
        let use_case = UserOnboardingUseCase::new();
        let request = UserOnboardingRequest {
            email: "test@example.com".to_string(),
            password: "password123".to_string(),
            username: "testuser".to_string(),
            first_name: "Test".to_string(),
            last_name: "User".to_string(),
            roles: vec!["user".to_string()],
            send_welcome_email: true,
            create_profile_folders: true,
        };

        let result = use_case.execute(request).await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.welcome_email_sent); // Should be true even if mocked
        assert!(response.profile_folders_created); // Should be true even if mocked
        assert_eq!(response.onboarding_steps.len(), 4);
    }
}