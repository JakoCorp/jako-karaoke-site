# Jako Karaoke Site

## Running

### Docker Compose

Runs the full stack server.

A `data/` folder is created at the repo root and mounted into the compose services (`data/storage` for uploaded media, managed by the `data_db` named volume for MariaDB).

#### Environment

Create a `.env` file at the repo root with the following:

| Variable | Description |
| --- | --- |
| `DB_ROOT_PASSWORD` | Root password for the MariaDB container. Required, no default. |
| `STORAGE_BASE_URL` | Public base URL that maps to uploaded media (see backend README). Required, no default. |
| `TWITCH_CLIENT_ID` / `TWITCH_CLIENT_SECRET` | Twitch OAuth app credentials. |
| `DISCORD_CLIENT_ID` / `DISCORD_CLIENT_SECRET` | Discord OAuth app credentials. |
| `SITE_DOMAIN` | Public domain for the frontend. Defaults to `localhost`. |
| `FRONTEND_URL` | Overrides the derived `https://${SITE_DOMAIN}` if set. |
| `API_DOMAIN` | Public domain for the backend. Defaults to `api.localhost`. |
| `BACKEND_URL` | Overrides the derived `https://${API_DOMAIN}` if set. |
| `BACKEND_RUST_LOG` | Overrides logger settings for the `backend` if set. |

<!-- markdownlint-disable MD033 -->
<details>
<summary>.env template</summary>

```bash
DB_ROOT_PASSWORD=
STORAGE_BASE_URL=

# OAuth
TWITCH_CLIENT_ID=
TWITCH_CLIENT_SECRET=
DISCORD_CLIENT_ID=
DISCORD_CLIENT_SECRET=

# Defaults to *.localhost for local development
SITE_DOMAIN=
FRONTEND_URL=
API_DOMAIN=
BACKEND_URL=

# Debug (optional)
BACKEND_RUST_LOG="server=debug,tower_http=debug"
```

</details>
<!-- markdownlint-enable MD033 -->

#### TLS certs

`nginx` expects a cert at `nginx/certs/origin.crt` / `nginx/certs/origin.key`. In production this should be an origin cert issued by whatever CDN sits in front.

For local development, use [`mkcert`](https://github.com/FiloSottile/mkcert) to generate a browser trusted cert:

```bash
mkcert -install
mkcert -key-file nginx/certs/origin.key -cert-file nginx/certs/origin.crt localhost api.localhost
```

#### Run

```bash
docker compose up --build
```

Add the `debug` profile to also run `cadvisor` (container resource monitoring, exposed at `127.0.0.1:8080`):

```bash
docker compose --profile debug up --build
```
