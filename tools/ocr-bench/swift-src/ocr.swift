/// Omni-Glass OCR Helper — Apple Vision Framework text recognition.
///
/// Usage: ./ocr-helper <image_path>
/// Output: JSON with extracted text, character count, and latency.
///
/// This is a standalone CLI compiled with:
///   swiftc -O -o ocr-helper swift-src/ocr.swift
///
/// Production integration will use swift-bridge for direct Rust↔Swift FFI.

import Foundation
import Vision
import CoreGraphics
import UniformTypeIdentifiers

struct OcrResult: Codable {
    let text: String
    let charCount: Int
    let latencyMs: Double
    let confidence: Double
    let recognitionLevel: String
}

struct OcrError: Codable {
    let error: String
}

func recognizeText(imagePath: String, level: VNRequestTextRecognitionLevel) -> OcrResult? {
    guard let imageURL = URL(string: "file://\(imagePath)") ?? URL(fileURLWithPath: imagePath) as URL? else {
        let err = OcrError(error: "Invalid image path: \(imagePath)")
        if let data = try? JSONEncoder().encode(err) {
            FileHandle.standardError.write(data)
        }
        return nil
    }

    guard let imageSource = CGImageSourceCreateWithURL(imageURL as CFURL, nil),
          let cgImage = CGImageSourceCreateImageAtIndex(imageSource, 0, nil) else {
        let err = OcrError(error: "Failed to load image: \(imagePath)")
        if let data = try? JSONEncoder().encode(err) {
            FileHandle.standardError.write(data)
        }
        return nil
    }

    let startTime = CFAbsoluteTimeGetCurrent()

    var recognizedText = ""
    var totalConfidence: Double = 0.0
    var observationCount = 0

    let request = VNRecognizeTextRequest { request, error in
        guard error == nil,
              let observations = request.results as? [VNRecognizedTextObservation] else {
            return
        }

        for observation in observations {
            guard let candidate = observation.topCandidates(1).first else { continue }
            recognizedText += candidate.string + "\n"
            totalConfidence += Double(candidate.confidence)
            observationCount += 1
        }
    }

    request.recognitionLevel = level
    request.usesLanguageCorrection = true
    request.automaticallyDetectsLanguage = true

    let handler = VNImageRequestHandler(cgImage: cgImage, options: [:])

    do {
        try handler.perform([request])
    } catch {
        let err = OcrError(error: "Vision request failed: \(error.localizedDescription)")
        if let data = try? JSONEncoder().encode(err) {
            FileHandle.standardError.write(data)
        }
        return nil
    }

    let elapsed = (CFAbsoluteTimeGetCurrent() - startTime) * 1000.0
    let avgConfidence = observationCount > 0 ? totalConfidence / Double(observationCount) : 0.0
    let levelName = level == .accurate ? "accurate" : "fast"

    // Trim trailing newline
    let trimmed = recognizedText.trimmingCharacters(in: .whitespacesAndNewlines)

    return OcrResult(
        text: trimmed,
        charCount: trimmed.count,
        latencyMs: elapsed,
        confidence: avgConfidence,
        recognitionLevel: levelName
    )
}

// --- Main ---

guard CommandLine.arguments.count >= 2 else {
    let usage = "Usage: ocr-helper <image_path> [--fast]\n"
    FileHandle.standardError.write(usage.data(using: .utf8)!)
    exit(1)
}

let imagePath = CommandLine.arguments[1]
let useFast = CommandLine.arguments.contains("--fast")
let level: VNRequestTextRecognitionLevel = useFast ? .fast : .accurate

// Resolve to absolute path
let absolutePath = (imagePath as NSString).expandingTildeInPath
let resolvedPath: String
if absolutePath.hasPrefix("/") {
    resolvedPath = absolutePath
} else {
    resolvedPath = FileManager.default.currentDirectoryPath + "/" + absolutePath
}

guard FileManager.default.fileExists(atPath: resolvedPath) else {
    let err = OcrError(error: "File not found: \(resolvedPath)")
    if let data = try? JSONEncoder().encode(err) {
        FileHandle.standardOutput.write(data)
    }
    exit(1)
}

if let result = recognizeText(imagePath: resolvedPath, level: level) {
    let encoder = JSONEncoder()
    encoder.outputFormatting = .prettyPrinted
    if let data = try? encoder.encode(result) {
        FileHandle.standardOutput.write(data)
        print("") // trailing newline
    }
} else {
    exit(1)
}
