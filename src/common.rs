use serde::Serialize;

#[derive(Debug, Serialize)]
pub struct R<T> {
    pub success: bool,
    pub code: i32,
    pub message: Option<String>,
    pub data: Option<T>
}

impl<T> R<T> {
    pub fn ok() -> Self {
        Self {
            success: true,
            code: 200,
            message: Some("ok".to_owned()),
            data: None
        }
    }

    pub fn ok_data(data: T) -> Self {
        Self {
            success: true,
            code: 200,
            message: Some("ok".to_owned()),
            data: Some(data)
        }
    }
    
    #[allow(dead_code)]
    pub fn ok_message(msg: String) -> Self {
        Self {
            success: true,
            code: 200,
            message: Some(msg),
            data: None
        }
    }

    #[allow(dead_code)]
    pub fn ok_data_message(data: T, msg: String) -> Self {
        Self {
            success: true,
            code: 200,
            message: Some(msg),
            data: Some(data)
        }
    }

    pub fn error(code: i32, msg: String) -> Self {
        Self {
            success: false,
            code,
            message: Some(msg),
            data: None
        }
    }
}