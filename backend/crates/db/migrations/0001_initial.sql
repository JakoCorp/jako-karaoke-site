-- Users
-- Username is case-insensitive for uniqueness (ci collation) but case-preserving
CREATE TABLE IF NOT EXISTS users (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    username VARCHAR(64) CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci NOT NULL,
    twitch_id BIGINT UNSIGNED NULL,
    discord_id BIGINT UNSIGNED NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX (username),
    UNIQUE INDEX (twitch_id),
    UNIQUE INDEX (discord_id)
) ENGINE = InnoDB;

-- Pending OAuth claims awaiting username selection
CREATE TABLE IF NOT EXISTS pending_oauth (
    token CHAR(64) NOT NULL,
    provider VARCHAR(16) NOT NULL,
    provider_id BIGINT UNSIGNED NOT NULL,
    suggested_username VARCHAR(64) NOT NULL,
    expires_at DATETIME NOT NULL,
    PRIMARY KEY (token),
    INDEX (expires_at)
) ENGINE = InnoDB;

-- User Capabilities
CREATE TABLE IF NOT EXISTS user_capabilities (
    user_id BINARY(16) NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    capability VARCHAR(64) NOT NULL,
    PRIMARY KEY (user_id, capability)
) ENGINE = InnoDB;

-- Sessions
-- id is the SHA256 hash (hex) of the opaque session token issued to the client.
CREATE TABLE IF NOT EXISTS sessions (
    id CHAR(64) NOT NULL,
    user_id BINARY(16) NOT NULL,
    created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    expires_at DATETIME NOT NULL,
    PRIMARY KEY (id),
    INDEX (user_id),
    INDEX (expires_at),
    CONSTRAINT FOREIGN KEY (user_id) REFERENCES users (id) ON DELETE CASCADE
) ENGINE = InnoDB;

-- Tags
CREATE TABLE IF NOT EXISTS tags (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    name VARCHAR(64) NOT NULL,
    PRIMARY KEY (id),
    UNIQUE INDEX (name)
) ENGINE = InnoDB;

-- Lyrics
CREATE TABLE IF NOT EXISTS lyrics (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    content TEXT NOT NULL,
    PRIMARY KEY (id)
) ENGINE = InnoDB;

-- Images
-- Can be cover art, thumbnail, etc.
CREATE TABLE IF NOT EXISTS images (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    public_url VARCHAR(512) NOT NULL,
    internal_path VARCHAR(512) NULL,
    credits TEXT NULL,
    PRIMARY KEY (id)
) ENGINE = InnoDB;

-- Playlists
CREATE TABLE IF NOT EXISTS playlists (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    title VARCHAR(256) NOT NULL,
    description TEXT NULL,
    kind VARCHAR(64) NOT NULL,
    is_public BOOLEAN NOT NULL DEFAULT TRUE,
    created_by BINARY(16) NULL REFERENCES users (id) ON DELETE CASCADE,
    PRIMARY KEY (id),
    INDEX (title),
    INDEX (kind),
    INDEX (is_public),
    INDEX (created_by, kind)
) ENGINE = InnoDB;

-- Artists
-- Representative of both original artists of a song and any singers.
CREATE TABLE IF NOT EXISTS artists (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    name VARCHAR(256) NOT NULL,
    description TEXT NULL,
    PRIMARY KEY (id),
    INDEX (name)
) ENGINE = InnoDB;

-- Artist external links (e.g. YouTube, website)
CREATE TABLE IF NOT EXISTS artist_links (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    artist_id BINARY(16) NOT NULL REFERENCES artists (id) ON DELETE CASCADE,
    url VARCHAR(1024) NOT NULL,
    kind VARCHAR(32) NOT NULL,
    label VARCHAR(128) NULL,
    PRIMARY KEY (id),
    INDEX (artist_id)
) ENGINE = InnoDB;

-- Artist <-> Image (M2M)
-- kind denotes the semantic role of the image (e.g. "avatar").
CREATE TABLE IF NOT EXISTS artist_images (
    artist_id BINARY(16) NOT NULL REFERENCES artists (id) ON DELETE CASCADE,
    image_id BINARY(16) NOT NULL REFERENCES images (id) ON DELETE CASCADE,
    kind VARCHAR(32) NOT NULL,
    PRIMARY KEY (artist_id, image_id),
    INDEX (artist_id)
) ENGINE = InnoDB;

-- Songs
-- Has default base lyrics of a song
CREATE TABLE IF NOT EXISTS songs (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    title VARCHAR(256) NOT NULL,
    created_by BINARY(16) NULL REFERENCES users (id) ON DELETE SET NULL,
    lyrics_id BINARY(16) NULL REFERENCES lyrics (id) ON DELETE SET NULL,
    date_added DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
    PRIMARY KEY (id),
    INDEX (title),
    INDEX (date_added)
) ENGINE = InnoDB;

-- Song <-> Image (M2M)
CREATE TABLE IF NOT EXISTS song_images (
    song_id BINARY(16) NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
    image_id BINARY(16) NOT NULL REFERENCES images (id) ON DELETE CASCADE,
    PRIMARY KEY (song_id, image_id)
) ENGINE = InnoDB;

-- Song <-> Original Artist (M2M)
CREATE TABLE IF NOT EXISTS song_original_artists (
    song_id BINARY(16) NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
    artist_id BINARY(16) NOT NULL REFERENCES artists (id) ON DELETE CASCADE,
    PRIMARY KEY (song_id, artist_id),
    INDEX (artist_id)
) ENGINE = InnoDB;

-- Song <-> Tag (M2M)
CREATE TABLE IF NOT EXISTS song_tags (
    song_id BINARY(16) NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
    tag_id BINARY(16) NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    kind VARCHAR(32) NOT NULL,
    PRIMARY KEY (song_id, tag_id),
    INDEX (tag_id)
) ENGINE = InnoDB;

-- Performances
-- Representative of a song/mashup performed on stream
-- Providing a lyrics means:
--  Its a custom lyrics tied to this particular performance.
--  Or its a mashup of multiple songs and needs one lyrics from the set.
CREATE TABLE IF NOT EXISTS performances (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    title VARCHAR(256) NULL,
    created_by BINARY(16) NULL REFERENCES users (id) ON DELETE SET NULL,
    lyrics_id BINARY(16) NULL REFERENCES lyrics (id) ON DELETE SET NULL,
    play_count INT NOT NULL DEFAULT 0,
    duration INT UNSIGNED NULL,
    stream_time INT UNSIGNED NULL,
    performance_date DATE NOT NULL,
    stream_number TINYINT UNSIGNED NOT NULL,
    performance_number SMALLINT UNSIGNED NOT NULL,
    PRIMARY KEY (id),
    INDEX (performance_date),
    UNIQUE INDEX (performance_date, stream_number, performance_number)
) ENGINE = InnoDB;

-- Performance <-> Song (M2M)
CREATE TABLE IF NOT EXISTS performance_songs (
    performance_id BINARY(16) NOT NULL REFERENCES performances (id) ON DELETE CASCADE,
    song_id BINARY(16) NOT NULL REFERENCES songs (id) ON DELETE CASCADE,
    PRIMARY KEY (performance_id, song_id),
    INDEX (song_id)
) ENGINE = InnoDB;

-- Performance <-> Tag (M2M)
CREATE TABLE IF NOT EXISTS performance_tags (
    performance_id BINARY(16) NOT NULL REFERENCES performances (id) ON DELETE CASCADE,
    tag_id BINARY(16) NOT NULL REFERENCES tags (id) ON DELETE CASCADE,
    kind VARCHAR(32) NOT NULL,
    PRIMARY KEY (performance_id, tag_id),
    INDEX (tag_id)
) ENGINE = InnoDB;

-- Performance <-> Singer (M2M)
CREATE TABLE IF NOT EXISTS performance_singers (
    performance_id BINARY(16) NOT NULL REFERENCES performances (id) ON DELETE CASCADE,
    artist_id BINARY(16) NOT NULL REFERENCES artists (id) ON DELETE CASCADE,
    PRIMARY KEY (performance_id, artist_id),
    INDEX (artist_id)
) ENGINE = InnoDB;

-- Performance audios
CREATE TABLE IF NOT EXISTS performance_audios (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    performance_id BINARY(16) NOT NULL REFERENCES performances (id) ON DELETE CASCADE,
    public_url VARCHAR(512) NOT NULL,
    internal_path VARCHAR(512) NULL,
    PRIMARY KEY (id),
    INDEX (performance_id)
) ENGINE = InnoDB;

-- Performance videos
CREATE TABLE IF NOT EXISTS performance_videos (
    id BINARY(16) NOT NULL DEFAULT (UNHEX(REPLACE(UUID_V7(), '-', ''))),
    performance_id BINARY(16) NOT NULL REFERENCES performances (id) ON DELETE CASCADE,
    public_url VARCHAR(512) NOT NULL,
    internal_path VARCHAR(512) NULL,
    PRIMARY KEY (id),
    INDEX (performance_id)
) ENGINE = InnoDB;

-- Playlist <-> Performance (M2M, ordered)
CREATE TABLE IF NOT EXISTS playlist_performances (
    playlist_id BINARY(16) NOT NULL REFERENCES playlists (id) ON DELETE CASCADE,
    performance_id BINARY(16) NOT NULL REFERENCES performances (id) ON DELETE CASCADE,
    sort_order INT NOT NULL DEFAULT 0,
    PRIMARY KEY (playlist_id, performance_id),
    INDEX (performance_id)
) ENGINE = InnoDB;
