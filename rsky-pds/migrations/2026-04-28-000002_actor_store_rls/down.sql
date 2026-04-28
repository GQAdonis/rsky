-- Remove RLS policies and disable RLS on per-actor tables.

DROP POLICY IF EXISTS blob_actor_isolation ON pds.blob;
ALTER TABLE pds.blob DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS record_actor_isolation ON pds.record;
ALTER TABLE pds.record DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS record_blob_actor_isolation ON pds.record_blob;
ALTER TABLE pds.record_blob DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS repo_block_actor_isolation ON pds.repo_block;
ALTER TABLE pds.repo_block DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS repo_root_actor_isolation ON pds.repo_root;
ALTER TABLE pds.repo_root DISABLE ROW LEVEL SECURITY;

DROP POLICY IF EXISTS account_pref_actor_isolation ON pds.account_pref;
ALTER TABLE pds.account_pref DISABLE ROW LEVEL SECURITY;
