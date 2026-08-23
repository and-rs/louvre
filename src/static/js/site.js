const themeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)")

function currentTheme() {
  return localStorage.getItem("theme") || "system"
}

function applyTheme(theme) {
  const dark =
    theme === "dark" || (theme === "system" && themeMediaQuery.matches)
  document.documentElement.classList.toggle("dark", dark)
  document.documentElement.style.colorScheme = dark ? "dark" : "light"

  const toggle = document.querySelector("[data-theme-toggle]")
  if (!toggle) return

  const label = `${theme[0].toUpperCase()}${theme.slice(1)} theme. Switch theme.`
  toggle.setAttribute("aria-label", label)
  toggle.setAttribute("title", label)
  toggle.querySelector("[data-theme-label]").textContent =
    theme[0].toUpperCase() + theme.slice(1)
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

function updateHomeLink(url = window.location.href) {
  const homeLink = document.querySelector("[data-home-link]")
  if (!homeLink) return

  const isHome = new URL(url, window.location.origin).pathname === "/"
  homeLink.classList.toggle("opacity-0", isHome)
  homeLink.classList.toggle("pointer-events-none", isHome)
}

themeMediaQuery.addEventListener("change", () => {
  if (currentTheme() === "system") applyTheme("system")
})

function initializePage() {
  updateHomeLink()
  initThemeToggle()
}

document.addEventListener("DOMContentLoaded", () => {
  window.mu.init({ target: "main", source: "main", transition: false })
  initializePage()
})

document.addEventListener("mu:after-render", (event) => {
  requestAnimationFrame(() => {
    updateHomeLink(event.detail.finalUrl)
    initializePage()
  })
})
window.addEventListener("popstate", updateHomeLink)
