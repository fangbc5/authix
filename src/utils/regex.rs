use once_cell::sync::Lazy;
use regex::Regex;

static E164: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\+[1-9][0-9]{7,14}$").unwrap());
static CN: Lazy<Regex> = Lazy::new(|| Regex::new(r"^\+86(1[3-9][0-9]{9})$").unwrap());
static USERNAME_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_\-\.@#$%^&*]{6,32}$").unwrap());
static PASSWORD_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[A-Za-z0-9_\-\.@#$%^&*]{8,32}$").unwrap());
static EMAIL_REGEX: Lazy<Regex> = Lazy::new(|| Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap());

pub fn is_valid_username(username: &str) -> bool {
    // 允许大小写字母、数字和常见符号 _-.@#$%^&*
    // 长度 1..=64（可按需调整）
    USERNAME_REGEX.is_match(username)
}

pub fn is_valid_password(password: &str) -> bool {
    // 8-32 长度，允许大小写字母、数字和常见符号 _-.@#$%^&*
    PASSWORD_REGEX.is_match(password)
}

pub fn is_valid_phone(phone: &str) -> bool {
    let s = phone.trim();

    // 基本格式必须是 E.164
    if !E164.is_match(s) {
        return false;
    }

    // 如果是中国号码，必须严格匹配中国规则（11位）
    if s.starts_with("+86") {
        return CN.is_match(s);
    }

    // 非中国号码：如果你允许其它国家只要符合 E.164 就通过，返回 true；
    // 如果需要限制到白名单国家，改为检查前缀并用相应正则验证或返回 false。
    true
}

pub fn is_valid_email(email: &str) -> bool {
    // 简单邮箱校验
    EMAIL_REGEX.is_match(email)
}