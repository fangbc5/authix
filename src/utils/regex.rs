use regex::Regex;

pub fn is_valid_username(username: &str) -> bool {
    // 允许大小写字母、数字和常见符号 _-.@#$%^&*
    // 长度 1..=64（可按需调整）
    let re = Regex::new(r"^[A-Za-z0-9_\-\.@#$%^&*]{1,64}$").unwrap();
    re.is_match(username)
}

pub fn is_valid_password(password: &str) -> bool {
    // 8-32 长度，允许大小写字母、数字和常见符号 _-.@#$%^&*
    let re = Regex::new(r"^[A-Za-z0-9_\-\.@#$%^&*]{8,32}$").unwrap();
    re.is_match(password)
}

pub fn is_valid_phone(phone: &str) -> bool {
    // 简单手机号校验：以数字开头，允许 +，总长 6-20
    // 可根据地区规范替换为更严格的正则
    let re = Regex::new(r"^(\+)?[0-9]{6,20}$").unwrap();
    re.is_match(phone)
}

pub fn is_valid_email(email: &str) -> bool {
    // 简单邮箱校验
    let re = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
    re.is_match(email)
}