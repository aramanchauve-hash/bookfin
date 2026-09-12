-- Bookfin Development Seed (seeds/development.sql)
-- Exécutable explicitement via : psql -U bookfin -d bookfin -f seeds/development.sql

-- 1. Utilisateur fixe de développement
INSERT INTO users (id, created_at)
VALUES ('00000000-0000-0000-0000-000000000001', NOW())
ON CONFLICT (id) DO NOTHING;

-- 2. Préférences linguistiques ordonnées (multiples langues)
INSERT INTO user_language_preferences (user_id, language_tag, priority, created_at)
VALUES 
    ('00000000-0000-0000-0000-000000000001', 'en', 1, NOW()),
    ('00000000-0000-0000-0000-000000000001', 'fr', 2, NOW()),
    ('00000000-0000-0000-0000-000000000001', 'es', 3, NOW()),
    ('00000000-0000-0000-0000-000000000001', 'de', 4, NOW()),
    ('00000000-0000-0000-0000-000000000001', 'ja', 5, NOW())
ON CONFLICT (user_id, language_tag) DO NOTHING;

-- 3. Livres et extraits
-- Livre 1 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111101', 'The Whispering Pines', 'Eleanor Vance', 'en', 1924, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222201', '11111111-1111-1111-1111-111111111101', 'The fog drifted between the tall pines like ancient breath, curling around the stone stairs that led nowhere.', 'en', 19, 0.02, NOW()),
    ('22222222-2222-2222-2222-222222222202', '11111111-1111-1111-1111-111111111101', 'A single lantern flickered in the watchtower, trembling against the cold mountain wind.', 'en', 13, 0.06, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 2 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111102', 'Shadows over Greenwich', 'Arthur Pendelton', 'en', 1888, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222203', '11111111-1111-1111-1111-111111111102', 'Clocks in London struck eleven in discordant chorus, muffled by the heavy yellow river mist.', 'en', 16, 0.10, NOW()),
    ('22222222-2222-2222-2222-222222222204', '11111111-1111-1111-1111-111111111102', 'He traced the watermark on the parchment, noticing the subtle seal of the Admiralty.', 'en', 14, 0.14, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 3 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111103', 'The Silicon Horizon', 'Mira Thorne', 'en', 2018, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222205', '11111111-1111-1111-1111-111111111103', 'In the quiet desert night, data centers humming with synthetic thought cast amber halos toward Orion.', 'en', 16, 0.18, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 4 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111104', 'Letters to the Sea', 'Julian Croft', 'en', 1952, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222206', '11111111-1111-1111-1111-111111111104', 'The tide retreated, abandoning fragments of green glass polished smoother than emeralds.', 'en', 12, 0.22, NOW()),
    ('22222222-2222-2222-2222-222222222207', '11111111-1111-1111-1111-111111111104', 'Every sailor carries two maps: one printed on oilcloth, the other etched in sleepless memory.', 'en', 15, 0.26, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 5 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111105', 'Echoes of the North', 'Bridget O''Connor', 'en', 1976, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222208', '11111111-1111-1111-1111-111111111105', 'The peat fire crackled, warming hands that had spent decades hauling nets across freezing waters.', 'en', 15, 0.30, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 6 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111106', 'The Clockmaker of Prague', 'Karel Vaneck', 'en', 1935, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222209', '11111111-1111-1111-1111-111111111106', 'Behind the silver gears, tiny brass dancers waited for the strike of noon to begin their perpetual waltz.', 'en', 18, 0.34, NOW()),
    ('22222222-2222-2222-2222-222222222210', '11111111-1111-1111-1111-111111111106', 'Escapements clicked like heartbeat cadences in the dimly lit workshop.', 'en', 10, 0.38, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 7 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111107', 'Dust and Constellations', 'Sarah K. Adams', 'en', 2004, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222211', '11111111-1111-1111-1111-111111111107', 'Telescopes pointed at the zenith revealed galaxies spinning like silver spirals in bottomless ink.', 'en', 14, 0.42, NOW()),
    ('22222222-2222-2222-2222-222222222212', '11111111-1111-1111-1111-111111111107', 'We measure light that began traveling when mountains were still seabed.', 'en', 11, 0.46, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 8 (en)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111108', 'Winter at Heron Hall', 'Thomas Sterling', 'en', 1912, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222213', '11111111-1111-1111-1111-111111111108', 'Snow sealed the tall windows in lace patterns of ice, silencing the grand corridors.', 'en', 14, 0.50, NOW()),
    ('22222222-2222-2222-2222-222222222214', '11111111-1111-1111-1111-111111111108', 'Footsteps vanished under the falling snow within mere minutes.', 'en', 9, 0.54, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 9 (fr)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111109', 'Les Chemins d''Automne', 'Camille Delacroix', 'fr', 1968, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222215', '11111111-1111-1111-1111-111111111109', 'Les feuilles rousses tourbillonnaient sur les pavés humides, emportant avec elles l''odeur sucrée du raisin fermenté.', 'fr', 17, 0.58, NOW()),
    ('22222222-2222-2222-2222-222222222216', '11111111-1111-1111-1111-111111111109', 'Au loin, le clocher du village égrenait les heures avec une lenteur rassurante.', 'fr', 13, 0.62, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 10 (fr)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111110', 'L''Ombre du Marais', 'Henri de Saint-Pol', 'fr', 1895, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222217', '11111111-1111-1111-1111-111111111110', 'Une barque glissait sans bruit entre les roseaux denses, fendant l''eau noire comme un miroir liquide.', 'fr', 17, 0.66, NOW()),
    ('22222222-2222-2222-2222-222222222218', '11111111-1111-1111-1111-111111111110', 'Le héron immobile veillait sur l''aurore naissante, statue de plumes argentées.', 'fr', 12, 0.70, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 11 (fr)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111111', 'Le Dernier Phare', 'Élise Marceau', 'fr', 1984, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222219', '11111111-1111-1111-1111-111111111111', 'La tempête frappait la falaise de granit, projetant des gerbes d''écume blanche jusqu''au sommet de la tour.', 'fr', 18, 0.74, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 12 (es)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111112', 'El Viento del Sur', 'Mateo Silva', 'es', 1973, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222220', '11111111-1111-1111-1111-111111111112', 'El viento soplaba caliente a través de los olivares, levantando polvo dorado entre las colinas.', 'es', 15, 0.78, NOW()),
    ('22222222-2222-2222-2222-222222222221', '11111111-1111-1111-1111-111111111112', 'Las campanas sonaban a siesta, mientras los patios descansaban a la sombra del jazmín.', 'es', 14, 0.82, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 13 (de)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111113', 'Stimmen im Wald', 'Lukas Weber', 'de', 1981, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222222', '11111111-1111-1111-1111-111111111113', 'Im dichten Tannenwald brach die Abenddämmerung an, und der Moosboden dämpfte jeden Schritt.', 'de', 14, 0.86, NOW()),
    ('22222222-2222-2222-2222-222222222223', '11111111-1111-1111-1111-111111111113', 'Ein kalter Bach murmelte zwischen bemoosten Granitblöcken dem Tal entgegen.', 'de', 10, 0.90, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 14 (ja - Non-Latin)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111114', '雪の庭 (Snow Garden)', 'Kenji Sato', 'ja', 1962, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222224', '11111111-1111-1111-1111-111111111114', '雪は静かに庭の石灯籠の上に降り積もり、夜の静寂をいっそう深くしていた。', 'ja', 34, 0.94, NOW())
ON CONFLICT (id) DO NOTHING;

-- Livre 15 (el - Non-Latin)
INSERT INTO books (id, title, author, original_language_tag, publication_year, created_at)
VALUES ('11111111-1111-1111-1111-111111111115', 'Το Φως του Αιγαίου (The Light of the Aegean)', 'Elena Pappas', 'el', 1999, NOW())
ON CONFLICT (id) DO NOTHING;

INSERT INTO extracts (id, book_id, content, language_tag, token_count, random_key, created_at)
VALUES 
    ('22222222-2222-2222-2222-222222222225', '11111111-1111-1111-1111-111111111115', 'Ο ήλιος έλουζε τα λευκά σπίτια του νησιού με ένα φως τόσο δυνατό που πονούσαν τα μάτια.', 'el', 17, 0.98, NOW())
ON CONFLICT (id) DO NOTHING;
