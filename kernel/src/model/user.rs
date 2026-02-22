use crate::model::{
    id::UserId,
    role::Role,
    value::{UserEmail, UserName},
};
use shared::error::AppError;

pub mod event;

#[derive(Debug, PartialEq, Eq)]
pub struct User {
    id: UserId,
    name: UserName,
    email: UserEmail,
    role: Role,
}

impl User {
    pub fn new(id: UserId, name: UserName, email: UserEmail, role: Role) -> Self {
        Self {
            id,
            name,
            email,
            role,
        }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn email(&self) -> &UserEmail {
        &self.email
    }

    pub fn role(&self) -> &Role {
        &self.role
    }

    pub fn into_parts(self) -> (UserId, UserName, UserEmail, Role) {
        (self.id, self.name, self.email, self.role)
    }
}

impl TryFrom<event::CreateUser> for User {
    type Error = AppError;

    fn try_from(value: event::CreateUser) -> Result<Self, Self::Error> {
        Ok(Self::new(
            UserId::new(),
            value.name,
            value.email,
            Role::User,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BookOwner {
    id: UserId,
    name: UserName,
}

impl BookOwner {
    pub fn new(id: UserId, name: UserName) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn into_parts(self) -> (UserId, UserName) {
        (self.id, self.name)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CheckoutUser {
    id: UserId,
    name: UserName,
}

impl CheckoutUser {
    pub fn new(id: UserId, name: UserName) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> UserId {
        self.id
    }

    pub fn name(&self) -> &UserName {
        &self.name
    }

    pub fn into_parts(self) -> (UserId, UserName) {
        (self.id, self.name)
    }
}
