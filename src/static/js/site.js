let activeAnimations = []
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

themeMediaQuery.addEventListener("change", () => {
  if (currentTheme() === "system") applyTheme("system")
})

function initPerformanceMetrics() {
  const panel = document.querySelector("[data-performance-metrics]")
  if (!panel || panel.dataset.initialized) return

  panel.dataset.initialized = "true"
  const values = Object.fromEntries(
    [...panel.querySelectorAll("[data-metric]")].map((metric) => [
      metric.dataset.metric,
      metric.querySelector("[data-metric-value]"),
    ]),
  )
  const setMetric = (key, value, unit = "ms") => {
    const target = values[key]
    if (target) target.textContent = value === null ? "N/A" : `${value}${unit}`
  }

  const navigation = performance.getEntriesByType("navigation")[0]
  if (navigation) {
    setMetric("TTFB", Math.round(navigation.responseStart))
    setMetric("DOM_READY", Math.round(navigation.domContentLoadedEventEnd))
  } else {
    setMetric("TTFB", null)
    setMetric("DOM_READY", null)
  }

  const updatePaint = (entries) => {
    for (const entry of entries) {
      if (entry.name === "first-paint")
        setMetric("FP", Math.round(entry.startTime))
      if (entry.name === "first-contentful-paint") {
        setMetric("FCP", Math.round(entry.startTime))
      }
    }
  }

  updatePaint(performance.getEntriesByType("paint"))
  const supported = PerformanceObserver.supportedEntryTypes
  if (!supported.includes("paint")) {
    setMetric("FP", null)
    setMetric("FCP", null)
  } else {
    new PerformanceObserver((list) => updatePaint(list.getEntries())).observe({
      type: "paint",
      buffered: true,
    })
  }

  if (!supported.includes("largest-contentful-paint")) {
    setMetric("LCP", null)
  } else {
    new PerformanceObserver((list) => {
      const entries = list.getEntries()
      const latest = entries[entries.length - 1]
      if (latest) setMetric("LCP", Math.round(latest.startTime))
    }).observe({ type: "largest-contentful-paint", buffered: true })
  }

  if (!supported.includes("layout-shift")) {
    setMetric("CLS", null, "")
  } else {
    let cls = 0
    setMetric("CLS", cls, "")
    new PerformanceObserver((list) => {
      for (const entry of list.getEntries()) {
        if (!entry.hadRecentInput) cls += entry.value
      }
      setMetric("CLS", cls.toFixed(3), "")
    }).observe({ type: "layout-shift", buffered: true })
  }
}

function initSpaAnimation() {
  const spa = document.querySelector("[data-spa]")
  const traceOne = spa?.querySelector("[data-spa-trace-one]")
  const traceTwo = spa?.querySelector("[data-spa-trace-two]")
  const motionPath = spa?.querySelector("#spa-motion-path")
  const car = spa?.querySelector("[data-spa-car]")
  if (
    !spa ||
    !traceOne ||
    !traceTwo ||
    !motionPath ||
    !car ||
    spa.dataset.initialized
  )
    return

  spa.dataset.initialized = "true"
  const trail = 0.1
  const totalLife = 4000
  const phase = totalLife / (1 + trail)
  const entry = trail * phase
  const transit = (1 - trail) * phase
  const [firstTrace] = window.anime.svg.createDrawable(traceOne)
  const [secondTrace] = window.anime.svg.createDrawable(traceTwo)

  const motion = window.anime.svg.createMotionPath(motionPath)

  activeAnimations.push(
    window.anime.animate([traceOne, traceTwo], {
      opacity: [0, 1],
      duration: 1000,
      delay: 200,
      ease: "inOutCirc",
    }),
  )
  activeAnimations.push(
    window.anime.animate(car, {
      opacity: [0, 1],
      duration: 1000,
      delay: 200,
      ease: "inOutCirc",
    }),
  )
  activeAnimations.push(
    window.anime
      .createTimeline({
        loop: true,
        duration: phase * 2,
        defaults: { ease: "linear" },
      })
      .label("first", 0)
      .label("second", phase)
      .add(
        firstTrace,
        { draw: [`0 0`, `0 ${trail}`], duration: entry },
        "first",
      )
      .add(
        firstTrace,
        { draw: [`0 ${trail}`, `${1 - trail} 1`], duration: transit },
        entry,
      )
      .add(
        firstTrace,
        { draw: [`${1 - trail} 1`, "1 1"], duration: entry },
        "second",
      )
      .add(
        secondTrace,
        { draw: [`0 0`, `0 ${trail}`], duration: entry },
        "second",
      )
      .add(
        secondTrace,
        { draw: [`0 ${trail}`, `${1 - trail} 1`], duration: transit },
        phase + entry,
      )
      .add(
        secondTrace,
        { draw: [`${1 - trail} 1`, "1 1"], duration: entry },
        "first",
      )
      .add(car, { ...motion, duration: phase }, "first")
      .add(car, { ...motion, duration: phase }, "second"),
  )
}

function initLogoAnimation() {
  const logo = document.querySelector("[data-logo]")
  if (!logo || logo.dataset.initialized) return

  logo.dataset.initialized = "true"
  let rotations = 0
  logo.addEventListener("click", () => {
    rotations += 1
    activeAnimations.push(
      window.anime.animate(logo, {
        rotate: rotations * 360,
        duration: 1500,
        ease: "out(4)",
      }),
    )
  })
}

function initializePage() {
  initThemeToggle()
  initSpaAnimation()
  initLogoAnimation()
  initPerformanceMetrics()
}

document.addEventListener("DOMContentLoaded", () => {
  window.mu.init({ target: "main", source: "main", transition: false })
  initializePage()
})

document.addEventListener("mu:after-render", initializePage)
document.addEventListener("mu:before-render", () => {
  for (const animation of activeAnimations) animation.cancel()
  activeAnimations = []
})
