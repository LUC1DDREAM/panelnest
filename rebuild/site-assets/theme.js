"use strict";
(() => {
  let theme = "dark";
  try { if (localStorage.getItem("panelnest.theme") === "light") theme = "light"; } catch (_) { /* Optional storage. */ }
  document.documentElement.dataset.theme = theme;
  document.querySelector('meta[name="theme-color"]').content = theme === "dark" ? "#0d1423" : "#f5f7fb";
})();
