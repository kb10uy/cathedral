CREATE TABLE "versions" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "name" TEXT NOT NULL,
    "number" INTEGER NOT NULL,
    "abbrev" TEXT NOT NULL
);

CREATE TABLE "songs" (
    "id" INTEGER PRIMARY KEY AUTOINCREMENT,
    "version_id" INTEGER NOT NULL REFERENCES "versions"("id"),
    "genre" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "artist" TEXT NOT NULL,
    "min_bpm" INTEGER NULL,
    "max_bpm" INTEGER NOT NULL,
    "unlock_info" TEXT NULL
);

CREATE TABLE "play_sides" (
    "side" TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE "difficulties" (
    "name" TEXT NOT NULL PRIMARY KEY,
    "abbrev" TEXT NOT NULL,
    "color" TEXT NOT NULL
);

CREATE TABLE "cn_types" (
    "type" TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE "bss_types" (
    "type" TEXT NOT NULL PRIMARY KEY
);

CREATE TABLE "diffs" (
    "song_id" INTEGER NOT NULL REFERENCES "songs"("id"),
    "play_side" INTEGER NOT NULL REFERENCES "play_sides"("side"),
    "difficulty" INTEGER NOT NULL REFERENCES "difficulties"("name"),
    "level" INTEGER NOT NULL,
    "notes" INTEGER NULL,
    "cn_type" TEXT NULL REFERENCES "cn_types"("type"),
    "bss_type" TEXT NULL REFERENCES "bss_types"("type")
);

INSERT INTO "play_sides" ("side")
VALUES
    ('SP'),
    ('DP');

INSERT INTO "difficulties" ("name", "abbrev", "color")
VALUES
    ('BEGINNER', 'B', '#2eff7e'),
    ('NORMAL', 'N', '#2e7bff'),
    ('HYPER', 'H', '#2eff7e'),
    ('ANOTHER', 'A', '#ff2e2e'),
    ('LEGGENDARIA', 'L', '#bd2eff');

INSERT INTO "cn_types" ("type")
VALUES
    ('CN'),
    ('HCN');

INSERT INTO "bss_types" ("type")
VALUES
    ('BSS'),
    ('HBSS'),
    ('MSS');
