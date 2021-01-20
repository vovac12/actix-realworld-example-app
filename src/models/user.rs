use chrono::NaiveDateTime;
use uuid::Uuid;

use crate::schema::users;

#[derive(Debug, Queryable, Identifiable)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
    pub image: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Insertable)]
#[table_name = "users"]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password: String,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[derive(Debug, AsChangeset)]
#[table_name = "users"]
pub struct UserChange {
    pub username: Option<String>,
    pub email: Option<String>,
    pub password: Option<String>,
    pub bio: Option<String>,
    pub image: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    impl Default for User {
        fn default() -> Self {
            User {
                username: "username".to_string(),
                password: "password".to_string(),
                email: "email@e.mail".to_string(),
                id: Uuid::from_slice(&[1; 16]).unwrap(),
                bio: None,
                image: None,
                created_at: NaiveDateTime::from_timestamp(12, 12),
                updated_at: NaiveDateTime::from_timestamp(12, 12),
            }
        }
    }
    impl Default for NewUser {
        fn default() -> Self {
            NewUser {
                username: "username".to_string(),
                password: "password".to_string(),
                email: "email@e.mail".to_string(),
                bio: None,
                image: None,
            }
        }
    }

    impl Default for UserChange {
        fn default() -> Self {
            UserChange {
                username: Some("username".to_string()),
                password: Some("password".to_string()),
                email: Some("email@e.mail".to_string()),
                bio: None,
                image: None,
            }
        }
    }
}
