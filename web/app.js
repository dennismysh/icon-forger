import init, {
  render_icon_png,
  render_icon,
  svg_to_png,
  svg_to_format,
  available_formats,
  standard_sizes,
} from "./pkg/icon_forger.js";

let wasmReady = false;
let currentPngBlob = null;
let currentMode = "codegen";
let loadedSvg = null;

// ── Init ─────────────────────────────────────────────────────────

async function boot() {
  await init();
  wasmReady = true;
  console.log("WASM loaded. Formats:", available_formats());
  console.log("Standard sizes:", standard_sizes());
}

boot().catch((e) => console.error("WASM init failed:", e));

// ── Tab switching ────────────────────────────────────────────────

document.querySelectorAll(".tab").forEach((tab) => {
  tab.addEventListener("click", () => {
    document.querySelectorAll(".tab").forEach((t) => t.classList.remove("active"));
    document.querySelectorAll(".mode-panel").forEach((p) => p.classList.remove("active"));
    tab.classList.add("active");
    const mode = tab.dataset.mode;
    document.getElementById(mode).classList.add("active");
    currentMode = mode;
  });
});

// ── Helpers ──────────────────────────────────────────────────────

function hexToRgba(hex) {
  const r = parseInt(hex.slice(1, 3), 16);
  const g = parseInt(hex.slice(3, 5), 16);
  const b = parseInt(hex.slice(5, 7), 16);
  return { r, g, b, a: 255 };
}

function showPreview(pngBytes) {
  const blob = new Blob([pngBytes], { type: "image/png" });
  currentPngBlob = blob;
  const url = URL.createObjectURL(blob);
  const area = document.getElementById("preview-area");
  area.innerHTML = `<img src="${url}" alt="icon preview" />`;

  // Enable export buttons
  document.querySelectorAll(".export-btn").forEach((btn) => (btn.disabled = false));
}

function downloadBlob(blob, filename) {
  const url = URL.createObjectURL(blob);
  const a = document.createElement("a");
  a.href = url;
  a.download = filename;
  a.click();
  URL.revokeObjectURL(url);
}

// ── Mode 1: Code-first ──────────────────────────────────────────

function buildIconDef() {
  const bg = hexToRgba(document.getElementById("bg-color").value);
  const fg = hexToRgba(document.getElementById("fg-color").value);
  const shape = document.getElementById("shape-select").value;
  const cornerRadius = parseInt(document.getElementById("corner-radius").value) / 100;

  const layers = [];

  // Background rounded rect
  layers.push({
    RoundedRect: { corner_radius: cornerRadius, color: bg },
  });

  // Foreground shape
  switch (shape) {
    case "circle":
      layers.push({ Circle: { radius: 0.5, color: fg } });
      break;
    case "rounded-rect":
      layers.push({ RoundedRect: { corner_radius: cornerRadius * 0.8, color: fg } });
      break;
    case "polygon-3":
      layers.push({ Polygon: { sides: 3, color: fg } });
      break;
    case "polygon-5":
      layers.push({ Polygon: { sides: 5, color: fg } });
      break;
    case "polygon-6":
      layers.push({ Polygon: { sides: 6, color: fg } });
      break;
    case "ring":
      layers.push({ Ring: { inner: 0.35, outer: 0.8, color: fg } });
      break;
  }

  return { background: bg, layers };
}

document.getElementById("btn-render").addEventListener("click", () => {
  if (!wasmReady) return alert("WASM is still loading…");

  const def = buildIconDef();
  const size = parseInt(document.getElementById("render-size").value);

  try {
    const pngBytes = render_icon_png(JSON.stringify(def), size);
    showPreview(pngBytes);
  } catch (e) {
    console.error("Render error:", e);
    alert("Render failed: " + e);
  }
});

// ── Mode 2: SVG Import ──────────────────────────────────────────

const dropZone = document.getElementById("drop-zone");
const svgInput = document.getElementById("svg-file");
const btnImport = document.getElementById("btn-import");

function handleSvgFile(file) {
  const reader = new FileReader();
  reader.onload = (e) => {
    loadedSvg = e.target.result;
    dropZone.querySelector("p").textContent = `Loaded: ${file.name}`;
    btnImport.disabled = false;
  };
  reader.readAsText(file);
}

svgInput.addEventListener("change", (e) => {
  if (e.target.files[0]) handleSvgFile(e.target.files[0]);
});

dropZone.addEventListener("dragover", (e) => {
  e.preventDefault();
  dropZone.classList.add("drag-over");
});

dropZone.addEventListener("dragleave", () => {
  dropZone.classList.remove("drag-over");
});

dropZone.addEventListener("drop", (e) => {
  e.preventDefault();
  dropZone.classList.remove("drag-over");
  const file = e.dataTransfer.files[0];
  if (file && file.name.endsWith(".svg")) handleSvgFile(file);
});

btnImport.addEventListener("click", () => {
  if (!wasmReady || !loadedSvg) return;
  const size = parseInt(document.getElementById("import-size").value);

  try {
    const pngBytes = svg_to_png(loadedSvg, size);
    showPreview(pngBytes);
  } catch (e) {
    console.error("SVG render error:", e);
    alert("SVG render failed: " + e);
  }
});

// ── Export buttons ───────────────────────────────────────────────

document.querySelectorAll(".export-btn").forEach((btn) => {
  btn.addEventListener("click", () => {
    if (!wasmReady) return;

    const format = btn.dataset.format;
    const size = 1024; // Export at max resolution

    const mimeTypes = {
      png: "image/png",
      ico: "image/x-icon",
      icns: "application/octet-stream",
      webp: "image/webp",
    };

    const extensions = { png: "png", ico: "ico", icns: "icns", webp: "webp" };

    try {
      let bytes;

      if (currentMode === "codegen") {
        const def = buildIconDef();
        bytes = render_icon(JSON.stringify(def), size, format);
      } else if (loadedSvg) {
        bytes = svg_to_format(loadedSvg, size, format);
      } else {
        return alert("No icon to export");
      }

      const blob = new Blob([bytes], { type: mimeTypes[format] });
      downloadBlob(blob, `icon.${extensions[format]}`);
    } catch (e) {
      console.error("Export error:", e);
      alert("Export failed: " + e);
    }
  });
});
