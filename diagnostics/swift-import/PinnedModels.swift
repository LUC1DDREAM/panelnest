// Aidoku v0.9 f9836a736ccf477c91df11bbebdafd19086cb9d0
// AidokuRunner cc4d06ff399e7169b9c647bccede7cb29bc805c6
// Unchanged Codable fields, synthesized decoder and URL resolution.
// Namespace wrapper replaces module import; non-decoding toInfo() omitted.
import Foundation
enum AidokuRunner {
public enum SourceContentRating: Int, Sendable, Codable, CaseIterable {
    case safe = 0
    case containsNsfw = 1
    case primarilyNsfw = 2
}

}
//
//  ExternalSourceInfo.swift
//  Aidoku
//
//  Created by Skitty on 1/16/22.
//

import Foundation

struct ExternalSourceInfo: Codable, Hashable {
    let id: String
    let name: String
    let version: Int
    let iconURL: String?
    let downloadURL: String?
    let languages: [String]?
    let contentRating: AidokuRunner.SourceContentRating?
    let altNames: [String]?
    let baseURL: String?
    let minAppVersion: String?
    let maxAppVersion: String?

    // deprecated
    let lang: String?
    let nsfw: Int?
    let file: String?
    let icon: String?

    var sourceUrl: URL?

    var fileURL: URL? {
        sourceUrl.flatMap { sourceUrl in
            if let downloadURL {
                URL(string: downloadURL, relativeTo: sourceUrl)
            } else if let file {
                URL(string: "sources/\(file)", relativeTo: sourceUrl)
            } else {
                nil
            }
        }
    }

    var resolvedContentRating: AidokuRunner.SourceContentRating {
        if let contentRating {
            contentRating
        } else if let nsfw, let rating = AidokuRunner.SourceContentRating(rawValue: nsfw) {
            rating
        } else {
            .safe
        }
    }
}

extension ExternalSourceInfo {
    func with(sourceUrl: URL) -> ExternalSourceInfo {
        var copy = self
        copy.sourceUrl = sourceUrl
        return copy
    }

}
//
//  SourceList.swift
//  Aidoku
//
//  Created by Skitty on 6/11/25.
//

import Foundation

struct SourceList: Equatable {
    let url: URL
    let name: String
    var feedbackURL: URL?
    let sources: [ExternalSourceInfo]
    var legacy: Bool = false

    static func == (lhs: Self, rhs: Self) -> Bool {
        lhs.url == rhs.url
    }
}

struct CodableSourceList: Codable {
    let name: String
    let feedbackURL: String?
    let sources: [ExternalSourceInfo]

    func into(url: URL) -> SourceList {
        .init(
            url: url,
            name: name,
            feedbackURL: feedbackURL.flatMap { URL(string: $0) },
            sources: sources.map {
                $0.with(sourceUrl: url)
            }
        )
    }
}
