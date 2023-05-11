# Architecture Design Record
## ADR-08, Data Store

Postgres will be used. Or maybe SurrealDB!!!

```sh
docker run --rm --pull always -p 80:8000 -v ./surrealdb:/opt/surrealdb/ surrealdb/surrealdb:latest start --log trace --user root --pass root file:/opt/surrealdb/iot.db
```

```sh
curl -I -X GET localhost:80/health

curl -X POST \
    -u "root:root" \
    -H "NS: test" \
    -H "DB: test" \
    -H "Accept: application/json" \
    -d "SELECT * FROM person" \
    http://localhost:80/sql
```

```json
[
  {
    "time": "127.271\u00b5s",
    "status": "OK",
    "result": [
      {
        "id": "person:524rxscey5vz5uvx51p9",
        "marketing": true,
        "name": {
          "first": "Tobie",
          "last": "Morgan Hitchcock"
        },
        "title": "Founder & CEO"
      },
      {
        "id": "person:jaime",
        "marketing": true
      }
    ]
  }
]
```
