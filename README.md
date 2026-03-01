

#### 1. Lancer les services backend (PostgreSQL, Redis, API services)
```bash
docker-compose down -v && docker-compose up -d --build
```

#### 2. Lancer l'application frontend
```bash
cargo run --package frontend
```


