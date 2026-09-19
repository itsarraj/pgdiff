# pgdiff

Compares two live Postgres schemas and generates the SQL to reconcile
them — dev vs. prod, or two branches' migration state. Python's `migra`
is the closest existing tool (or paid SaaS); no Rust equivalent.

## Usage

```bash
pgdiff postgres://user@host/old_db postgres://user@host/new_db
pgdiff --schema myschema postgres://.../old postgres://.../new
```

Prints the SQL to go from the first database's schema to the second's —
pipeable straight into `psql` once you've reviewed it, which is exactly
how the live verification below was actually run.

## How it works

Introspects both databases via `information_schema.columns` (the
standard, cross-version-portable catalog view — not `pg_catalog`
directly) rather than parsing `pg_dump` output, matching how `migra`
itself works: live schema comparison, not text diffing.

## Status: built, and the full round trip verified against two real live databases — generate, apply, and confirm actual convergence

- **11 unit tests** (`cargo test --lib`): every diff category (table
  added/removed, column added/removed, type changed, nullability
  changed) and every SQL rendering rule (a nullable column doesn't get a
  spurious `NOT NULL`, `DEFAULT` values rendered correctly, `SET NOT
  NULL` vs. `DROP NOT NULL` chosen correctly in each direction) — all
  pure, no database needed for this half.
- **The full real round trip, not just "the diff looks plausible"**: spun
  up a real local Postgres cluster (no Docker — `initdb`/`pg_ctl`, same
  as this workspace's `pgqueue`), created two databases with genuinely
  different schemas covering every diff category at once (a dropped
  table, a new table with a real `DEFAULT` value, a removed column, an
  added column, a nullability change), and:
  1. Ran `pgdiff` for real — correctly reported all 5 changes, including
     rendering Postgres's own exact stored default literal
     (`DEFAULT 'x'::text`, the real catalog value, not a guess).
  2. **Piped the generated SQL directly into `psql` against the "old"
     database** — every statement executed cleanly.
  3. **Ran `pgdiff` again on the now-migrated database** — `-- no
     differences found`. The schemas genuinely converged, not just "the
     SQL looked right when read."

**Not done / deliberately deferred**: indexes, constraints (primary
keys, foreign keys, `CHECK`), and sequences — this only diffs columns,
which is the most common day-to-day drift but a real, meaningful subset
of a full schema; views, functions, and triggers (same reasoning);
multi-schema comparison in one run (`--schema` is one schema per
invocation).
