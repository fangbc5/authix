use axum::{async_trait, response::IntoResponse};
use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub phone: String,
    pub email: String,
    pub password: String,
}

#[async_trait]
pub trait UserProvider: Send + Sync {
    #[allow(dead_code)]
    async fn get_user_by_id(&self, id: i32) -> Result<User, String>;
    #[allow(dead_code)]
    async fn create_user(&self, user: User) -> Result<User, String>;
    #[allow(dead_code)]
    async fn delete_user(&self, id: i32) -> Result<(), String>;
    #[allow(dead_code)]
    async fn get_user_by_username(&self, username: String) -> Result<User, String>;
    #[allow(dead_code)]
    async fn get_user_by_phone(&self, phone: String) -> Result<User, String>;
    #[allow(dead_code)]
    async fn get_user_by_email(&self, email: String) -> Result<User, String>;
}

#[derive(Default)]
pub struct UserService;

impl UserProvider for UserService {
    #[allow(dead_code)]
    fn get_user_by_id<'life0,'async_trait>(&'life0 self,id:i32) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[allow(dead_code)]
    fn create_user<'life0,'async_trait>(&'life0 self,user:User) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[allow(dead_code)]
    fn delete_user<'life0,'async_trait>(&'life0 self,id:i32) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<(),String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[allow(dead_code)]
    fn get_user_by_username<'life0,'async_trait>(&'life0 self,username:String) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[allow(dead_code)]
    fn get_user_by_phone<'life0,'async_trait>(&'life0 self,phone:String) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[allow(dead_code)]
    fn get_user_by_email<'life0,'async_trait>(&'life0 self,email:String) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }
}

pub async fn user_profile() -> impl IntoResponse {
    "profile"
}