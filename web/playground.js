const form = document.querySelector("#source-form");
const source = document.querySelector("#source");
const editor = document.querySelector("#editor");
const highlight = document.querySelector("#highlight");
const lineNumbers = document.querySelector("#line-numbers");
const render = document.querySelector("#render");
const previewNote = document.querySelector("#preview-note");
const renderAnnouncement = document.querySelector("#render-announcement");
const diagnostics = document.querySelector("#diagnostics");
const diagnosticList = document.querySelector("#diagnostic-list");
const preview = document.querySelector("#preview");
const download = document.querySelector("#download");
const prompt = document.querySelector("#sample-prompt");
const copy = document.querySelector("#copy-prompt");
const copyStatus = document.querySelector("#copy-status");
const requestedExample = new URLSearchParams(location.search).get("example");
const knownExample = [...document.querySelectorAll("[data-example]")].some(
  link => link.dataset.example === requestedExample
);
const example = knownExample ? requestedExample : "paper-sizes";
const examplePath = "./examples/" + example;

if (example !== "paper-sizes") {
  document.querySelector("#filename").textContent = example + ".page";
  download.href = examplePath + ".html";
  download.download = example + ".html";
}

let compiler;
let compilerReady = false;
let pendingSource = null;
let renderedSource = "";
let initialSource = "";
let downloadURL;

function setStatus(state, message = "") {
  previewNote.dataset.state = state;
  previewNote.hidden = state === "rendered";
  previewNote.textContent = previewNote.hidden ? "" : message;
  if (renderAnnouncement.textContent !== message) renderAnnouncement.textContent = message;
}

function syncEditorScroll() {
  lineNumbers.scrollTop = source.scrollTop;
  highlight.scrollTop = source.scrollTop;
  highlight.scrollLeft = source.scrollLeft;
}

function drawSource() {
  const text = source.value;
  const plain = text.length > 120000;
  editor.dataset.plain = String(plain);
  if (plain) {
    highlight.textContent = "";
    lineNumbers.textContent = "";
    return;
  }

  const fragments = document.createDocumentFragment();
  let start = 0;
  for (const token of text.matchAll(/"(?:\\.|[^"\\\n])*"|::\/?[a-z][\w-]*/gi)) {
    fragments.append(text.slice(start, token.index));
    const span = document.createElement("span");
    span.className = token[0].startsWith('"') ? "token-string" : "token-directive";
    span.textContent = token[0];
    fragments.append(span);
    start = token.index + token[0].length;
  }
  fragments.append(text.slice(start), "\n");
  highlight.replaceChildren(fragments);
  lineNumbers.textContent = text.split("\n").map((_, index) => index + 1).join("\n") + "\n";
  syncEditorScroll();
}

function clearDiagnostics() {
  diagnostics.hidden = true;
  diagnosticList.replaceChildren();
  source.removeAttribute("aria-invalid");
}

function selectDiagnostic({ line, column }) {
  const lines = source.value.split("\n");
  const index = Math.max(0, Math.min(line - 1, lines.length - 1));
  const lineStart = lines.slice(0, index).reduce((offset, text) => offset + text.length + 1, 0);
  const lineEnd = lineStart + lines[index].length;
  const lineHeight = Number.parseFloat(getComputedStyle(source).lineHeight);
  source.focus();
  source.setSelectionRange(Math.min(lineStart + Math.max(0, column - 1), lineEnd), lineEnd);
  source.scrollTop = Math.max(0, index - 2) * lineHeight;
  syncEditorScroll();
}

function showDiagnostics(errors) {
  source.setAttribute("aria-invalid", "true");
  for (const problem of errors) {
    const item = document.createElement("li");
    const button = document.createElement("button");
    button.type = "button";
    button.textContent = "Line " + problem.line + ", column " + problem.column + ". " + problem.message;
    button.addEventListener("click", () => selectDiagnostic(problem));
    item.append(button);
    diagnosticList.append(item);
  }
  diagnostics.hidden = false;
  diagnosticList.querySelector("button")?.focus();
}

function updateSourceStatus(message) {
  if (source.value === renderedSource) setStatus("rendered", message);
  else setStatus("stale", "Choose Render to preview your edits.");
}

function applyCompilation(result) {
  const submitted = pendingSource;
  pendingSource = null;
  render.removeAttribute("aria-disabled");

  if (result.status === "rendered") {
    renderedSource = submitted;
    preview.srcdoc = result.html;
    if (downloadURL) URL.revokeObjectURL(downloadURL);
    downloadURL = URL.createObjectURL(new Blob([result.html], { type: "text/html" }));
    download.href = downloadURL;
    updateSourceStatus("Preview updated.");
  } else if (source.value !== submitted) {
    updateSourceStatus();
  } else if (result.status === "invalid") {
    showDiagnostics(result.diagnostics);
    setStatus("error", "Fix the source errors. The preview shows the last successful render.");
  } else {
    setStatus("error", result.message + " The preview shows the last successful render.");
  }
}

function startCompiler() {
  return new Promise((resolve, reject) => {
    compiler = new Worker(new URL("./compiler-worker.js", import.meta.url), { type: "module" });
    compiler.addEventListener("message", ({ data }) => {
      if (data.type === "ready") resolve();
      else if (data.type === "startup_error") reject(new Error("Compiler startup failed"));
      else applyCompilation(data.result);
    });
    compiler.addEventListener("error", () => {
      if (!compilerReady) {
        reject(new Error("Could not load the compiler worker"));
        return;
      }
      compilerReady = false;
      pendingSource = null;
      render.removeAttribute("aria-disabled");
      render.disabled = true;
      setStatus("error", "The compiler stopped. Reload the page to start it again.");
    });
  });
}

function renderSource() {
  if (!compilerReady || pendingSource !== null) return;
  clearDiagnostics();
  if (!source.value.trim()) {
    source.setAttribute("aria-invalid", "true");
    setStatus("error", "Paste or write a .page file, then choose Render.");
    source.focus();
    return;
  }

  pendingSource = source.value;
  render.setAttribute("aria-disabled", "true");
  setStatus("working", "Rendering…");
  compiler.postMessage(pendingSource);
}

function warnBeforeLeaving(event) {
  event.preventDefault();
  event.returnValue = "";
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  renderSource();
});
source.addEventListener("input", () => {
  clearDiagnostics();
  drawSource();
  if (compilerReady && pendingSource === null) updateSourceStatus();
  if (source.value === initialSource) window.removeEventListener("beforeunload", warnBeforeLeaving);
  else window.addEventListener("beforeunload", warnBeforeLeaving);
});
source.addEventListener("scroll", syncEditorScroll);
source.addEventListener("keydown", (event) => {
  if (event.key === "Enter" && (event.ctrlKey || event.metaKey)) {
    event.preventDefault();
    renderSource();
  }
});

copy.addEventListener("click", async () => {
  copyStatus.textContent = "";
  try {
    await navigator.clipboard.writeText(prompt.value);
    copyStatus.textContent = "Prompt copied";
  } catch {
    prompt.focus();
    prompt.select();
    let copied = false;
    try {
      copied = document.execCommand("copy");
    } catch {
      // The selected prompt remains available for manual copying.
    }
    if (copied) {
      copy.focus();
      copyStatus.textContent = "Prompt copied";
    } else {
      copyStatus.textContent = "Copy the selected prompt.";
    }
  }
});
copy.disabled = false;

async function loadSample() {
  const [sourceFile, htmlFile] = await Promise.all([
    fetch(examplePath + ".page"),
    fetch(examplePath + ".html")
  ]);
  if (!sourceFile.ok || !htmlFile.ok) throw new Error("Sample could not load");
  const [pageSource, html] = await Promise.all([sourceFile.text(), htmlFile.text()]);
  initialSource = pageSource;
  preview.srcdoc = html;
  source.value = initialSource;
  renderedSource = initialSource;
  source.disabled = false;
  drawSource();
}

try {
  setStatus("loading", "Loading the compiler…");
  await Promise.all([loadSample(), startCompiler()]);
  compilerReady = true;
  render.disabled = false;
  updateSourceStatus();
} catch {
  setStatus("error", "The playground could not start. Reload to try again.");
}
