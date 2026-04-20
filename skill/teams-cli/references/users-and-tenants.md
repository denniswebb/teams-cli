# Users and Tenants

## User Commands

```sh
teams user me --output json                        # Current user profile
teams user get user@example.com --output json      # Lookup by email
teams user search "8:orgid:mri1,8:orgid:mri2"     # Search by MRI identifiers
```

MRI format: `8:orgid:{azure-ad-object-id}` (comma-separated for multiple).

## Tenant Commands

```sh
teams tenant list --output json      # List tenants you belong to
teams tenant domains --output json   # Verified domains for current tenant
```

## Check Current Identity

```sh
teams user me --output json | jq '{displayName: .data.displayName, email: .data.email}'
```
