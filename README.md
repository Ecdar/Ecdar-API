# Ecdar-API
API server between an ECDAR frontend and Reveaal

Prerequisites:
- Install docker and docker-compose (both comes with docker desktop)
- Install sea-orm-cli (`cargo install sea-orm-cli`)
- Set up `.env` file

Set up docker postgresql db:
- `docker-compose up -d`
- `DATABASE_URL=postgresql://postgres:{POSTGRES_PASSWORD}@{POSTGRES_DEV_IP}:{POSTGRES_DEV_PORT}/{POSTGRES_DB} sea-orm-cli migrate up`
- `DATABASE_URL=postgresql://postgres:{POSTGRES_PASSWORD}@{POSTGRES_TEST_IP}:{POSTGRES_TEST_PORT}/{POSTGRES_DB} sea-orm-cli migrate up`

After modifying DB:
- `sea-orm-cli migrate fresh`
- `sea-orm-cli generate entity -o src/entities`

Run tests:
- `cargo test -- --test-threads=1`