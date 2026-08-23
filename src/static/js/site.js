const themeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)")

function currentTheme() {
  return localStorage.getItem("theme") || "system"
}

function applyTheme(theme) {
  const dark =
    theme === "dark" || (theme === "system" && themeMediaQuery.matches)
  document.documentElement.classList.toggle("dark", dark)
  document.documentElement.style.colorScheme = dark ? "dark" : "light"
  document.documentElement.dataset.theme = theme

  const toggle = document.querySelector("[data-theme-toggle]")
  if (!toggle) return

  const label = `${theme[0].toUpperCase()}${theme.slice(1)} theme. Switch theme.`
  toggle.setAttribute("aria-label", label)
  toggle.setAttribute("title", label)
}

function initThemeToggle() {
  const toggle = document.querySelector("[data-theme-toggle]")
  if (!toggle || toggle.dataset.initialized) return

  toggle.dataset.initialized = "true"
  applyTheme(currentTheme())
  toggle.addEventListener("click", () => {
    const next = { system: "light", light: "dark", dark: "system" }[
      currentTheme()
    ]
    if (next === "system") localStorage.removeItem("theme")
    else localStorage.setItem("theme", next)
    applyTheme(next)
  })
}

themeMediaQuery.addEventListener("change", () => {
  if (currentTheme() === "system") applyTheme("system")
})

function initializePage() {
  initThemeToggle()
}

document.addEventListener("DOMContentLoaded", () => {
  window.mu.init({ target: "main", source: "main", transition: false })
  initializePage()
})

document.addEventListener("mu:after-render", () => {
  requestAnimationFrame(() => {
    initializePage()
  })
})
