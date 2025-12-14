cd auth_service
sqlx migrate run
cargo sqlx prepare
cd ../user_service
sqlx migrate run
cargo sqlx prepare
