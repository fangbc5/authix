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
    // 更严格校验：
    // - 支持 E.164 一般格式: +[1-9][0-9]{7,14}
    // - 常用地区示例：
    //   CN: +86 1[3-9][0-9]{9}
    //   US/CA: +1 [2-9][0-9]{9}
    //   UK: +44 7[0-9]{9}
    //   DE: +49(1|15|16|17)[0-9]{8,9}
    //   FR: +33[67][0-9]{8}
    //   SA: +966 5[0-9]{8}
    // 允许无空格的纯数字国际格式
    let e164 = Regex::new(r"^\+[1-9][0-9]{7,14}$").unwrap();
    if e164.is_match(phone) {
        return true;
    }
    // 中国大陆手机
    let cn = Regex::new(r"^\+86(?:1[3-9][0-9]{9})$").unwrap();
    if cn.is_match(phone) { return true; }
    // 美国/加拿大
    let us = Regex::new(r"^\+1(?:[2-9][0-9]{9})$").unwrap();
    if us.is_match(phone) { return true; }
    // 英国
    let uk = Regex::new(r"^\+44(?:7[0-9]{9})$").unwrap();
    if uk.is_match(phone) { return true; }
    // 德国（简化常见移动号段匹配）
    let de = Regex::new(r"^\+49(?:1[5-7][0-9]{8,9}|1[0-9]{9})$").unwrap();
    if de.is_match(phone) { return true; }
    // 法国（移动号段 6/7 开头）
    let fr = Regex::new(r"^\+33(?:[67][0-9]{8})$").unwrap();
    if fr.is_match(phone) { return true; }
    // 沙特阿拉伯（移动号段 5 开头，NSN 共9位）
    let sa = Regex::new(r"^\+966(?:5[0-9]{8})$").unwrap();
    if sa.is_match(phone) { return true; }

    false
}

pub fn is_valid_email(email: &str) -> bool {
    // 简单邮箱校验
    let re = Regex::new(r"^[^@\s]+@[^@\s]+\.[^@\s]+$").unwrap();
    re.is_match(email)
}