import AppKit
import Foundation

// Renders real Apple SF Symbols to PNG files as alpha-only masks (solid
// black glyph on a transparent background) so the web UI can use them via
// CSS `mask-image` + `background-color: currentColor` — that keeps them
// tintable (hover states, dark/light mode) exactly like native SF Symbols
// behave in AppKit, rather than baking in one fixed color.
let symbols: [(name: String, symbol: String)] = [
    ("pencil", "pencil"),
    ("trash", "trash"),
    ("plus", "plus"),
    ("xmark", "xmark"),
    ("chevron-left", "chevron.left"),
    ("photo", "photo"),
]

// Resolve output dir relative to this script's own location (scripts/../src/assets/icons)
// so the script works regardless of where the repo is checked out.
let scriptDir = URL(fileURLWithPath: #filePath).deletingLastPathComponent()
let outDir = scriptDir.appendingPathComponent("../src/assets/icons").standardizedFileURL.path
try? FileManager.default.createDirectory(atPath: outDir, withIntermediateDirectories: true)

let pointSize: CGFloat = 128
let config = NSImage.SymbolConfiguration(pointSize: pointSize, weight: .regular)

for (name, symbol) in symbols {
    guard let base = NSImage(systemSymbolName: symbol, accessibilityDescription: nil),
          let sized = base.withSymbolConfiguration(config) else {
        FileHandle.standardError.write("FAILED to load symbol: \(symbol)\n".data(using: .utf8)!)
        continue
    }

    let size = sized.size
    let scale: CGFloat = 2 // export at 2x for crispness
    let pixelWidth = Int(size.width * scale)
    let pixelHeight = Int(size.height * scale)

    guard let rep = NSBitmapImageRep(
        bitmapDataPlanes: nil,
        pixelsWide: pixelWidth,
        pixelsHigh: pixelHeight,
        bitsPerSample: 8,
        samplesPerPixel: 4,
        hasAlpha: true,
        isPlanar: false,
        colorSpaceName: .deviceRGB,
        bytesPerRow: 0,
        bitsPerPixel: 0
    ) else {
        FileHandle.standardError.write("FAILED to create bitmap for: \(symbol)\n".data(using: .utf8)!)
        continue
    }
    rep.size = size

    NSGraphicsContext.saveGraphicsState()
    NSGraphicsContext.current = NSGraphicsContext(bitmapImageRep: rep)
    NSColor.black.set()
    let rect = NSRect(origin: .zero, size: size)
    sized.isTemplate = true
    sized.draw(in: rect, from: .zero, operation: .sourceOver, fraction: 1.0)
    // Force pure black fill regardless of template rendering: composite
    // solid black using the drawn shape as the alpha mask.
    NSGraphicsContext.current?.cgContext.setBlendMode(.sourceIn)
    NSColor.black.setFill()
    rect.fill()
    NSGraphicsContext.restoreGraphicsState()

    guard let data = rep.representation(using: .png, properties: [:]) else {
        FileHandle.standardError.write("FAILED to encode PNG for: \(symbol)\n".data(using: .utf8)!)
        continue
    }

    let path = "\(outDir)/\(name).png"
    try? data.write(to: URL(fileURLWithPath: path))
    print("wrote \(path) (\(pixelWidth)x\(pixelHeight))")
}
