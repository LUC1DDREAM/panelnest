# Aidoku 0.9 build 3: language filter regression

## Pinned evidence

- [SourceLanguage.swift, v0.9](https://github.com/Aidoku/Aidoku/blob/v0.9/Aidoku/Core/Sources/SourceLanguage.swift): `static let multi = "multi"`; display grouping of arrays with more than one language uses `multi`.
- [AddSourceFilterMenu.swift, v0.9](https://github.com/Aidoku/Aidoku/blob/v0.9/Aidoku/Features/Browse/AddSource/AddSourceFilterMenu.swift): the multilingual menu entry uses `SourceLanguage.multi`. It removes `All` from the menu-language list, but does not alias `All` to `multi` in source metadata.
- [AddSourceView.swift, v0.9](https://github.com/Aidoku/Aidoku/blob/v0.9/Aidoku/Features/Browse/AddSource/AddSourceView.swift), `filterExternalSources`, uses exact membership:

  ```swift
  selectedLanguages.contains(where: { info.languages?.contains($0) ?? (info.lang == $0) })
  ```

  This does not use the display-group primary code. Therefore `['All']` fails selections `{'multi','en'}`. `['multi']` passes multilingual, not English-only. Multiple actual language codes alone do not match multilingual-only in this app version. Built-in sources in this same view use `languages: ["multi"]`.
- [SDK/CLI source manifest schema, pinned e1320b0](https://github.com/Aidoku/aidoku-rs/blob/e1320b0a2e11afb59e4dee374883a2212d325699/crates/cli/src/supporting/schema/source.schema.json): languages are unique strings, no enum/pattern whitelist. Documentation describes ISO 639-1 for individual languages; the application's supported `multi` sentinel is accepted by the actual pinned CLI schema validator. The ID-description's `mult` example is not the app's language-filter sentinel and is no reason to change IDs.
- Build correspondence (v0.9 release IPA build 3) and pinned Runner `cc4d06ff399e7169b9c647bccede7cb29bc805c6` are recorded in `diagnostics/swift-import/PROVENANCE.md`.

## Correction and scope

IMHentai and HentaiFox change only `info.languages: ["All"]` to `["multi"]` and integer package version 1 to 2. This preserves their existing multilingual classification without asserting unverified English-specific support. IDs and adult rating 2 stay unchanged. The other four manifests are unchanged: nhentai still matches through its existing `en`, WEBTOON/AsuraScans/WeebCentral through `en`.

All six remain experimental, with `runtime_tested: false`. IMHentai's prior HTTP 403 is unresolved; this metadata fix does not verify site access, gallery parsing, image rendering or physical iPhone behavior. No website content/image requests are required by this fix.

## Regression gates

`python -m unittest discover -s rebuild -p test_pipeline.py -v` first failed with exactly the two missing IDs, then passed after correction. Controls prove legacy `All` and uppercase `MULTI` fail exact matching; `multi` is not an English-only match; absent-language legacy fallback and empty-language behavior are covered. Tests also guard catalog/package/source agreement for ID, name, version, languages and rating.

The full local pipeline rebuilds/tests/verifies all six packages, validates agreement against source manifests and generated catalog, and preserves experimental segregation. The native macOS probe additionally decodes all six local manifests with pinned Codable models and executes the app's exact language predicate with negative controls. Neither probe is iPhone UI confirmation.
