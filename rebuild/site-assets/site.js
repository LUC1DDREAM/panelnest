"use strict";
(() => {
  const root = document.body;
  const links = [...document.querySelectorAll("[data-language]")];
  const available = links.map(link => link.dataset.language);
  const store = "panelnest.language";
  let saved;
  try { saved = localStorage.getItem(store); } catch (_) { /* Storage is optional. */ }
  if (root.dataset.autoLanguage === "true") {
    const preferred = available.includes(saved) ? saved : "en";
    if (preferred && preferred !== "en") location.replace(root.dataset.base + preferred + "/" + location.hash);
  }
  links.forEach(link => link.addEventListener("click", () => {
    try { localStorage.setItem(store, link.dataset.language); } catch (_) { /* Links still work. */ }
  }));
  document.addEventListener("keydown", event => {
    if (event.key === "Escape") document.querySelectorAll("details[open]").forEach(detail => { detail.open = false; detail.querySelector("summary").focus(); });
  });
  const toggle = document.getElementById("theme-toggle");
  const updateTheme = () => {
    const dark = document.documentElement.dataset.theme === "dark";
    toggle.textContent = dark ? toggle.dataset.light : toggle.dataset.dark;
    toggle.setAttribute("aria-label", toggle.textContent);
    document.querySelector('meta[name="theme-color"]').content = dark ? "#101b2a" : "#f4f7fb";
  };
  toggle.hidden = false;
  updateTheme();
  toggle.addEventListener("click", () => {
    const theme = document.documentElement.dataset.theme === "dark" ? "light" : "dark";
    document.documentElement.dataset.theme = theme;
    try { localStorage.setItem("panelnest.theme", theme); } catch (_) { /* Optional storage. */ }
    updateTheme();
  });
  const copy = document.getElementById("copy-url");
  copy.hidden = false;
  copy.addEventListener("click", async () => {
    const code = document.getElementById("list-url");
    const status = document.getElementById("copy-status");
    try {
      await navigator.clipboard.writeText(code.textContent);
      status.textContent = copy.dataset.success;
    } catch (_) {
      const range = document.createRange(); range.selectNodeContents(code);
      const selection = window.getSelection(); selection.removeAllRanges(); selection.addRange(range);
      code.focus(); status.textContent = copy.dataset.failure;
    }
  });
})();
