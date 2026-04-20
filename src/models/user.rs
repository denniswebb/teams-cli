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
