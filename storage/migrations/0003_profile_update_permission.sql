-- v0.1.1 additive permission for self-service profile update

INSERT INTO permission_definitions (permission_code, permission_name)
VALUES ('user.profile.update', 'Update current user profile')
ON CONFLICT (permission_code) DO NOTHING;

INSERT INTO role_permissions (id, role_code, permission_code)
VALUES
    (gen_random_uuid(), 'workspace_admin', 'user.profile.update'),
    (gen_random_uuid(), 'workspace_member', 'user.profile.update')
ON CONFLICT (role_code, permission_code) DO NOTHING;
