import init, { solveLayoutJson } from "../pkg/layout_rs.js";

type RotationMode = "default" | "yes" | "no";

type Row = { id: string; w: string; h: string; target: string; rotation: RotationMode };

/** Matches `solveLayoutJson` response (camelCase). */
interface WasmSolveResult {
  k: number;
  countsPerPage: number[];
  pages: number;
  overproduction: number;
  utilization: number;
  pattern: { d: number; p: number[] };
  pack: {
    mode: string;
    utilization: number;
    rects: Array<{
      x: number;
      y: number;
      w: number;
      h: number;
      productId: string;
      rotated: boolean;
    }>;
  };
  svg: string;
}

type ProductWire = {
  id: string;
  w: number;
  h: number;
  target: number;
  allowRotation?: boolean;
};

async function main(): Promise<void> {
  await init();

  const defaultRows: Row[] = [
    { id: "A", w: "100", h: "300", target: "1000", rotation: "default" },
    { id: "B", w: "70", h: "100", target: "2000", rotation: "default" },
  ];

  let rows: Row[] = [...defaultRows];

  const productList = document.getElementById("productList")!;
  const sheetW = document.getElementById("sheetW") as HTMLInputElement;
  const sheetH = document.getElementById("sheetH") as HTMLInputElement;
  const kMaxInput = document.getElementById("kMax") as HTMLInputElement;
  const allowRotationEl = document.getElementById("allowRotation") as HTMLInputElement;
  const computeBtn = document.getElementById("compute")!;
  const addBtn = document.getElementById("addProduct")!;
  const errorEl = document.getElementById("error") as HTMLParagraphElement;
  const statsSection = document.getElementById("statsSection") as HTMLElement;
  const statsEl = document.getElementById("stats")!;
  const svgHost = document.getElementById("svgHost")!;
  const placeholder = document.getElementById("placeholder") as HTMLParagraphElement;

  function escapeAttr(s: string): string {
    return s.replace(/&/g, "&amp;").replace(/"/g, "&quot;").replace(/</g, "&lt;");
  }

  function renderRows(): void {
    productList.innerHTML = "";
    rows.forEach((row, index) => {
      const div = document.createElement("div");
      div.className = "product-row";
      div.innerHTML = `
      <div class="product-row-top">
        <label class="product-field product-field-id">
          <span>Product id</span>
          <input type="text" data-f="id" data-i="${index}" value="${escapeAttr(row.id)}" autocomplete="off" />
        </label>
        <button type="button" class="btn danger product-remove" data-remove="${index}" title="Remove this product" ${rows.length <= 1 ? "disabled" : ""}>Remove</button>
      </div>
      <div class="product-row-dims">
        <label class="product-field">
          <span>Width W</span>
          <input type="number" data-f="w" data-i="${index}" min="1" value="${escapeAttr(row.w)}" />
        </label>
        <label class="product-field">
          <span>Height H</span>
          <input type="number" data-f="h" data-i="${index}" min="1" value="${escapeAttr(row.h)}" />
        </label>
        <label class="product-field">
          <span>Quantity</span>
          <input type="number" data-f="target" data-i="${index}" min="1" value="${escapeAttr(row.target)}" />
        </label>
        <label class="product-field product-field-select">
          <span>90° rotation</span>
          <select data-f="rotation" data-i="${index}" class="product-select">
            <option value="default" ${row.rotation === "default" ? "selected" : ""}>Sheet default</option>
            <option value="yes" ${row.rotation === "yes" ? "selected" : ""}>Allow</option>
            <option value="no" ${row.rotation === "no" ? "selected" : ""}>Disallow</option>
          </select>
        </label>
      </div>
    `;
      productList.appendChild(div);
    });

    productList.querySelectorAll("input").forEach((el) => {
      el.addEventListener("input", onRowInput);
    });
    productList.querySelectorAll("select[data-f]").forEach((el) => {
      el.addEventListener("change", onRowInput);
    });
    productList.querySelectorAll("[data-remove]").forEach((btn) => {
      btn.addEventListener("click", () => {
        const i = Number((btn as HTMLButtonElement).dataset.remove);
        if (rows.length > 1) {
          rows.splice(i, 1);
          renderRows();
        }
      });
    });
  }

  function onRowInput(ev: Event): void {
    const el = ev.target as HTMLInputElement | HTMLSelectElement;
    const i = Number(el.dataset.i);
    const f = el.dataset.f as keyof Row | "rotation";
    if (Number.isNaN(i) || !f) return;
    if (f === "rotation") {
      const v = (el as HTMLSelectElement).value;
      if (v === "default" || v === "yes" || v === "no") {
        rows[i]!.rotation = v;
      }
      return;
    }
    (rows[i]![f] as string) = el.value;
  }

  function readProducts(): ProductWire[] | null {
    const out: ProductWire[] = [];
    for (const r of rows) {
      const id = r.id.trim() || "?";
      const w = Math.floor(Number(r.w));
      const h = Math.floor(Number(r.h));
      const target = Math.floor(Number(r.target));
      if (!Number.isFinite(w) || w < 1 || !Number.isFinite(h) || h < 1) {
        return null;
      }
      if (!Number.isFinite(target) || target < 1) {
        return null;
      }
      const spec: ProductWire = { id, w, h, target };
      if (r.rotation === "yes") spec.allowRotation = true;
      else if (r.rotation === "no") spec.allowRotation = false;
      out.push(spec);
    }
    const ids = new Set(out.map((p) => p.id));
    if (ids.size !== out.length) {
      return null;
    }
    return out;
  }

  function escapeHtml(s: string): string {
    return s
      .replace(/&/g, "&amp;")
      .replace(/</g, "&lt;")
      .replace(/>/g, "&gt;")
      .replace(/"/g, "&quot;");
  }

  function rotationSummary(products: ProductWire[], sheetDefault: boolean): string {
    return products
      .map((p) => {
        if (p.allowRotation === true) return `${p.id}: allow`;
        if (p.allowRotation === false) return `${p.id}: disallow`;
        return `${p.id}: default (${sheetDefault ? "allow" : "disallow"})`;
      })
      .join("; ");
  }

  function showError(msg: string): void {
    errorEl.textContent = msg;
    errorEl.hidden = false;
  }

  function clearSvg(): void {
    svgHost.innerHTML = "";
    const p = document.createElement("p");
    p.className = "placeholder";
    p.id = "placeholder";
    p.innerHTML =
      "Set sheet size and products, then click <strong>Compute layout</strong>.";
    svgHost.appendChild(p);
  }

  function compute(): void {
    errorEl.hidden = true;
    statsSection.hidden = true;
    placeholder.hidden = false;

    const W = Math.floor(Number(sheetW.value));
    const H = Math.floor(Number(sheetH.value));
    if (!Number.isFinite(W) || W < 1 || !Number.isFinite(H) || H < 1) {
      showError("Sheet width and height must be positive integers.");
      return;
    }

    const products = readProducts();
    if (!products) {
      showError("Each product needs id, W, H, and quantity ≥ 1. Ids must be unique.");
      return;
    }

    const kRaw = kMaxInput.value.trim();
    const kMax = kRaw === "" ? undefined : Math.floor(Number(kRaw));
    if (kRaw !== "" && (!Number.isFinite(kMax) || kMax! < 1)) {
      showError("Max k must be a positive integer or left empty.");
      return;
    }

    const defaultAllowRotation = allowRotationEl.checked;

    const payload: Record<string, unknown> = {
      sheetW: W,
      sheetH: H,
      products,
      allowRotation: defaultAllowRotation,
    };
    if (kMax !== undefined) payload.kMax = kMax;

    let sol: WasmSolveResult;
    try {
      const raw = solveLayoutJson(JSON.stringify(payload));
      sol = JSON.parse(raw) as WasmSolveResult;
    } catch (e) {
      const msg =
        e instanceof Error
          ? e.message
          : "No feasible layout for this sheet and k range. Try a larger sheet, fewer items per sheet (lower k via max k), or smaller products.";
      showError(msg);
      clearSvg();
      return;
    }

    placeholder.hidden = true;
    statsSection.hidden = false;

    const perPage = products.map((p, i) => `${p.id}×${sol.countsPerPage[i]}`).join(", ");
    const producedByType = products
      .map((p, i) => {
        const perSheet = sol.countsPerPage[i] ?? 0;
        const produced = perSheet * sol.pages;
        return `${p.id}: ${produced}`;
      })
      .join("; ");
    const overByType = products
      .map((p, i) => {
        const perSheet = sol.countsPerPage[i] ?? 0;
        const produced = perSheet * sol.pages;
        const over = Math.max(0, produced - p.target);
        return `${p.id}: +${over}`;
      })
      .join("; ");
    const kLabel = sol.k > 0 ? String(sol.k) : "— (mixed)";

    statsEl.innerHTML = `
    <dt>k (gcd scale)</dt><dd>${kLabel}</dd>
    <dt>Per sheet</dt><dd>${perPage}</dd>
    <dt>Press sheets P</dt><dd>${sol.pages}</dd>
    <dt>Produced by type</dt><dd>${escapeHtml(producedByType)}</dd>
    <dt>Utilization</dt><dd>${(sol.utilization * 100).toFixed(2)}%</dd>
    <dt>Overproduction by type</dt><dd>${escapeHtml(overByType)}</dd>
    <dt>Overproduction (total pcs)</dt><dd>+${sol.overproduction}</dd>
    <dt>Pack mode</dt><dd>${sol.pack.mode} (multi-stage)</dd>
    <dt>gcd d</dt><dd>${sol.pattern.d}</dd>
    <dt>Rotation</dt><dd>${escapeHtml(rotationSummary(products, defaultAllowRotation))}</dd>
  `;

    svgHost.innerHTML = sol.svg;
  }

  addBtn.addEventListener("click", () => {
    const n = rows.length + 1;
    rows.push({ id: `P${n}`, w: "50", h: "50", target: "100", rotation: "default" });
    renderRows();
  });

  computeBtn.addEventListener("click", compute);

  renderRows();
}

void main().catch((e) => {
  console.error(e);
});
