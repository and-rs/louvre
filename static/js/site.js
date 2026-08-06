let activeAnimation

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

function initOrbitDemo() {
  const demo = document.querySelector("[data-orbit-demo]")
  const node = demo?.querySelector("[data-orbit-node]")
  if (!node || node.dataset.initialized) return

  node.dataset.initialized = "true"
  activeAnimation = window.anime.animate(node, {
    rotate: "1turn",
    transformOrigin: "50% 104px",
    duration: 2800,
    ease: "linear",
    loop: true,
  })
}

function initializePage() {
  initOrbitDemo()
  initPerformanceMetrics()
}

document.addEventListener("DOMContentLoaded", () => {
  window.mu.init({ target: "main", source: "main", transition: false })
  initializePage()
})

document.addEventListener("mu:after-render", initializePage)
document.addEventListener("mu:before-render", () => {
  activeAnimation?.cancel()
  activeAnimation = undefined
})
