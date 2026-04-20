use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {
    #[serde(default, alias = "displayName")]
    pub display_name: String,
    #[serde(default)]
    pub email: String,
    #[serde(default)]
    pub mri: String,
    #[serde(default, alias = "objectId")]
    pub object_id: String,
    #[serde(default, alias = "objectType")]
    pub object_type: String,
    #[serde(default, alias = "jobTitle")]
    pub job_title: String,
    #[serde(default)]
    pub department: String,
    #[serde(default)]
    pub mobile: String,
    #[serde(default, alias = "isShortProfile")]
    pub is_short_profile: bool,
    #[serde(default, alias = "givenName")]
    pub given_name: String,
    #[serde(default)]
    pub surname: String,
    #[serde(default, alias = "userPrincipalName")]
    pub user_principal_name: String,
    #[serde(default, alias = "telephoneNumber")]
    pub telephone_number: String,
    #[serde(default, alias = "companyName")]
    pub company_name: String,
    #[serde(default, alias = "userType")]
    pub user_type: String,
    #[serde(default, alias = "tenantName")]
    pub tenant_name: String,
    #[serde(default, alias = "userLocation")]
    pub user_location: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserResponse {
    #[serde(default)]
    pub value: Option<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UsersResponse {
    #[serde(default)]
    pub value: Vec<User>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Tenant {
    #[serde(default)]
    pub tenant_id: String,
    #[serde(default)]
    pub tenant_name: String,
    #[serde(default)]
    pub user_id: String,
    #[serde(default)]
    pub is_signed_in_tenant: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VerifiedDomain {
    #[serde(default)]
    pub name: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_deserialization_full() {
        let json = r#"{
            "displayName": "John Doe",
            "email": "john@example.com",
            "mri": "8:orgid:abc-123",
            "objectId": "obj-456",
            "objectType": "user",
            "jobTitle": "Engineer",
            "department": "Engineering",
            "mobile": "+1234567890",
            "isShortProfile": false,
            "givenName": "John",
            "surname": "Doe",
            "userPrincipalName": "john@example.com",
            "telephoneNumber": "+0987654321",
            "companyName": "Contoso",
            "userType": "Member",
            "tenantName": "Contoso Inc",
            "userLocation": "US"
        }"#;

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.display_name, "John Doe");
        assert_eq!(user.email, "john@example.com");
        assert_eq!(user.mri, "8:orgid:abc-123");
        assert_eq!(user.object_id, "obj-456");
        assert_eq!(user.job_title, "Engineer");
        assert_eq!(user.department, "Engineering");
        assert_eq!(user.given_name, "John");
        assert_eq!(user.surname, "Doe");
        assert_eq!(user.company_name, "Contoso");
        assert_eq!(user.user_type, "Member");
        assert!(!user.is_short_profile);
    }

    #[test]
    fn user_deserialization_missing_optional_fields() {
        let json = r#"{}"#;

        let user: User = serde_json::from_str(json).unwrap();
        assert_eq!(user.display_name, "");
        assert_eq!(user.email, "");
        assert_eq!(user.mri, "");
        assert_eq!(user.job_title, "");
        assert!(!user.is_short_profile);
    }

    #[test]
    fn user_response_with_value() {
        let json = r#"{
            "value": {
                "displayName": "Alice",
                "email": "alice@example.com"
            }
        }"#;

        let resp: UserResponse = serde_json::from_str(json).unwrap();
        let user = resp.value.unwrap();
        assert_eq!(user.display_name, "Alice");
        assert_eq!(user.email, "alice@example.com");
    }

    #[test]
    fn user_response_null_value() {
        let json = r#"{"value": null}"#;

        let resp: UserResponse = serde_json::from_str(json).unwrap();
        assert!(resp.value.is_none());
    }

    #[test]
    fn user_response_missing_value() {
        let json = r#"{}"#;

        let resp: UserResponse = serde_json::from_str(json).unwrap();
        assert!(resp.value.is_none());
    }

    #[test]
    fn users_response_deserialization() {
        let json = r#"{
            "value": [
                {"displayName": "User A", "mri": "8:orgid:aaa"},
                {"displayName": "User B", "mri": "8:orgid:bbb"}
            ]
        }"#;

        let resp: UsersResponse = serde_json::from_str(json).unwrap();
        assert_eq!(resp.value.len(), 2);
        assert_eq!(resp.value[0].display_name, "User A");
        assert_eq!(resp.value[1].mri, "8:orgid:bbb");
    }

    #[test]
    fn users_response_empty() {
        let json = r#"{}"#;

        let resp: UsersResponse = serde_json::from_str(json).unwrap();
        assert!(resp.value.is_empty());
    }

    #[test]
    fn tenant_deserialization() {
        let json = r#"{
            "tenantId": "tid-123",
            "tenantName": "Contoso",
            "userId": "uid-456",
            "isSignedInTenant": true
        }"#;

        let tenant: Tenant = serde_json::from_str(json).unwrap();
        assert_eq!(tenant.tenant_id, "tid-123");
        assert_eq!(tenant.tenant_name, "Contoso");
        assert_eq!(tenant.user_id, "uid-456");
        assert!(tenant.is_signed_in_tenant);
    }

    #[test]
    fn tenant_defaults() {
        let json = r#"{}"#;

        let tenant: Tenant = serde_json::from_str(json).unwrap();
        assert_eq!(tenant.tenant_id, "");
        assert!(!tenant.is_signed_in_tenant);
    }

    #[test]
    fn verified_domain_deserialization() {
        let json = r#"{"name": "contoso.com"}"#;

        let domain: VerifiedDomain = serde_json::from_str(json).unwrap();
        assert_eq!(domain.name, "contoso.com");
    }
}
