# Supplemental dependency notices

These are verbatim notice files from exact upstream revisions. They supplement
notice material omitted from published package archives. `manifest.json` binds
each package name/version and Cargo source to its vendored Cargo manifest hash,
upstream repository/revision, original file URL and file SHA-256. Text files are
deduplicated by content hash. No dependency or vendored source is changed.

The current 40 mappings cover 18 distinct files. Four verbatim MIT/Apache
notices for Fluent Langneg 0.13.1 and Intl Pluralrules 7.0.2 were added at their
crate-recorded exact revisions; two distinct texts reuse existing content hashes.
The initial 38 mappings covered 16 distinct files. The imported revision was checked
against each package's `.cargo_vcs_info.json`. The two Foundation packages use the
repository's notice at their own recorded revision. Zune's Zlib text also matched
a fresh download from its exact-revision URL. These checks establish provenance;
they do not establish license sufficiency or compatibility.

Some exact revisions contain licensing guidance or links instead of full license
texts. Those mappings retain an explicit review note. Zune's captured Zlib text
does not supply its missing MIT/Apache alternatives. The manifest does not choose
distribution license alternatives or resolve Apple SDK derivation concerns.

Run the collector from the repository root after retaining successful Cargo build
JSON and locked/offline metadata:

```sh
./scripts/cargo-headstart metadata --locked --offline --format-version 1 > target/headstart/evidence/release-checks/metadata.json
python3 scripts/collect_dependency_notices.py \
  --repo-root . \
  --metadata target/headstart/evidence/release-checks/metadata.json \
  --build-json target/headstart/evidence/release-checks/native-build.jsonl \
  --supplement-manifest docs/release/dependency-notices/manifest.json \
  --supplement-root docs/release/dependency-notices \
  --output target/headstart/evidence/dependency-notices-new
```

The output must be new. The collector never replaces existing output. It rejects
source/output overlap, source symlinks, identity/hash mismatches and inputs that
change during collection. It reports packages observed in the successful build,
including build dependencies, rather than claiming an exact linked runtime
closure. `report.json` records captured files, provenance and review notes. A
package with captured files can still lack sufficient license or notice material.

The retained 2026-10-25 native build initially had 43 packages without captured
notice files. Supplementation reduced that count to five: `block 0.1.6`,
`mac 0.1.1`, `malloc_buf 0.0.6`, `pathfinder_geometry 0.5.1` and `seahash 4.1.0`.
Exact tagged sources for four of these lacked standalone texts; the recorded
Pathfinder geometry source revision could not be retrieved. Full-text and
compatibility reviews, MPL source obligations, runtime/build-only classification
and actual distribution packaging remain open. RustUI is not cleared for release
by this collection.
