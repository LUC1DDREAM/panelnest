import Foundation
import CryptoKit

@main struct Probe {
    static func require(_ value: Bool, _ message: String) throws {
        if !value { throw NSError(domain: "Probe", code: 1, userInfo: [NSLocalizedDescriptionKey: message]) }
    }
    static func fetch(_ url: URL, session: URLSession, head: Bool = false) async throws -> Data {
        var request = URLRequest(url: url)
        request.httpMethod = head ? "HEAD" : "GET"
        let (data, response) = try await session.data(for: request)
        let http = response as! HTTPURLResponse
        print("HTTP \(request.httpMethod!) \(url.absoluteString) status=\(http.statusCode) final=\(http.url!.absoluteString) MIME=\(http.mimeType ?? "nil") bytes=\(data.count)")
        if !head {
            print("SHA256 \(SHA256.hash(data: data).map { String(format: "%02x", $0) }.joined()) prefix=\(String(decoding: data.prefix(160), as: UTF8.self))")
        }
        try require(http.statusCode == 200, "HTTP status \(http.statusCode)")
        return data
    }
    static func load(_ url: URL, session: URLSession) async throws -> SourceList {
        let data = try await fetch(url, session: session)
        do { return try JSONDecoder().decode(CodableSourceList.self, from: data).into(url: url) }
        catch { print("MODERN DECODE ERROR \(error)") }
        let legacyData: Data
        if !url.pathExtension.isEmpty { legacyData = data }
        else { legacyData = try await fetch(url.appendingPathComponent("index.min.json"), session: session) }
        let sources = try JSONDecoder().decode([ExternalSourceInfo].self, from: legacyData)
        return SourceList(url: url, name: "Legacy Source List", sources: sources.map { $0.with(sourceUrl: url) }, legacy: true)
    }
    static func main() async throws {
        print("OS \(ProcessInfo.processInfo.operatingSystemVersionString); native Foundation JSONDecoder and URLSession; NOT iPhone proof")
        let config = URLSessionConfiguration.default
        config.timeoutIntervalForRequest = 15
        let session = URLSession(configuration: config)
        let base = "https://luc1ddream.github.io/my-aidoku-sources/experimental/"
        let urls = [base + "index.min.json", base + "index.json", "https://aidoku-community.github.io/sources/index.min.json"] + Array(CommandLine.arguments.dropFirst())
        var ours: SourceList?
        for (index, text) in urls.enumerated() {
            let url = URL(string: text)!
            let list = try await load(url, session: session)
            print("PASS NATIVE list=\(list.name) count=\(list.sources.count) legacy=\(list.legacy)")
            if index < 2 { try require(list.sources.count == 6, "six-source invariant") }
            if index == 0 { ours = list }
            if index == 1 { try require(list.sources.map { $0.with(sourceUrl: ours!.url) } == ours!.sources, "pretty/minified metadata mismatch") }
            if index == 2 { try require(!list.sources.isEmpty, "Community empty") }
            if index != 2 {
                for source in list.sources {
                    let file = source.fileURL!.absoluteURL
                    let icon = URL(string: source.iconURL!, relativeTo: url)!.absoluteURL
                    print("RESOLVED \(source.id) file=\(file.absoluteString) icon=\(icon.absoluteString)")
                    _ = try await fetch(file, session: session, head: true)
                    _ = try await fetch(icon, session: session, head: true)
                }
            }
        }
        let entry = try JSONSerialization.jsonObject(with: JSONEncoder().encode(ours!.sources[0])) as! [String: Any]
        let invalid: [(String, Any)] = [("contentRating", "safe"), ("contentRating", 3), ("version", "1"), ("languages", [1]), ("sourceUrl", 7)]
        for (key, value) in invalid {
            var changed = entry; changed[key] = value
            let data = try JSONSerialization.data(withJSONObject: ["name": "negative", "sources": [changed]])
            do {
                _ = try JSONDecoder().decode(CodableSourceList.self, from: data)
                throw NSError(domain: "AcceptedInvalidControl", code: 1)
            } catch let error as DecodingError { print("PASS NEGATIVE \(key)=\(value): \(error)") }
        }
        let missing = Data("{\"sources\":[]}".utf8)
        do { _ = try JSONDecoder().decode(CodableSourceList.self, from: missing); throw NSError(domain: "AcceptedMissingName", code: 1) }
        catch let error as DecodingError { print("PASS NEGATIVE missing name: \(error)") }
        for key in ["sourceUrl"] {
            var changed = entry; changed.removeValue(forKey: key)
            let data = try JSONSerialization.data(withJSONObject: ["name": "optional URL", "sources": [changed]])
            let decoded = try JSONDecoder().decode(CodableSourceList.self, from: data)
            try require(decoded.sources[0].sourceUrl == nil, "omitted sourceUrl must decode nil")
            print("PASS OPTIONAL omitted sourceUrl -> nil")
        }
        var components = URLComponents()
        components.scheme = "aidoku"; components.host = "addSourceList"
        components.queryItems = [URLQueryItem(name: "url", value: urls[0])]
        let deepLink = components.url!
        let extracted = URLComponents(url: deepLink, resolvingAgainstBaseURL: false)!.queryItems!.first(where: { $0.name == "url" })!.value!
        try require(URL(string: extracted) == ours!.url, "deep link changed URL")
        print("PASS DEEP LINK roundtrip \(deepLink.absoluteString)")
        do { _ = try await load(URL(string: "https://luc1ddream.github.io/my-aidoku-sources/")!, session: session); throw NSError(domain: "UnexpectedRootSuccess", code: 1) }
        catch let error as DecodingError { print("PASS ROOT REJECTED by legacy array fallback: \(error)") }
        // Own-source, fresh route, minimal optional metadata; generated only after full native pass.
        let selected = ours!.sources.first(where: { $0.id == "en.luc1d-asurascans" })!
        let diagnostic: [String: Any] = ["name": "LUC1D EXPERIMENTAL import diagnostic (one source)", "sources": [["id": selected.id, "name": selected.name, "version": selected.version, "iconURL": URL(string: selected.iconURL!, relativeTo: ours!.url)!.absoluteString, "downloadURL": selected.fileURL!.absoluteURL.absoluteString]]]
        let data = try JSONSerialization.data(withJSONObject: diagnostic, options: [.prettyPrinted, .sortedKeys])
        let decoded = try JSONDecoder().decode(CodableSourceList.self, from: data)
        try require(decoded.sources.count == 1 && decoded.sources[0].id == selected.id, "diagnostic identity")
        try FileManager.default.createDirectory(atPath: "native-evidence", withIntermediateDirectories: true)
        try data.write(to: URL(fileURLWithPath: "native-evidence/one-source.json"))
        print("PASS MINIMAL own-source catalog generated; production unchanged")
        print("ALL NATIVE CHECKS PASSED")
    }
}
