# M1186 — Stage 6 / cover Unicode 17 RTL letter scripts

Date: 2026-10-23

Status: Implemented; focused script-direction tests passed. This closes missing explicit script overrides for the Unicode 17 assigned-letter R/AL set; it does not close Stage 6 acceptance.

## Evidence and scope

Compared Unicode 17 `Scripts.txt`, `UnicodeData.txt`, and `extracted/DerivedBidiClass.txt`. Among scripts with assigned Letter-category code points, 37 have letter characters exclusively in bidi classes R or AL. Thirteen were already in the resolver. Added these 24 missing ISO 15924 script codes: `Chrs`, `Cprt`, `Gara`, `Hatr`, `Hung`, `Khar`, `Lydi`, `Mani`, `Mend`, `Merc`, `Mero`, `Narb`, `Nbat`, `Orkh`, `Ougr`, `Palm`, `Phli`, `Phlp`, `Phnx`, `Prti`, `Sarb`, `Sidt`, `Sogd`, and `Sogo`.

The Common script (`Zyyy`) is excluded: its punctuation and format characters include R/AL values, which does not make Common an RTL writing script.

Reproducible witnesses in the Unicode 17 UCD: `Sogo` U+10F00 is an Old Sogdian letter with bidi class R (`Scripts.txt:2896`, `extracted/DerivedBidiClass.txt:1331`, `UnicodeData.txt:19650`); `Sogd` U+10F30 is a distinct letter with bidi class AL (`Scripts.txt:2887`, `extracted/DerivedBidiClass.txt:2494`, `UnicodeData.txt:19690`). Common U+061B demonstrates why punctuation must not select a script direction (`Scripts.txt:94`, `extracted/DerivedBidiClass.txt:2453`, `UnicodeData.txt:1526`). The source files are [Scripts.txt](https://www.unicode.org/Public/17.0.0/ucd/Scripts.txt), [DerivedBidiClass.txt](https://www.unicode.org/Public/17.0.0/ucd/extracted/DerivedBidiClass.txt), and [UnicodeData.txt](https://www.unicode.org/Public/17.0.0/ucd/UnicodeData.txt).

## Change and verification

Added the 24 script subtags to explicit RTL resolution and added a regression that checks each `und-{script}` locale resolves right-to-left. Command passed (exit status 0):

`./scripts/cargo-headstart test -p rustui-i18n --lib script --locked --offline`

## Remaining

Default locale-direction policy, full CLDR/ICU/Fluent behavior, text shaping, and native/browser RTL acceptance remain open.
