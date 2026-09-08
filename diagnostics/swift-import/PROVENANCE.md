# Pinned decoder provenance

Aidoku v0.9/build 3: https://raw.githubusercontent.com/Aidoku/Aidoku/f9836a736ccf477c91df11bbebdafd19086cb9d0/

Runner: cc4d06ff399e7169b9c647bccede7cb29bc805c6. SourceContentRating enum is copied unchanged into a namespace replacing its module. ExternalSourceInfo retains ALL stored properties, including optional Foundation URL sourceUrl, computed fileURL/resolvedContentRating and with(sourceUrl:). Only toInfo() UI conversion is omitted. SourceList.swift is copied whole. No hand-written decoder.

Upstream SHA256:
- ExternalSourceInfo.swift: `550467298e50d354151a7250b37bd4636b5b4bbb21ac468c4414187556525f98`
- SourceList.swift: `5cf66ba15393a9c81c0f186278c85cce3bad5a0e6dd10e0dd69c5ee26a29b6e3`
- LICENSE: `3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986`
- RunnerSourceInfo.swift: `f1acf8024790a72f826abcc4ec99a5be107b369c0aa7400ffcaecd4561340b79`

Harness reproduces modern-first then legacy fallback and default URLSession request timeout 15 seconds. Errors are printed rather than swallowed. macOS Foundation execution is NOT iOS/device proof. No packages or images are downloaded (asset checks use HEAD only).
