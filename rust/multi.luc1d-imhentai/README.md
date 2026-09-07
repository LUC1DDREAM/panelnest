# imhentai (LUC1D)
Independent current Aidoku API implementation. SDK pinned to `e1320b0a2e11afb59e4dee374883a2212d325699`.

Implemented: query search / latest browsing with pagination, title/cover/artist/tag metadata, one gallery chapter, numerically ordered pages from the complete public JSON reader manifest, image Referer headers, one request/second.

## Verification
Run `cargo test --locked`, `cargo build --release --locked`, `aidoku package .`, `aidoku verify package.aix` here. Synthetic non-explicit WASM fixtures cover search, URL escaping, details/chapters flags, malformed/empty pages, image extensions and numeric ordering. No content/images downloaded by tests.

## Limitations
No advanced filter UI, language filter, account/favorites, home/listings, deep links or dates. No thumbnail guessing fallback: incomplete/missing reader JSON is an explicit error. Site layout changes can require updates. IMHentai returned HTTP 403 during public access check; no bypass attempted. HentaiFox public homepage and gallery metadata schema returned HTTP 200; this is NOT Aidoku device/reader verification. Keep runtime_tested=false and publish=false until authorized on-device checks.

## Provenance
New Rust implementation referencing public endpoint/schema behavior in Keiyoushi extensions-source `lib-multisrc/galleryadults/.../GalleryAdults.kt`, `src/all/imhentai/.../IMHentai.kt`, and `src/all/hentaifox/.../HentaiFox.kt`. Apache-2.0 license preserved in LICENSE. Legacy Aidoku HentaiFox inspected, not copied. Original neutral slate icon generated locally. This is not a repackaged upstream AIX.
