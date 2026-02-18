docker-compose up auth_postgres -d
docker-compose up user_postgres -d
docker-compose up files_postgres -d
cd auth_service
sqlx migrate run
cargo sqlx prepare
cd ../user_service
sqlx migrate run
cargo sqlx prepare
cd ../files_service
sqlx migrate run
cargo sqlx prepare
cd ..
docker-compose down
