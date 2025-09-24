use headers::Header;
use serde::Serialize;
use axum::http::{HeaderName, HeaderValue};

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

pub struct TenantIdHeader(pub String);
pub struct UidHeader(pub String);

impl Header for TenantIdHeader {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("tenant_id");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        if let Some(value) = values.next() {
            return value
                .to_str()
                .map(|s| TenantIdHeader(s.to_string()))
                .map_err(|_| headers::Error::invalid());
        }
        Err(headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_str(&self.0).unwrap()));
    }
}

impl Header for UidHeader {
    fn name() -> &'static HeaderName {
        static NAME: HeaderName = HeaderName::from_static("uid");
        &NAME
    }

    fn decode<'i, I>(values: &mut I) -> Result<Self, headers::Error>
    where
        I: Iterator<Item = &'i HeaderValue>,
    {
        if let Some(value) = values.next() {
            return value
                .to_str()
                .map(|s| UidHeader(s.to_string()))
                .map_err(|_| headers::Error::invalid());
        }
        Err(headers::Error::invalid())
    }

    fn encode<E: Extend<HeaderValue>>(&self, values: &mut E) {
        values.extend(std::iter::once(HeaderValue::from_str(&self.0).unwrap()));
    }
}