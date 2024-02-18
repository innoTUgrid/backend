use redis::aio::Connection;
use redis::{AsyncCommands, RedisResult};

const REDIS_CONNECTION_STRING: &str = "redis://127.0.0.1:12758/";

pub struct Cache {
    connection: Connection,
}

impl Cache {
    pub async fn new() -> RedisResult<Self> {
        let client = redis::Client::open(REDIS_CONNECTION_STRING)?;
        let connection = client.get_async_connection().await?;
        Ok(Self { connection })
    }
    pub(crate) async fn get(&mut self, key: &str) -> RedisResult<String> {
        let result: String = self.connection.get(key).await?;
        Ok(result)
    }
    pub(crate) async fn set(
        &mut self,
        key: &str,
        value: &str,
        ttl_seconds: i64,
    ) -> RedisResult<()> {
        self.connection.set(key, value).await?;
        if ttl_seconds > 0 {
            self.connection.expire(key, ttl_seconds).await?;
        }
        Ok(())
    }
}

#[tokio::test]
async fn test_cache_vec_structs() {
    use serde::{Deserialize, Serialize};
    #[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
    struct TestStruct {
        some_field: String,
        some_other_field: f64,
    }
    let mut cache = Cache::new().await.unwrap();
    let key = "some_key";
    let data = vec![
        TestStruct {
            some_field: "some_test".to_string(),
            some_other_field: 1.0,
        },
        TestStruct {
            some_field: "some_test2".to_string(),
            some_other_field: 2.0,
        },
    ];
    let serialized = serde_json::to_string(&data).unwrap();
    cache.set(key, &serialized, 5).await.unwrap();
    let cached = cache.get(key).await.unwrap();
    let deserialized: Vec<TestStruct> = serde_json::from_str(&cached).unwrap();
    let matching = data
        .iter()
        .zip(deserialized.iter())
        .filter(|&(a, b)| a == b)
        .count();
    assert!(matching == data.len() && matching == deserialized.len());
}
