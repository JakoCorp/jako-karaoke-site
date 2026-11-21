# Jako Karaoke Site

## Running

### Docker Compose

Runs the full stack server.

A `data/` folder is created at the repo root and mounted into the compose services (`data/storage` for uploaded media, `data/backups` for backups, managed by the `data_db` named volume for MariaDB).

#### Environment

Create a `.env` file at the repo root with the following:

| Variable | Description |
| --- | --- |
| `DB_ROOT_PASSWORD` | Root password for the MariaDB container. Required, no default. |
| `STORAGE_BASE_URL` | Public base URL that maps to uploaded media (see backend README). Required, no default. |
| `TWITCH_CLIENT_ID` / `TWITCH_CLIENT_SECRET` | Twitch OAuth app credentials. |
| `DISCORD_CLIENT_ID` / `DISCORD_CLIENT_SECRET` | Discord OAuth app credentials. |
| `GOOGLE_API_KEY` | Optional Google Cloud API key (see backend README). |
| `SITE_DOMAIN` | Public domain for the frontend. Defaults to `localhost`. |
| `FRONTEND_URL` | Overrides the derived `https://${SITE_DOMAIN}` if set. |
| `API_DOMAIN` | Public domain for the backend. Defaults to `api.localhost`. |
| `BACKEND_URL` | Overrides the derived `https://${API_DOMAIN}` if set. |
| `BACKEND_RUST_LOG` | Overrides logger settings for the `backend` if set. |
| `GIT_COMMIT` | Commit hash built into the backend image and recorded in backups. Defaults to `unknown`. Usually passed on the command line (see Run). |
| `BACKUP_SCHEDULE_ENABLED` | Set to `true` to take scheduled backups (see Backups). Defaults to off. |
| `BACKUP_SCHEDULE` | Cron expression in UTC for scheduled backups. Defaults to `0 3 * * *`. |
| `BACKUP_KEEP_DAYS` | Days a scheduled backup is kept. `0` keeps them all. Defaults to `14`. |

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

# Optional
GOOGLE_API_KEY=

# Scheduled backups (optional)
BACKUP_SCHEDULE_ENABLED=
BACKUP_SCHEDULE=
BACKUP_KEEP_DAYS=

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
GIT_COMMIT=$(git rev-parse HEAD) docker compose up --build
```

`GIT_COMMIT` can be left out, but backups then record the commit as `unknown`.

Add the `debug` profile to also run `cadvisor` (container resource monitoring, exposed at `127.0.0.1:8080`):

```bash
docker compose --profile debug up --build
```

### Backups

A backup is one zip file in `data/backups`, named `jako-backup-<UTC timestamp>-<short commit>.zip`. It holds the database, every uploaded file and the commit of the running build. Copy the file anywhere to export it.

Backups contain user data (usernames and OAuth provider IDs), so store and share them with care. Login sessions are not included, so restoring logs everyone out.

```bash
# Create and list backups while the server runs
docker compose exec backend backup create
docker compose exec backend backup list

# Restore a backup (a file name in data/backups, a path, or latest)
docker compose stop backend
docker compose run --rm backend backup restore latest --yes
docker compose up -d backend
```

Restore replaces the whole database and every uploaded file. It first saves a backup of the current state, unless `--no-safety-backup` is passed. A backup can only be restored by a build whose migrations match it. When they do not, the restore stops before changing anything and names the commit to check out.

#### Scheduled backups

With `BACKUP_SCHEDULE_ENABLED=true`, the backend takes a backup on the `BACKUP_SCHEDULE` cron expression (UTC, daily at 03:00 by default). Scheduled backups end in `-scheduled.zip`, and those older than `BACKUP_KEEP_DAYS` (14 by default) are deleted after each new one. Backups from `backup create` and the safety backups taken by a restore are never deleted automatically. A run missed while the backend is down is skipped.
