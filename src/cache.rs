use std::time::{SystemTime, UNIX_EPOCH};

use deadpool_redis::redis::AsyncCommands;

use crate::{common::PageResult, utils::redis::REDIS_POOL};

const TOKEN_CACHE_KEY: &str = "user:session:token";
const ONLINE_USERS_KEY: &str = "user:online";



/// 保存用户的 access token 到 Redis Hash，并设置过期时间（单位：毫秒）
/// - Hash key: TOKEN_CACHE_HASH_KEY
/// - field: user_id（字符串）
/// - value: access_token
pub async fn save_user_access_token(
    user_id: u64,
    access_token: &str,
    ttl_millis: usize,
) -> Result<(), String> {
    let ttl_secs = ttl_millis.saturating_div(1000) as u64;
    let mut conn = REDIS_POOL
        .get()
        .await
        .map_err(|e| format!("redis get conn error: {}", e))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as usize;

    let expire_at = now + ttl_millis / 1000;

    // 用 zset 存储：member=user_id，score=过期时间戳
    let _: () = conn
        .zadd(ONLINE_USERS_KEY, user_id, expire_at as isize)
        .await
        .map_err(|e| format!("redis zadd error: {}", e))?;

    // 你如果还需要存 token 本体，可以单独存储：
    let user_token_key = format!("{}:{}", TOKEN_CACHE_KEY, user_id);
    let _: () = conn
        .set_ex(&user_token_key, access_token, ttl_secs)
        .await
        .map_err(|e| format!("redis set_ex error: {}", e))?;

    Ok(())
}

/// 获取在线用户数量（自动排除过期）
pub async fn get_online_user_count() -> Result<u64, String> {
    let mut conn = REDIS_POOL
        .get()
        .await
        .map_err(|e| format!("redis get conn error: {}", e))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as isize;

    // 先清理掉已过期的用户
    let _: () = conn
        .zrembyscore(ONLINE_USERS_KEY, 0, now)
        .await
        .map_err(|e| format!("redis zrembyscore error: {}", e))?;

    // 获取剩余的在线人数
    let count: u64 = conn
        .zcount(ONLINE_USERS_KEY, now, "+inf")
        .await
        .map_err(|e| format!("redis zcount error: {}", e))?;

    Ok(count)
}

/// 删除用户在缓存中的 access token（用于注销 Session）
pub async fn delete_user_access_token(user_id: u64) -> Result<(), String> {
    let mut conn = REDIS_POOL.get().await.map_err(|e| format!("redis get conn error: {}", e))?;
    // 删除单用户 token 键
    let _: () = conn
        .del(format!("{}:{}",TOKEN_CACHE_KEY, user_id.to_string()))
        .await
        .map_err(|e| format!("redis del error: {}", e))?;
    // 同步从在线统计 ZSet 中移除该用户
    let _: () = conn
        .zrem(ONLINE_USERS_KEY, user_id)
        .await
        .map_err(|e| format!("redis zrem error: {}", e))?;

    Ok(())
}

/// 分页获取在线用户 ID
pub async fn get_online_user_ids_paginated(
    page: isize,
    page_size: isize,
) -> Result<PageResult<u64>, String> {
    let mut conn = REDIS_POOL
        .get()
        .await
        .map_err(|e| format!("redis get conn error: {}", e))?;

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as isize;

    // 清理过期用户
    let _: () = conn
        .zrembyscore(ONLINE_USERS_KEY, 0, now)
        .await
        .map_err(|e| format!("redis zrembyscore error: {}", e))?;

    // 获取剩余的在线人数
    let total: u64 = conn
        .zcount(ONLINE_USERS_KEY, now, "+inf")
        .await
        .map_err(|e| format!("redis zcount error: {}", e))?;

    // 计算分页起点
    let offset = (page.saturating_sub(1)) * page_size;

    // 获取分页后的在线用户 ID
    let ids: Vec<u64> = conn
        .zrangebyscore_limit(ONLINE_USERS_KEY, now, "+inf", offset, page_size)
        .await
        .map_err(|e| format!("redis zrangebyscore_limit error: {}", e))?;

    Ok(PageResult { total, records: ids })
}
