# Schéma de base de données — Partage de fichiers

## Tables

---

### `users`

| Colonne | Type | Contraintes |
|---|---|---|
| id | UUID | 🔑 PK, DEFAULT gen_random_uuid() |
| username | TEXT | UNIQUE NOT NULL |
| password_hash | TEXT | NOT NULL |
| email | TEXT | NOT NULL |
| created_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |
| storage_quota_bytes | BIGINT | NOT NULL DEFAULT 0 |
| is_active | BOOLEAN | NOT NULL DEFAULT true |

---

### `files`

| Colonne | Type | Contraintes |
|---|---|---|
| id | UUID | 🔑 PK |
| owner_id | UUID | 🔗 FK → users(id) |
| filename | TEXT | NOT NULL |
| storage_path | TEXT | NOT NULL |
| size_bytes | BIGINT | NOT NULL |
| mime_type | TEXT | |
| checksum | TEXT | |
| created_at | TIMESTAMPTZ | NOT NULL |
| updated_at | TIMESTAMPTZ | NOT NULL |
| is_deleted | BOOLEAN | NOT NULL DEFAULT false |

---

### `share_links` *(lien anonyme — URL + token)*

| Colonne | Type | Contraintes |
|---|---|---|
| id | UUID | 🔑 PK |
| file_id | UUID | 🔗 FK → files(id) |
| created_by | UUID | 🔗 FK → users(id) |
| token | TEXT | UNIQUE NOT NULL |
| label | TEXT | |
| **— permissions —** | | |
| can_read | BOOLEAN | DEFAULT true |
| can_write | BOOLEAN | DEFAULT false |
| can_reshare | BOOLEAN | DEFAULT false |
| **— limites —** | | |
| max_reads | INT | NULL → lectures illimitées |
| expires_at | TIMESTAMPTZ | NULL → pas d'expiration |
| password_hash | TEXT | NULL → pas de mot de passe |
| is_active | BOOLEAN | DEFAULT true |

---

### `share_grants` *(partage nommé — utilisateur connu)*

| Colonne | Type | Contraintes |
|---|---|---|
| id | UUID | 🔑 PK |
| file_id | UUID | 🔗 FK → files(id) |
| granted_by | UUID | 🔗 FK → users(id) |
| granted_to | UUID | 🔗 FK → users(id) |
| **— permissions —** | | |
| can_read | BOOLEAN | NOT NULL |
| can_write | BOOLEAN | NOT NULL |
| can_reshare | BOOLEAN | NOT NULL |
| **— limites —** | | |
| max_reads | INT | NULL → lectures illimitées |
| expires_at | TIMESTAMPTZ | NULL → pas d'expiration |
| granted_at | TIMESTAMPTZ | NOT NULL DEFAULT now() |

---

### `access_log`

| Colonne | Type | Contraintes |
|---|---|---|
| id | BIGSERIAL | 🔑 PK |
| file_id | UUID | 🔗 FK → files(id) |
| accessed_by | UUID | 🔗 FK → users(id) — NULL |
| share_link_id | UUID | 🔗 FK → share_links(id) — NULL |
| grant_id | UUID | 🔗 FK → share_grants(id) — NULL |
| action | ENUM | `read`, `write`, `share` |
| accessed_at | TIMESTAMPTZ | NOT NULL |
| ip_address | INET | |
| user_agent | TEXT | |
| bytes_transferred | BIGINT | |

---

## Vues

### `read_counters` *(vue matérialisée)*

| Colonne | Type | Notes |
|---|---|---|
| share_link_id | UUID | 🔗 PK |
| grant_id | UUID | 🔗 PK |
| read_count | INT | NOT NULL |
| last_read_at | TIMESTAMPTZ | |
| is_exhausted | BOOLEAN | GENERATED |
| refreshed_at | TIMESTAMPTZ | |

> Se recalcule via **trigger** après chaque INSERT dans `access_log`.

---

### `v_effective_permissions` *(vue SQL)*

Résout les droits effectifs d'un utilisateur sur un fichier.

| Colonne | Type | Notes |
|---|---|---|
| file_id | UUID | |
| user_id | UUID | |
| source | TEXT | `'owner'` \| `'grant'` \| `'link'` |
| can_read | BOOLEAN | |
| can_write | BOOLEAN | |
| can_reshare | BOOLEAN | |
| reads_remaining | INT | NULL = illimité |
| is_expired | BOOLEAN | |
| is_valid | BOOLEAN | non expiré + non épuisé |

---

## Relations

```
users ──< files           (1 utilisateur possède N fichiers)
files ──< share_links     (1 fichier a N liens de partage)
files ──< share_grants    (1 fichier a N partages nommés)
files ──< access_log      (1 fichier a N entrées de journal)
users ──< share_links     (created_by)
users ──< share_grants    (granted_by, granted_to)
share_links ──< access_log
share_grants ──< access_log
```

---

## Légende & règles métier

| Symbole | Signification |
|---|---|
| 🔑 | Clé primaire |
| 🔗 | Clé étrangère |
| NULL | Valeur facultative |

- `max_reads NULL` → lectures illimitées
- `expires_at NULL` → pas d'expiration
- `password_hash NULL` → pas de mot de passe
- `share_grants` = partage nommé (utilisateur connu)
- `share_links` = lien anonyme (URL + token)
- `read_counters` se recalcule via trigger après chaque INSERT dans `access_log`
