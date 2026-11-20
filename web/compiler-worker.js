import initialize, { compile_page } from "./playground/compiler/pagescript.js";

try {
  await initialize();
  self.addEventListener("message", ({ data }) => {
    try {
      self.postMessage({ type: "result", result: JSON.parse(compile_page(data)) });
    } catch {
      self.postMessage({
        type: "result",
        result: { status: "render_error", message: "The compiler could not render this source." }
      });
    }
  });
  self.postMessage({ type: "ready" });
} catch {
  self.postMessage({ type: "startup_error" });
}
