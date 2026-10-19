-- Development seed data.
--
-- user_a (01000000000000000000000000000001) is the dev admin used by /auth/dev-login.

-- Users
INSERT IGNORE INTO users (id, username) VALUES
(UNHEX('01000000000000000000000000000001'), 'user_a'),
(UNHEX('01000000000000000000000000000002'), 'user_b');

-- All capabilities granted to user_a
INSERT IGNORE INTO user_capabilities (user_id, capability) VALUES
(UNHEX('01000000000000000000000000000001'), 'capabilities:manage'),
(UNHEX('01000000000000000000000000000001'), 'artists:manage_any'),
(UNHEX('01000000000000000000000000000001'), 'songs:manage_any'),
(UNHEX('01000000000000000000000000000001'), 'tags:manage_any'),
(UNHEX('01000000000000000000000000000001'), 'performances:manage_any'),
(UNHEX('01000000000000000000000000000001'), 'playlists:view_private'),
(UNHEX('01000000000000000000000000000001'), 'playlists:create_official'),
(UNHEX('01000000000000000000000000000001'), 'playlists:create_favorites'),
(UNHEX('01000000000000000000000000000001'), 'playlists:manage_any'),
(UNHEX('01000000000000000000000000000001'), 'plays:manage_any');

-- Tags
INSERT IGNORE INTO tags (id, name) VALUES
(UNHEX('01000000000000000000000000000031'), 'tag_a'),
(UNHEX('01000000000000000000000000000032'), 'tag_b'),
(UNHEX('01000000000000000000000000000033'), 'tag_c'),
(UNHEX('01000000000000000000000000000034'), 'tag_d');

-- Artists
INSERT IGNORE INTO artists (id, name, description) VALUES
(UNHEX('01000000000000000000000000000011'), 'artist_a', 'Description for artist_a.'),
(
    UNHEX('01000000000000000000000000000012'),
    'artist_b',
    'Lorem ipsum dolor sit amet, consectetur adipiscing elit. '
    'Nullam dictum, dolor sed feugiat vulputate, mi ligula congue justo, at faucibus diam massa et ex. '
    'Nam commodo, mi quis consectetur porttitor, mi massa suscipit purus, ac hendrerit sapien ex a ex. '
    'Praesent imperdiet auctor urna vitae maximus. Suspendisse potenti. '
    'Curabitur faucibus efficitur mauris eget aliquam. Sed laoreet tincidunt ornare. '
    'Pellentesque a interdum lorem, in porta enim. '
    'Aenean cursus ullamcorper ante, et varius augue tristique eu. '
    'Duis pharetra nibh commodo, tempor dolor id, auctor eros. '
    'Nulla non libero bibendum, luctus justo eget, iaculis enim. '
    'Mauris id mi libero. Etiam suscipit vitae elit sed pellentesque. '
    'In elit tellus, eleifend at vulputate ut, rhoncus nec mauris.\n\n'
    'Nam mi justo, semper nec mi in, rhoncus malesuada sem. '
    'Aenean in magna tempor, porta ante nec, bibendum erat. '
    'Proin porta odio vitae vehicula venenatis. Phasellus iaculis maximus mauris id tempus. '
    'Maecenas finibus mi in dui vestibulum viverra. Pellentesque in suscipit magna. Nulla facilisi. '
    'Fusce ultricies mollis mi a tincidunt. Sed at justo sit amet eros cursus gravida a ut nisi. '
    'Proin in aliquam dui. Maecenas sit amet urna auctor ante iaculis blandit. '
    'Nulla facilisis mi iaculis urna elementum aliquam. '
    'Donec varius erat lorem, et vehicula leo ultricies vitae. '
    'Pellentesque sagittis facilisis euismod. Vivamus finibus quam sed elit volutpat auctor. '
    'Nunc non purus sed velit maximus scelerisque imperdiet at sapien.\n\n'
    'Donec mauris erat, dictum sit amet aliquam ac, pulvinar at odio. Vivamus vel venenatis nisl. '
    'Quisque vel bibendum felis. Curabitur vitae ante ut lectus rutrum condimentum. '
    'Vivamus blandit vitae ex in vehicula. Donec bibendum malesuada iaculis. '
    'Quisque finibus enim vel est lacinia, ut dignissim eros lacinia. '
    'Nulla laoreet, felis eu placerat tristique, sapien lorem bibendum lorem, a fermentum ipsum ante eu nibh. '
    'Duis aliquam turpis id volutpat molestie. '
    'Donec pharetra massa a dui scelerisque, ut bibendum nunc venenatis. '
    'In in convallis est, vel condimentum est. Integer vehicula in metus quis euismod.\n\n'
    'Aenean ultrices eu leo et suscipit. Donec accumsan lobortis tellus, at volutpat ex. '
    'Duis vel sodales nibh, non tempor nisi. Nullam eu urna tortor. Morbi euismod sodales sem ut luctus. '
    'Mauris id mattis nisl, non lobortis augue. '
    'Vivamus bibendum sodales metus, cursus rutrum enim volutpat non. '
    'Integer ante leo, finibus ut molestie sit amet, sollicitudin et ante. '
    'Ut dictum ac purus vitae suscipit. Cras elementum odio arcu, ac auctor purus dapibus in. '
    'In vel pretium massa, eget interdum nisl. Etiam tincidunt semper orci in condimentum.\n\n'
    'Vivamus facilisis justo sed posuere pharetra. '
    'Pellentesque non libero ac risus hendrerit ullamcorper quis a massa. '
    'Fusce at nisi ac neque euismod imperdiet. '
    'Aliquam purus eros, dapibus at eros quis, feugiat dapibus augue. '
    'Integer vel sapien lacus. Curabitur mattis hendrerit eros. Donec semper in metus ut dictum. '
    'Duis quis vulputate velit. '
    'In mollis, mi sed varius rutrum, ipsum orci accumsan odio, ac tempor velit massa et metus. '
    'Donec tortor lorem, molestie in erat eget, dapibus efficitur erat.'
),
(UNHEX('01000000000000000000000000000013'), 'artist_c', NULL),
(UNHEX('01000000000000000000000000000014'), 'artist_d', 'Description for artist_d, who has no performances.');

-- Songs
INSERT IGNORE INTO songs (id, title, created_by) VALUES
(UNHEX('01000000000000000000000000000021'), 'song_a', UNHEX('01000000000000000000000000000001')),
(UNHEX('01000000000000000000000000000022'), 'song_b', UNHEX('01000000000000000000000000000001')),
(UNHEX('01000000000000000000000000000023'), 'song_c', UNHEX('01000000000000000000000000000001')),
(UNHEX('01000000000000000000000000000024'), 'song_d', UNHEX('01000000000000000000000000000001')),
(UNHEX('01000000000000000000000000000025'), 'song_e', UNHEX('01000000000000000000000000000001')),
(UNHEX('01000000000000000000000000000026'), 'song_f', UNHEX('01000000000000000000000000000001'));

-- Song original artists
INSERT IGNORE INTO song_original_artists (song_id, artist_id) VALUES
(UNHEX('01000000000000000000000000000021'), UNHEX('01000000000000000000000000000011')),
(UNHEX('01000000000000000000000000000022'), UNHEX('01000000000000000000000000000011')),
(UNHEX('01000000000000000000000000000023'), UNHEX('01000000000000000000000000000012')),
(UNHEX('01000000000000000000000000000024'), UNHEX('01000000000000000000000000000012')),
(UNHEX('01000000000000000000000000000025'), UNHEX('01000000000000000000000000000013')),
(UNHEX('01000000000000000000000000000026'), UNHEX('01000000000000000000000000000013'));

-- Song tags
INSERT IGNORE INTO song_tags (song_id, tag_id, kind) VALUES
(UNHEX('01000000000000000000000000000021'), UNHEX('01000000000000000000000000000031'), 'genre'),
(UNHEX('01000000000000000000000000000022'), UNHEX('01000000000000000000000000000031'), 'genre'),
(UNHEX('01000000000000000000000000000023'), UNHEX('01000000000000000000000000000032'), 'source'),
(UNHEX('01000000000000000000000000000024'), UNHEX('01000000000000000000000000000032'), 'source'),
(UNHEX('01000000000000000000000000000025'), UNHEX('01000000000000000000000000000033'), 'language'),
(UNHEX('01000000000000000000000000000026'), UNHEX('01000000000000000000000000000034'), 'genre');

-- Performances (two dates, multiple streams/numbers)
INSERT IGNORE INTO performances (
    id,
    title,
    created_by,
    performance_date,
    stream_number,
    performance_number,
    duration
) VALUES
(UNHEX('01000000000000000000000000000041'), NULL, UNHEX('01000000000000000000000000000001'), '2026-01-1', 1, 1, 1),
(UNHEX('01000000000000000000000000000042'), NULL, UNHEX('01000000000000000000000000000001'), '2026-01-1', 1, 2, 2),
(UNHEX('01000000000000000000000000000043'), NULL, UNHEX('01000000000000000000000000000001'), '2026-01-1', 1, 3, 3),
(UNHEX('01000000000000000000000000000044'), NULL, UNHEX('01000000000000000000000000000001'), '2026-01-2', 1, 1, 4),
(UNHEX('01000000000000000000000000000045'), NULL, UNHEX('01000000000000000000000000000001'), '2026-01-2', 1, 2, 5),
(
    UNHEX('01000000000000000000000000000046'),
    'perf_f_title',
    UNHEX('01000000000000000000000000000001'),
    '2026-01-2',
    1,
    3,
    NULL
),
(UNHEX('01000000000000000000000000000047'), NULL, UNHEX('01000000000000000000000000000001'), '2024-02-03', 1, 1, 7),
(UNHEX('01000000000000000000000000000048'), NULL, UNHEX('01000000000000000000000000000001'), '2024-02-03', 1, 2, 8),
(
    UNHEX('01000000000000000000000000000049'),
    'perf_g_upload',
    UNHEX('01000000000000000000000000000001'),
    '2026-01-2',
    NULL,
    NULL,
    9
),
(
    UNHEX('0100000000000000000000000000004A'),
    'perf_h_upload',
    UNHEX('01000000000000000000000000000001'),
    '2026-01-2',
    NULL,
    NULL,
    10
);

-- Performance songs
INSERT IGNORE INTO performance_songs (performance_id, song_id) VALUES
(UNHEX('01000000000000000000000000000041'), UNHEX('01000000000000000000000000000021')),
(UNHEX('01000000000000000000000000000042'), UNHEX('01000000000000000000000000000022')),
(UNHEX('01000000000000000000000000000043'), UNHEX('01000000000000000000000000000023')),
(UNHEX('01000000000000000000000000000044'), UNHEX('01000000000000000000000000000024')),
(UNHEX('01000000000000000000000000000045'), UNHEX('01000000000000000000000000000025')),
(UNHEX('01000000000000000000000000000046'), UNHEX('01000000000000000000000000000026')),
(UNHEX('01000000000000000000000000000047'), UNHEX('01000000000000000000000000000021')),
(UNHEX('01000000000000000000000000000048'), UNHEX('01000000000000000000000000000022')),
(UNHEX('01000000000000000000000000000049'), UNHEX('01000000000000000000000000000023')),
(UNHEX('0100000000000000000000000000004A'), UNHEX('01000000000000000000000000000024'));

-- Performance singers
INSERT IGNORE INTO performance_singers (performance_id, artist_id) VALUES
(UNHEX('01000000000000000000000000000041'), UNHEX('01000000000000000000000000000011')),
(UNHEX('01000000000000000000000000000042'), UNHEX('01000000000000000000000000000011')),
(UNHEX('01000000000000000000000000000043'), UNHEX('01000000000000000000000000000012')),
(UNHEX('01000000000000000000000000000044'), UNHEX('01000000000000000000000000000012')),
(UNHEX('01000000000000000000000000000045'), UNHEX('01000000000000000000000000000013')),
(UNHEX('01000000000000000000000000000046'), UNHEX('01000000000000000000000000000013')),
(UNHEX('01000000000000000000000000000047'), UNHEX('01000000000000000000000000000011')),
(UNHEX('01000000000000000000000000000048'), UNHEX('01000000000000000000000000000012')),
(UNHEX('01000000000000000000000000000049'), UNHEX('01000000000000000000000000000011')),
(UNHEX('0100000000000000000000000000004A'), UNHEX('01000000000000000000000000000012'));

-- Performance tags
INSERT IGNORE INTO performance_tags (performance_id, tag_id, kind) VALUES
(UNHEX('01000000000000000000000000000041'), UNHEX('01000000000000000000000000000032'), 'instrument'),
(UNHEX('01000000000000000000000000000042'), UNHEX('01000000000000000000000000000033'), 'modifier'),
(UNHEX('01000000000000000000000000000043'), UNHEX('01000000000000000000000000000032'), 'instrument'),
(UNHEX('01000000000000000000000000000047'), UNHEX('01000000000000000000000000000033'), 'modifier');

-- Playlists: favorites for user_a and one user playlist
INSERT IGNORE INTO playlists (id, title, description, kind, is_public, created_by) VALUES
(
    UNHEX('01000000000000000000000000000051'),
    'user_a favorites',
    NULL,
    'favorites',
    FALSE,
    UNHEX('01000000000000000000000000000001')
),
(
    UNHEX('01000000000000000000000000000052'),
    'playlist_a',
    'A sample playlist.',
    'user',
    TRUE,
    UNHEX('01000000000000000000000000000001')
);

-- Playlist entries for playlist_a
INSERT IGNORE INTO playlist_performances (playlist_id, performance_id, sort_order) VALUES
(UNHEX('01000000000000000000000000000052'), UNHEX('01000000000000000000000000000041'), 0),
(UNHEX('01000000000000000000000000000052'), UNHEX('01000000000000000000000000000042'), 1),
(UNHEX('01000000000000000000000000000052'), UNHEX('01000000000000000000000000000043'), 2),
(UNHEX('01000000000000000000000000000052'), UNHEX('01000000000000000000000000000044'), 3);

-- Assets (external URLs)
INSERT IGNORE INTO assets (id, title, external_url) VALUES
(UNHEX('01000000000000000000000000000061'), 'perf_a vod', 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'),
(UNHEX('01000000000000000000000000000062'), 'perf_b vod', 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'),
(UNHEX('01000000000000000000000000000063'), 'perf_c vod', 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'),
(UNHEX('01000000000000000000000000000064'), 'perf_a audio', 'https://www.youtube.com/watch?v=dQw4w9WgXcQ'),
(UNHEX('01000000000000000000000000000065'), 'artist_a avatar', 'https://placecats.com/128/128');

-- Performance videos
INSERT IGNORE INTO performance_videos (performance_id, asset_id, kind) VALUES
(UNHEX('01000000000000000000000000000041'), UNHEX('01000000000000000000000000000061'), 'vod'),
(UNHEX('01000000000000000000000000000042'), UNHEX('01000000000000000000000000000062'), 'vod'),
(UNHEX('01000000000000000000000000000043'), UNHEX('01000000000000000000000000000063'), 'vod');

-- Performance audio
INSERT IGNORE INTO performance_audios (performance_id, asset_id, kind) VALUES
(UNHEX('01000000000000000000000000000041'), UNHEX('01000000000000000000000000000064'), 'primary');

-- Artist images
INSERT IGNORE INTO artist_images (artist_id, asset_id, kind) VALUES
(UNHEX('01000000000000000000000000000011'), UNHEX('01000000000000000000000000000065'), 'avatar');
