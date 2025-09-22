use axum::async_trait;
use serde::Deserialize;


#[derive(Debug, Clone, Deserialize)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[async_trait]
pub trait UserProvider: Send + Sync {
    async fn get_user_by_id(&self, id: i32) -> Result<User, String>;
    async fn create_user(&self, user: User) -> Result<User, String>;
    async fn delete_user(&self, id: i32) -> Result<(), String>;
    async fn get_user_by_username(&self, username: String) -> Result<User, String>;
    async fn get_user_by_phone(&self, phone: String) -> Result<User, String>;
    async fn get_user_by_email(&self, email: String) -> Result<User, String>;
}

pub struct UserService;

impl UserProvider for UserService {
    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn get_user_by_id<'life0,'async_trait>(&'life0 self,id:i32) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn create_user<'life0,'async_trait>(&'life0 self,user:User) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn delete_user<'life0,'async_trait>(&'life0 self,id:i32) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<(),String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn get_user_by_username<'life0,'async_trait>(&'life0 self,username:String) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn get_user_by_phone<'life0,'async_trait>(&'life0 self,phone:String) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }

    #[must_use]
    #[allow(elided_named_lifetimes,clippy::type_complexity,clippy::type_repetition_in_bounds)]
    fn get_user_by_email<'life0,'async_trait>(&'life0 self,email:String) ->  ::core::pin::Pin<Box<dyn ::core::future::Future<Output = Result<User,String> > + ::core::marker::Send+'async_trait> >where 'life0:'async_trait,Self:'async_trait {
        todo!()
    }
}