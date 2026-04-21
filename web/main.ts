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

type Lang = "en" | "ua";

const LANG_STORAGE = "layoutRsLang";

function normalizeLangToken(raw: string | null | undefined): Lang | null {
  if (raw == null) return null;
  const v = raw.trim().toLowerCase();
  if (v === "ua" || v === "uk") return "ua";
  if (v === "en") return "en";
  return null;
}

/** Resolves UI + WASM sheet locale from `?lang=` (case-insensitive) or `localStorage`. */
function readLang(): Lang {
  const params = new URLSearchParams(window.location.search);
  const fromUrl = normalizeLangToken(params.get("lang"));
  if (fromUrl) {
    localStorage.setItem(LANG_STORAGE, fromUrl);
    return fromUrl;
  }
  const stored = normalizeLangToken(localStorage.getItem(LANG_STORAGE));
  if (stored) return stored;
  return "en";
}

/** Keep the address bar aligned with the resolved language (bookmark / refresh). */
function syncUrlLangParam(lang: Lang): void {
  const params = new URLSearchParams(window.location.search);
  const want = lang === "ua" ? "ua" : "en";
  const cur = (params.get("lang") ?? "").trim().toLowerCase();
  if (cur === want) return;
  params.set("lang", want);
  const qs = params.toString();
  const next = `${location.pathname}${qs ? `?${qs}` : ""}${location.hash}`;
  history.replaceState(null, "", next);
}

function setLang(lang: Lang): void {
  if (readLang() === lang) return;
  localStorage.setItem(LANG_STORAGE, lang);
  const u = new URL(window.location.href);
  u.searchParams.set("lang", lang);
  location.href = u.toString();
}

type Msg = {
  docTitle: string;
  appH1: string;
  appTagline: string;
  engineHtml: string;
  langLabel: string;
  sheetH2: string;
  lblSheetW: string;
  lblSheetH: string;
  lblKmax: string;
  kMaxPlaceholder: string;
  kMaxTitle: string;
  kMaxHintHtml: string;
  lblRotationSheet: string;
  productsH2: string;
  addProduct: string;
  compute: string;
  resultH2: string;
  previewH2: string;
  placeholderHtml: string;
  productId: string;
  remove: string;
  removeTitle: string;
  widthW: string;
  heightH: string;
  quantity: string;
  rot90: string;
  rotDefault: string;
  rotYes: string;
  rotNo: string;
  errSheetDims: string;
  errProducts: string;
  errKmax: string;
  errLayout: string;
  statK: string;
  statPerSheet: string;
  statPressSheets: string;
  statProduced: string;
  statUtil: string;
  statOverByType: string;
  statOverTotal: string;
  statPackMode: string;
  statGcd: string;
  statRotation: string;
  statPackStageNote: string;
  kMixed: string;
  rotAllow: string;
  rotDisallow: string;
  rotDefaultFmt: (sheetAllow: boolean) => string;
};

const MESSAGES: Record<Lang, Msg> = {
  en: {
    docTitle: "layout-rs — sheet preview (WASM)",
    appH1: "Print sheet layout",
    appTagline:
      "Proportional counts per sheet, multi-stage guillotine (V–H / H–V). Optional 90° rotation per piece.",
    engineHtml: "Engine: Rust → WebAssembly (<code>solveLayoutJson</code>).",
    langLabel: "Language:",
    sheetH2: "Sheet",
    lblSheetW: "Width Wp",
    lblSheetH: "Height Hp",
    lblKmax: "Max k (optional)",
    kMaxPlaceholder: "default",
    kMaxTitle:
      "Largest scale k tried when building proportional per-sheet piece counts. Empty uses default (500); also capped from sheet area (overall max 2000). Lower runs faster; higher searches more mixes.",
    kMaxHintHtml: `The solver scales your targets’ proportion pattern by an integer
                <strong>k</strong> to get candidate piece counts per sheet. This
                field is the largest <strong>k</strong> it will try. Leave empty
                for the default (500). Smaller values finish faster but may miss
                a better layout; values are also limited by sheet size (up to
                2000).`,
    lblRotationSheet: 'Default: allow 90° rotation (for products set to “Sheet default”)',
    productsH2: "Products",
    addProduct: "+ Add",
    compute: "Compute layout",
    resultH2: "Result",
    previewH2: "One sheet preview",
    placeholderHtml:
      "Set sheet size and products, then click <strong>Compute layout</strong>.",
    productId: "Product id",
    remove: "Remove",
    removeTitle: "Remove this product",
    widthW: "Width W",
    heightH: "Height H",
    quantity: "Quantity",
    rot90: "90° rotation",
    rotDefault: "Sheet default",
    rotYes: "Allow",
    rotNo: "Disallow",
    errSheetDims: "Sheet width and height must be positive integers.",
    errProducts: "Each product needs id, W, H, and quantity ≥ 1. Ids must be unique.",
    errKmax: "Max k must be a positive integer or left empty.",
    errLayout:
      "No feasible layout for this sheet and k range. Try a larger sheet, fewer items per sheet (lower k via max k), or smaller products.",
    statK: "k (gcd scale)",
    statPerSheet: "Per sheet",
    statPressSheets: "Press sheets P",
    statProduced: "Produced by type",
    statUtil: "Utilization",
    statOverByType: "Overproduction by type",
    statOverTotal: "Overproduction (total pcs)",
    statPackMode: "Pack mode",
    statGcd: "gcd d",
    statRotation: "Rotation",
    statPackStageNote: "(multi-stage)",
    kMixed: "— (mixed)",
    rotAllow: "allow",
    rotDisallow: "disallow",
    rotDefaultFmt: (sheetAllow: boolean) => `default (${sheetAllow ? "allow" : "disallow"})`,
  },
  ua: {
    docTitle: "layout-rs — попередній перегляд аркуша (WASM)",
    appH1: "Розкладка друку на аркуші",
    appTagline:
      "Пропорційна кількість деталей на аркуші, багатоетапний гільйотинний розклад (V–H / H–V). За потреби — поворот кожної деталі на 90°.",
    engineHtml: "Рушій: Rust → WebAssembly (<code>solveLayoutJson</code>).",
    langLabel: "Мова:",
    sheetH2: "Аркуш",
    lblSheetW: "Ширина Wp",
    lblSheetH: "Висота Hp",
    lblKmax: "Макс. k (необов’язково)",
    kMaxPlaceholder: "типово",
    kMaxTitle:
      "Найбільший масштаб k, який перебирається для пропорційної кількості деталей на аркуші. Порожньо — типово (500); також обмежується площею аркуша (загалом макс. 2000). Менші значення швидші; більші перебирають більше варіантів.",
    kMaxHintHtml: `Розв’язувач масштабує пропорційний шаблон цілей на ціле число
                <strong>k</strong>, щоб отримати кандидатні кількості деталей на аркуші. Це поле задає
                найбільше <strong>k</strong>, яке буде перебрано. Залиште порожнім для типового значення (500).
                Менші значення швидші, але можуть пропустити кращий розклад; значення також обмежені розміром аркуша (до
                2000).`,
    lblRotationSheet:
      "За замовчуванням: дозволити поворот на 90° (для виробів з режимом «Як на аркуші»)",
    productsH2: "Вироби",
    addProduct: "+ Додати",
    compute: "Обчислити розкладку",
    resultH2: "Результат",
    previewH2: "Перегляд одного аркуша",
    placeholderHtml:
      "Задайте розмір аркуша та вироби, потім натисніть <strong>Обчислити розкладку</strong>.",
    productId: "Id виробу",
    remove: "Вилучити",
    removeTitle: "Вилучити цей виріб",
    widthW: "Ширина W",
    heightH: "Висота H",
    quantity: "Кількість",
    rot90: "Поворот на 90°",
    rotDefault: "Як на аркуші",
    rotYes: "Дозволити",
    rotNo: "Заборонити",
    errSheetDims: "Ширина та висота аркуша мають бути додатними цілими числами.",
    errProducts:
      "Кожен виріб потребує id, W, H і кількості ≥ 1. Ідентифікатори мають бути унікальними.",
    errKmax: "Макс. k має бути додатним цілим або залиште поле порожнім.",
    errLayout:
      "Немає придатного розкладу для цього аркуша та діапазону k. Спробуйте більший аркуш, менше деталей на аркуші (менший k через макс. k) або менші вироби.",
    statK: "k (масштаб НСД)",
    statPerSheet: "На аркуші",
    statPressSheets: "Листів друку P",
    statProduced: "Виготовлено за типами",
    statUtil: "Використання площі",
    statOverByType: "Перевироб за типами",
    statOverTotal: "Перевироб (усього шт.)",
    statPackMode: "Режим пакування",
    statGcd: "НСД d",
    statRotation: "Поворот",
    statPackStageNote: "(багатоетапно)",
    kMixed: "— (змішано)",
    rotAllow: "дозволено",
    rotDisallow: "заборонено",
    rotDefaultFmt: (sheetAllow: boolean) =>
      `як на аркуші (${sheetAllow ? "дозволено" : "заборонено"})`,
  },
};

function applyI18n(lang: Lang): void {
  const m = MESSAGES[lang];
  document.documentElement.lang = lang === "ua" ? "uk" : "en";
  document.title = m.docTitle;
  const setText = (id: string, text: string) => {
    const el = document.getElementById(id);
    if (el) el.textContent = text;
  };
  const setHtml = (id: string, html: string) => {
    const el = document.getElementById(id);
    if (el) el.innerHTML = html;
  };
  setText("i18n-h1", m.appH1);
  setText("i18n-tagline", m.appTagline);
  setHtml("i18n-engine", m.engineHtml);
  setText("i18n-langLabel", m.langLabel);
  setText("i18n-sheetH2", m.sheetH2);
  setText("i18n-lblSheetW", m.lblSheetW);
  setText("i18n-lblSheetH", m.lblSheetH);
  setText("i18n-lblKmax", m.lblKmax);
  setHtml("kMaxHint", m.kMaxHintHtml);
  setText("i18n-lblRotationSheet", m.lblRotationSheet);
  setText("i18n-productsH2", m.productsH2);
  setText("addProduct", m.addProduct);
  setText("compute", m.compute);
  setText("i18n-resultH2", m.resultH2);
  setText("i18n-previewH2", m.previewH2);
  setHtml("placeholder", m.placeholderHtml);
  const kMax = document.getElementById("kMax") as HTMLInputElement | null;
  if (kMax) {
    kMax.placeholder = m.kMaxPlaceholder;
    kMax.title = m.kMaxTitle;
  }
}

function rotationSummary(
  products: ProductWire[],
  sheetDefault: boolean,
  m: Msg,
): string {
  return products
    .map((p) => {
      if (p.allowRotation === true) return `${p.id}: ${m.rotAllow}`;
      if (p.allowRotation === false) return `${p.id}: ${m.rotDisallow}`;
      return `${p.id}: ${m.rotDefaultFmt(sheetDefault)}`;
    })
    .join("; ");
}

async function main(): Promise<void> {
  await init();

  const lang = readLang();
  syncUrlLangParam(lang);
  const msg = MESSAGES[lang];
  applyI18n(lang);

  document.getElementById("langEn")!.addEventListener("click", () => setLang("en"));
  document.getElementById("langUa")!.addEventListener("click", () => setLang("ua"));

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

  function currentPlaceholder(): HTMLParagraphElement | null {
    return svgHost.querySelector("#placeholder");
  }

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
          <span>${escapeAttr(msg.productId)}</span>
          <input type="text" data-f="id" data-i="${index}" value="${escapeAttr(row.id)}" autocomplete="off" />
        </label>
        <button type="button" class="btn danger product-remove" data-remove="${index}" title="${escapeAttr(msg.removeTitle)}" ${rows.length <= 1 ? "disabled" : ""}>${escapeAttr(msg.remove)}</button>
      </div>
      <div class="product-row-dims">
        <label class="product-field">
          <span>${escapeAttr(msg.widthW)}</span>
          <input type="number" data-f="w" data-i="${index}" min="1" value="${escapeAttr(row.w)}" />
        </label>
        <label class="product-field">
          <span>${escapeAttr(msg.heightH)}</span>
          <input type="number" data-f="h" data-i="${index}" min="1" value="${escapeAttr(row.h)}" />
        </label>
        <label class="product-field">
          <span>${escapeAttr(msg.quantity)}</span>
          <input type="number" data-f="target" data-i="${index}" min="1" value="${escapeAttr(row.target)}" />
        </label>
        <label class="product-field product-field-select">
          <span>${escapeAttr(msg.rot90)}</span>
          <select data-f="rotation" data-i="${index}" class="product-select">
            <option value="default" ${row.rotation === "default" ? "selected" : ""}>${escapeAttr(msg.rotDefault)}</option>
            <option value="yes" ${row.rotation === "yes" ? "selected" : ""}>${escapeAttr(msg.rotYes)}</option>
            <option value="no" ${row.rotation === "no" ? "selected" : ""}>${escapeAttr(msg.rotNo)}</option>
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

  function showError(text: string): void {
    errorEl.textContent = text;
    errorEl.hidden = false;
  }

  function clearSvg(): void {
    svgHost.innerHTML = "";
    const p = document.createElement("p");
    p.className = "placeholder";
    p.id = "placeholder";
    p.innerHTML = msg.placeholderHtml;
    svgHost.appendChild(p);
  }

  function compute(): void {
    errorEl.hidden = true;
    statsSection.hidden = true;
    const ph = currentPlaceholder();
    if (ph) ph.hidden = false;

    const W = Math.floor(Number(sheetW.value));
    const H = Math.floor(Number(sheetH.value));
    if (!Number.isFinite(W) || W < 1 || !Number.isFinite(H) || H < 1) {
      showError(msg.errSheetDims);
      return;
    }

    const products = readProducts();
    if (!products) {
      showError(msg.errProducts);
      return;
    }

    const kRaw = kMaxInput.value.trim();
    const kMax = kRaw === "" ? undefined : Math.floor(Number(kRaw));
    if (kRaw !== "" && (!Number.isFinite(kMax) || kMax! < 1)) {
      showError(msg.errKmax);
      return;
    }

    const defaultAllowRotation = allowRotationEl.checked;

    const sheetLocale = readLang() === "ua" ? "ua" : "en";
    const payload: Record<string, unknown> = {
      sheetW: W,
      sheetH: H,
      products,
      allowRotation: defaultAllowRotation,
      locale: sheetLocale,
    };
    if (kMax !== undefined) payload.kMax = kMax;

    let sol: WasmSolveResult;
    try {
      const raw = solveLayoutJson(JSON.stringify(payload));
      sol = JSON.parse(raw) as WasmSolveResult;
    } catch (e) {
      const fallback =
        e instanceof Error ? e.message : msg.errLayout;
      showError(fallback || msg.errLayout);
      clearSvg();
      return;
    }

    if (ph) ph.hidden = true;
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
    const kLabel = sol.k > 0 ? String(sol.k) : msg.kMixed;

    statsEl.innerHTML = `
    <dt>${escapeHtml(msg.statK)}</dt><dd>${kLabel}</dd>
    <dt>${escapeHtml(msg.statPerSheet)}</dt><dd>${perPage}</dd>
    <dt>${escapeHtml(msg.statPressSheets)}</dt><dd>${sol.pages}</dd>
    <dt>${escapeHtml(msg.statProduced)}</dt><dd>${escapeHtml(producedByType)}</dd>
    <dt>${escapeHtml(msg.statUtil)}</dt><dd>${(sol.utilization * 100).toFixed(2)}%</dd>
    <dt>${escapeHtml(msg.statOverByType)}</dt><dd>${escapeHtml(overByType)}</dd>
    <dt>${escapeHtml(msg.statOverTotal)}</dt><dd>+${sol.overproduction}</dd>
    <dt>${escapeHtml(msg.statPackMode)}</dt><dd>${escapeHtml(sol.pack.mode)} ${escapeHtml(msg.statPackStageNote)}</dd>
    <dt>${escapeHtml(msg.statGcd)}</dt><dd>${sol.pattern.d}</dd>
    <dt>${escapeHtml(msg.statRotation)}</dt><dd>${escapeHtml(rotationSummary(products, defaultAllowRotation, msg))}</dd>
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
