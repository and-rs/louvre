const themeMediaQuery = window.matchMedia("(prefers-color-scheme: dark)")
const initializedElements = new WeakSet()

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
  if (!toggle || initializedElements.has(toggle)) return

  initializedElements.add(toggle)
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

function initArtworkGallery() {
  const gallery = document.querySelector("[data-artwork-gallery]")
  if (!gallery || initializedElements.has(gallery)) return

  const track = gallery.querySelector("[data-gallery-track]")
  const slides = [...gallery.querySelectorAll("[data-gallery-slide]")]
  if (slides.length < 2) return

  initializedElements.add(gallery)
  const thumbnails = [...gallery.querySelectorAll("[data-gallery-select]")]
  const status = gallery.querySelector("[data-gallery-status]")
  let current = 0

  function show(index) {
    current = (index + slides.length) % slides.length
    track.scrollTo({
      left: current * track.clientWidth,
      behavior: window.matchMedia("(prefers-reduced-motion: reduce)").matches
        ? "auto"
        : "smooth",
    })
    updateCurrent()
  }

  function updateCurrent() {
    thumbnails.forEach((button, position) => {
      const active = position === current
      button.setAttribute("aria-pressed", String(active))
      button.classList.toggle("ring-2", active)
    })
    if (status) status.textContent = `Image ${current + 1} of ${slides.length}`
  }

  track.addEventListener(
    "scroll",
    () => {
      current = Math.max(
        0,
        Math.min(
          slides.length - 1,
          Math.round(track.scrollLeft / track.clientWidth),
        ),
      )
      updateCurrent()
    },
    { passive: true },
  )

  gallery
    .querySelector("[data-gallery-previous]")
    ?.addEventListener("click", () => show(current - 1))
  gallery
    .querySelector("[data-gallery-next]")
    ?.addEventListener("click", () => show(current + 1))
  thumbnails.forEach((button) => {
    button.addEventListener("click", () =>
      show(Number(button.dataset.gallerySelect)),
    )
  })
  gallery.addEventListener("keydown", (event) => {
    if (event.target.closest("input, textarea, select")) return
    if (event.key === "ArrowLeft") {
      event.preventDefault()
      show(current - 1)
    } else if (event.key === "ArrowRight") {
      event.preventDefault()
      show(current + 1)
    }
  })

  show(0)
}

themeMediaQuery.addEventListener("change", () => {
  if (currentTheme() === "system") applyTheme("system")
})

function initializePage() {
  initThemeToggle()
  initArtworkGallery()
}

// Deferred scripts can load after htmx has processed the initial document.
initializePage()
document.addEventListener("htmx:after:process", initializePage)
