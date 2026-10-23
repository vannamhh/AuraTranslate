import AppKit
import WebKit

let dir = URL(fileURLWithPath: CommandLine.arguments[1])
let variants = ["subgrid", "chunk", "rows"]
let app = NSApplication.shared
app.setActivationPolicy(.regular)
let window = NSWindow(contentRect: NSRect(x: 80, y: 80, width: 1200, height: 900), styleMask: [.titled], backing: .buffered, defer: false)
let web = WKWebView(frame: window.contentView!.bounds)
window.contentView!.addSubview(web)
window.makeKeyAndOrderFront(nil)
app.activate(ignoringOtherApps: true)
let bundle = Bundle(for: WKWebView.self)
print("{\"wkwebview\":\"\(bundle.bundlePath)\",\"version\":\"\(bundle.object(forInfoDictionaryKey: "CFBundleVersion") ?? "?")\"}")

var index = 0
func load() {
  let url = URL(string: "grid.html?v=\(variants[index])&n=9850&auto=1", relativeTo: dir.appendingPathComponent("grid.html"))!
  web.loadFileURL(url, allowingReadAccessTo: dir)
}
Timer.scheduledTimer(withTimeInterval: 0.5, repeats: true) { _ in
  web.evaluateJavaScript("document.title") { value, _ in
    guard let title = value as? String, title.hasPrefix("DONE:") else { return }
    print("{\"engine\":\"system-wkwebview\"," + title.dropFirst(6))
    fflush(stdout)
    index += 1
    if index == variants.count { exit(0) }
    load()
  }
}
load()
app.run()
