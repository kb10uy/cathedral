CREATE TABLE "versions" (
    "id" SERIAL PRIMARY KEY,
    "name" TEXT NOT NULL,
    "abbrev" TEXT NOT NULL
);

CREATE TABLE "songs" (
    "id" SERIAL PRIMARY KEY,
    "version_id" INTEGER NOT NULL REFERENCES "versions"("id"),
    "genre" TEXT NOT NULL,
    "title" TEXT NOT NULL,
    "artist" TEXT NOT NULL,
    "min_bpm" INTEGER NULL,
    "max_bpm" INTEGER NOT NULL,
    "unlock_info" TEXT NULL
);

CREATE TABLE "diffs" (
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

CREATE TABLE "scores" (
    "song_id" INTEGER NOT NULL REFERENCES "songs"("id"),
    "diff" INTEGER NOT NULL REFERENCES "diffs"("name"),
    "level" INTEGER NOT NULL,
    "notes" INTEGER NULL,
    "cn_type" TEXT NULL REFERENCES "cn_types"("type"),
    "bss_type" TEXT NULL REFERENCES "bss_types"("type")
);

INSERT INTO "diffs" ("name", "abbrev", "color")
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
