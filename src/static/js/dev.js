const stylesheet = document.querySelector("[data-site-stylesheet]")
let stylesheetVersion = ""

async function refreshStylesheet() {
  if (!stylesheet) return

  const response = await fetch(stylesheet.dataset.source, {
    cache: "no-store",
    method: "HEAD",
  })
  const version = `${response.headers.get("last-modified")}:${response.headers.get("content-length")}`

  if (stylesheetVersion && stylesheetVersion !== version) {
    stylesheet.href = `${stylesheet.dataset.source}?v=${Date.now()}`
  }
  stylesheetVersion = version
}

refreshStylesheet()
setInterval(refreshStylesheet, 750)
