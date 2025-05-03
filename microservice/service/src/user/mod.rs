use async_trait::async_trait;
use db::{
    Error as SqlxError, SqlitePool, db as database,
    faker_rand::en_us::names::FullName,
    models::User,
    public_key::PublicKey,
    public_key_hash::PublicKeyHash,
    uuid::{self, Uuid},
};
use mockall::automock;
use shared::{
    errors::AppError,
    models::{RegisterRequest, RegisterResponse, UpdateUserRequest},
};

#[automock]
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn insert_user(&self, public_key: &str, username: &str) -> Result<Uuid, AppError>;

    async fn get_user_by_pubkey(&self, public_key_hash: &str) -> Result<User, AppError>;

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AppError>;

    async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, AppError>;

    async fn update_user(
        &self,
        user_id: Uuid,
        new_username: Option<String>,
        new_public_key: Option<String>,
        new_public_key_hash: Option<String>,
    ) -> Result<(), AppError>;

    async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, AppError>;
}

#[async_trait]
impl UserRepository for SqlitePool {
    async fn insert_user(&self, public_key: &str, username: &str) -> Result<Uuid, AppError> {
        let pkey = PublicKey::new(public_key.to_string())?;

        let pkey_hash = pkey.to_hash()?;

        let uid = database::insert_user(self, &pkey_hash, &pkey, username).await?;

        Ok(uid)
    }

    async fn get_user_by_pubkey(&self, public_key_hash: &str) -> Result<User, AppError> {
        let pkey_hash = PublicKeyHash::new(public_key_hash.to_string())?;

        let user = database::get_user_by_pubkey(self, &pkey_hash).await?;
        Ok(user)
    }

    async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        let user = database::get_user_by_id(self, user_id).await?;
        Ok(user)
    }

    async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, AppError> {
        let users = database::get_users(self, limit).await?;
        Ok(users)
    }

    async fn update_user(
        &self,
        user_id: Uuid,
        new_username: Option<String>,
        new_public_key: Option<String>,
        new_public_key_hash: Option<String>,
    ) -> Result<(), AppError> {
        let (new_pubkey, new_pubkey_hash) = if let Some(pubkey_str) = new_public_key {
            let pubkey = PublicKey::new(pubkey_str)?;
            let pubkey_hash = pubkey.to_hash()?;
            (Some(pubkey), Some(pubkey_hash))
        } else {
            (None, None)
        };

        database::update_user(
            self,
            user_id,
            new_username.as_deref(),
            new_pubkey.as_ref(),
            new_pubkey_hash.as_ref(),
        )
        .await?;

        Ok(())
    }

    async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, AppError> {
        let pk_hash = database::fetch_public_key_hash(self, user_id).await?;

        Ok(pk_hash)
    }
}

#[derive(Clone)]
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn register_user(
        &self,
        request: RegisterRequest,
    ) -> Result<RegisterResponse, AppError> {
        let user_id = uuid::Uuid::now_v7();

        let username = match request.username {
            Some(name) => name,
            None => generate_random_username(),
        };

        let pkeyclone = request.public_key.as_str();

        let user_id = self
            .repository
            .insert_user(&request.public_key, &username)
            .await?;

        Ok(RegisterResponse {
            user_id: user_id,
            username: username.to_string(),
        })
    }

    pub async fn get_user_by_public_key(&self, public_key: &str) -> Result<User, AppError> {
        let public_key_hash = sha256_hash(public_key)?;
        self.repository.get_user_by_pubkey(&public_key_hash).await
    }

    pub async fn get_user_by_id(&self, user_id: Uuid) -> Result<User, AppError> {
        self.repository.get_user_by_id(user_id).await
    }

    pub async fn get_users(&self, limit: Option<i64>) -> Result<Vec<User>, AppError> {
        self.repository.get_users(limit).await
    }

    pub async fn fetch_public_key_hash(&self, user_id: Uuid) -> Result<String, AppError> {
        self.repository.fetch_public_key_hash(user_id).await
    }

    pub async fn update_user(
        &self,
        user_id: Uuid,
        request: UpdateUserRequest,
    ) -> Result<(), AppError> {
        let mut public_key_hash: Option<String> = None;
        if let Some(ref public_key) = request.new_public_key {
            public_key_hash = Some(sha256_hash(&public_key)?);
        }

        self.repository
            .update_user(
                user_id,
                request.new_username,
                request.new_public_key,
                public_key_hash,
            )
            .await?;

        Ok(())
    }
}

fn generate_random_username() -> String {
    rand::random::<FullName>().to_string().replace(" ", "")
}

fn sha256_hash(input: &str) -> Result<String, AppError> {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(input.as_bytes());
    let result = hasher.finalize();
    Ok(base64::encode(result))
}

#[cfg(test)]
mod tests {
    use super::*;
    use base64::{
        Engine as _, alphabet,
        engine::{self, general_purpose},
    };
    use chrono::{NaiveDateTime, Utc};
    use mockall::{mock, predicate::*};
    use p256::{
        ecdsa::{SigningKey, VerifyingKey},
        elliptic_curve::rand_core::OsRng,
    };
    use rand::Rng;

    impl Clone for MockUserRepository {
        fn clone(&self) -> Self {
            Self::default()
        }
    }

    const CUSTOM_ENGINE: engine::GeneralPurpose =
        engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

    pub async fn generate_key() -> (PublicKey, PublicKeyHash) {
        let signing_key = SigningKey::random(&mut OsRng);
        let verifying_key = VerifyingKey::from(&signing_key);

        let b64_key = CUSTOM_ENGINE.encode(verifying_key.to_encoded_point(true).as_bytes());
        let public_key = PublicKey::new(b64_key).unwrap();

        let public_key_hash = public_key.to_hash().unwrap();

        (public_key, public_key_hash)
    }

    async fn create_test_user(id: Uuid, username: &str) -> User {
        let now = Utc::now().naive_utc();
        let (public_key, public_key_hash) = generate_key().await;
        User {
            id,
            username: username.to_string(),
            public_key: public_key,
            public_key_hash: public_key_hash,
            created_at: now,
            updated_at: now,
            last_login: None,
        }
    }

    #[tokio::test]
    async fn test_register_user_with_username() {
        let mut mock_repo = MockUserRepository::new();

        let test_public_key = "test_public_key";
        let public_key_hash = sha256_hash(test_public_key).unwrap();
        let expected_user_id = Uuid::now_v7();

        mock_repo
            .expect_insert_user()
            .with(eq(test_public_key), eq("testuser"))
            .times(1)
            .returning(move |_, _| Ok(expected_user_id));

        let service = UserService::new(mock_repo);
        let request = RegisterRequest {
            username: Some("testuser".to_string()),
            public_key: test_public_key.to_string(),
        };

        let result = service.register_user(request).await.unwrap();

        assert_eq!(result.user_id, expected_user_id);
        assert_eq!(result.username, "testuser");
    }

    #[tokio::test]
    async fn test_register_user_without_username() {
        let mut mock_repo = MockUserRepository::new();
        let test_public_key = "test_public_key";
        let public_key_hash = sha256_hash(test_public_key).unwrap();
        let expected_user_id = Uuid::now_v7();

        // Setup mock to accept any username
        mock_repo
            .expect_insert_user()
            .with(eq(test_public_key), always())
            .times(1)
            .returning(move |_, username| {
                assert!(!username.is_empty());
                Ok(expected_user_id)
            });

        let service = UserService::new(mock_repo);
        let request = RegisterRequest {
            username: None,
            public_key: test_public_key.to_string(),
        };

        let result = service.register_user(request).await.unwrap();

        assert_eq!(result.user_id, expected_user_id);
        assert!(!result.username.is_empty());
    }

    #[tokio::test]
    async fn test_register_user_db_error() {
        let mut mock_repo = MockUserRepository::new();
        let test_public_key = "test_public_key";
        let public_key_hash = sha256_hash(test_public_key).unwrap();

        mock_repo
            .expect_insert_user()
            .with(eq(test_public_key), eq("testuser"))
            .times(1)
            .returning(|_, _| {
                Err(AppError::DatabaseError(SqlxError::InvalidArgument(
                    "DB error".to_string(),
                )))
            });

        let service = UserService::new(mock_repo);
        let request = RegisterRequest {
            username: Some("testuser".to_string()),
            public_key: test_public_key.to_string(),
        };

        let result = service.register_user(request).await;
        assert!(matches!(result, Err(AppError::DatabaseError(_))));
    }

    #[tokio::test]
    async fn test_get_user_by_public_key_success() {
        let mut mock_repo = MockUserRepository::new();
        let test_public_key = "test_public_key";
        let public_key_hash = sha256_hash(test_public_key).unwrap();
        let test_user = create_test_user(Uuid::now_v7(), "testuser").await;

        let test_user_key = test_user.public_key.clone();

        mock_repo
            .expect_get_user_by_pubkey()
            .with(eq(public_key_hash.clone()))
            .times(1)
            .returning(move |_| Ok(test_user.clone()));

        let service = UserService::new(mock_repo);
        let result = service
            .get_user_by_public_key(test_public_key)
            .await
            .unwrap();

        assert_eq!(result.username, "testuser");
        assert_eq!(result.public_key.as_str(), test_user_key.as_str());
    }

    #[tokio::test]
    async fn test_get_user_by_public_key_not_found() {
        let mut mock_repo = MockUserRepository::new();
        let test_public_key = "test_public_key";
        let public_key_hash = sha256_hash(test_public_key).unwrap();

        mock_repo
            .expect_get_user_by_pubkey()
            .with(eq(public_key_hash.clone()))
            .times(1)
            .returning(|_| Err(AppError::NotFound("User not found".to_string())));

        let service = UserService::new(mock_repo);
        let result = service.get_user_by_public_key(test_public_key).await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[test]
    fn test_generate_random_username() {
        let username1 = generate_random_username();
        let username2 = generate_random_username();

        assert!(!username1.is_empty());
        assert!(!username1.contains(' '));
        assert_ne!(username1, username2);
    }

    #[test]
    fn test_sha256_hash() {
        let input = "test_input";
        let hash1 = sha256_hash(input).unwrap();
        let hash2 = sha256_hash(input).unwrap();
        let different_hash = sha256_hash("different_input").unwrap();

        assert_eq!(hash1, hash2);
        assert_ne!(hash1, different_hash);
        assert!(base64::decode(&hash1).is_ok());
    }

    #[tokio::test]
    async fn test_get_user_by_id_success() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let test_user = create_test_user(user_id, "testuser").await;
        let test_user_key = test_user.public_key.clone();

        mock_repo
            .expect_get_user_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(test_user.clone()));

        let service = UserService::new(mock_repo);
        let result = service.get_user_by_id(user_id).await.unwrap();

        assert_eq!(result.id, user_id);
        assert_eq!(result.username, "testuser");
        assert_eq!(result.public_key.as_str(), test_user_key.as_str());
    }

    #[tokio::test]
    async fn test_get_user_by_id_not_found() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();

        mock_repo
            .expect_get_user_by_id()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(AppError::NotFound("User not found".to_string())));

        let service = UserService::new(mock_repo);
        let result = service.get_user_by_id(user_id).await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_get_users_with_limit() {
        let mut mock_repo = MockUserRepository::new();
        let limit = Some(10);
        let test_users = vec![
            create_test_user(Uuid::now_v7(), "user1").await,
            create_test_user(Uuid::now_v7(), "user2").await,
        ];
        let test_user_key0 = test_users[0].public_key.clone();
        let test_user_key1 = test_users[1].public_key.clone();

        mock_repo
            .expect_get_users()
            .with(eq(limit))
            .times(1)
            .returning(move |_| Ok(test_users.clone()));

        let service = UserService::new(mock_repo);
        let result = service.get_users(limit).await.unwrap();

        assert_eq!(result.len(), 2);
        assert_eq!(result[0].username, "user1");
        assert_eq!(result[0].public_key.as_str(), test_user_key0.as_str());
        assert_eq!(result[1].username, "user2");
        assert_eq!(result[1].public_key.as_str(), test_user_key1.as_str());
    }

    #[tokio::test]
    async fn test_get_users_without_limit() {
        let mut mock_repo = MockUserRepository::new();
        let test_users = vec![
            create_test_user(Uuid::now_v7(), "user1").await,
            create_test_user(Uuid::now_v7(), "user2").await,
            create_test_user(Uuid::now_v7(), "user3").await,
        ];

        mock_repo
            .expect_get_users()
            .with(eq(None))
            .times(1)
            .returning(move |_| Ok(test_users.clone()));

        let service = UserService::new(mock_repo);
        let result = service.get_users(None).await.unwrap();

        assert_eq!(result.len(), 3);
    }

    #[tokio::test]
    async fn test_get_users_empty() {
        let mut mock_repo = MockUserRepository::new();

        mock_repo
            .expect_get_users()
            .with(eq(None))
            .times(1)
            .returning(|_| Ok(Vec::new()));

        let service = UserService::new(mock_repo);
        let result = service.get_users(None).await.unwrap();

        assert!(result.is_empty());
    }

    #[tokio::test]
    async fn test_get_users_db_error() {
        let mut mock_repo = MockUserRepository::new();

        mock_repo
            .expect_get_users()
            .with(eq(Some(5)))
            .times(1)
            .returning(|_| {
                Err(AppError::DatabaseError(SqlxError::InvalidArgument(
                    "DB error".to_string(),
                )))
            });

        let service = UserService::new(mock_repo);
        let result = service.get_users(Some(5)).await;

        assert!(matches!(result, Err(AppError::DatabaseError(_))));
    }

    #[tokio::test]
    async fn test_update_user_success() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let new_public_key = "new_public_key";
        let public_key_hash = sha256_hash(new_public_key).unwrap();
        let request = UpdateUserRequest {
            new_username: Some("new_username".to_string()),
            new_public_key: Some(new_public_key.to_string()),
        };

        mock_repo
            .expect_update_user()
            .with(
                eq(user_id),
                eq(Some("new_username".to_string())),
                eq(Some(new_public_key.to_string())),
                eq(Some(public_key_hash)),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = UserService::new(mock_repo);
        let result = service.update_user(user_id, request).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_user_partial_fields() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let new_username = "new_username";

        // Test updating just username
        let username_request = UpdateUserRequest {
            new_username: Some(new_username.to_string()),
            new_public_key: None,
        };
        let mut mock_repo2 = mock_repo.clone();

        mock_repo
            .expect_update_user()
            .with(
                eq(user_id),
                eq(Some(new_username.to_string())),
                eq(None),
                eq(None),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = UserService::new(mock_repo);
        let result = service.update_user(user_id, username_request).await;
        assert!(result.is_ok());

        // Test updating just public key
        let new_public_key = "new_public_key";
        let public_key_hash = sha256_hash(new_public_key).unwrap();
        let public_key_request = UpdateUserRequest {
            new_username: None,
            new_public_key: Some(new_public_key.to_string()),
        };

        mock_repo2
            .expect_update_user()
            .with(
                eq(user_id),
                eq(None),
                eq(Some(new_public_key.to_string())),
                eq(Some(public_key_hash)),
            )
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = UserService::new(mock_repo2);
        let result = service.update_user(user_id, public_key_request).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_user_no_changes() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let request = UpdateUserRequest {
            new_username: None,
            new_public_key: None,
        };

        mock_repo
            .expect_update_user()
            .with(eq(user_id), eq(None), eq(None), eq(None))
            .times(1)
            .returning(|_, _, _, _| Ok(()));

        let service = UserService::new(mock_repo);
        let result = service.update_user(user_id, request).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_update_user_db_error() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let request = UpdateUserRequest {
            new_username: Some("new_username".to_string()),
            new_public_key: Some("new_public_key".to_string()),
        };

        mock_repo
            .expect_update_user()
            .with(always(), always(), always(), always())
            .times(1)
            .returning(|_, _, _, _| {
                Err(AppError::DatabaseError(SqlxError::InvalidArgument(
                    "DB error".to_string(),
                )))
            });

        let service = UserService::new(mock_repo);
        let result = service.update_user(user_id, request).await;

        assert!(matches!(result, Err(AppError::DatabaseError(_))));
    }

    #[tokio::test]
    async fn test_fetch_public_key_hash_success() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();
        let expected_hash = "test_hash";

        mock_repo
            .expect_fetch_public_key_hash()
            .with(eq(user_id))
            .times(1)
            .returning(move |_| Ok(expected_hash.to_string()));

        let service = UserService::new(mock_repo);
        let result = service.fetch_public_key_hash(user_id).await.unwrap();

        assert_eq!(result, expected_hash.to_string());
    }

    #[tokio::test]
    async fn test_fetch_public_key_hash_not_found() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();

        mock_repo
            .expect_fetch_public_key_hash()
            .with(eq(user_id))
            .times(1)
            .returning(|_| Err(AppError::NotFound("User not found".to_string())));

        let service = UserService::new(mock_repo);
        let result = service.fetch_public_key_hash(user_id).await;

        assert!(matches!(result, Err(AppError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_fetch_public_key_hash_db_error() {
        let mut mock_repo = MockUserRepository::new();
        let user_id = Uuid::now_v7();

        mock_repo
            .expect_fetch_public_key_hash()
            .with(eq(user_id))
            .times(1)
            .returning(|_| {
                Err(AppError::DatabaseError(SqlxError::InvalidArgument(
                    "DB error".to_string(),
                )))
            });

        let service = UserService::new(mock_repo);
        let result = service.fetch_public_key_hash(user_id).await;

        assert!(matches!(result, Err(AppError::DatabaseError(_))));
    }
}
