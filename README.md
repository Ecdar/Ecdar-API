# Ecdar-API
API server between an ECDAR frontend and Reveaal

`cargo install sea-orm-cli`
`sea-orm-cli migrate up`
`sea-orm-cli generate entity -o src/entities`

`docker run --rm -p 5432:5432 -e "POSTGRES_PASSWORD=postgres --name pg postgres:14`
`docker exec -it -u postgres pg psql`