use crate::model::{
    id::UserId,
    role::Role,
    value::{UserEmail, UserName},
};

#[derive(Debug)]
pub struct CreateUser {
    pub name: UserName,
    pub email: UserEmail,
    pub password: String,
}

#[derive(Debug)]
pub struct UpdateUserRole {
    pub user_id: UserId,
    pub role: Role,
}

#[derive(Debug)]
pub struct UpdateUserPassword {
    pub user_id: UserId,
    pub current_password: String,
    pub new_password: String,
}

#[derive(Debug)]
pub struct DeleteUser {
    pub user_id: UserId,
}
