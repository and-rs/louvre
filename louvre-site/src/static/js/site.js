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

function initBatchImageUploader() {
  const form = document.querySelector("[data-image-uploader]")
  if (!form || initializedElements.has(form)) return

  initializedElements.add(form)
  form.hidden = false
  const input = form.querySelector("[data-batch-files]")
  const queueElement = form.querySelector("[data-upload-queue]")
  const statusElement = form.querySelector("[data-upload-status]")
  const submitButton = form.querySelector("[data-upload-submit]")
  const dropzone = form.querySelector("[data-dropzone]")
  const publishButton = document.querySelector("[data-publish-button]")
  const savedCountElement = document.querySelector("[data-saved-image-count]")
  let savedCount = Number(savedCountElement?.textContent || 0)
  let queue = []
  let uploading = false

  function setStatus(message) {
    statusElement.textContent = message
  }

  function updateButtons() {
    submitButton.disabled =
      uploading ||
      !queue.some((item) => item.state === "queued" || item.state === "failed")
    submitButton.textContent = queue.some((item) => item.state === "failed")
      ? "Upload / retry selected images"
      : "Upload selected images"
    if (publishButton) {
      const unfinished = queue.some((item) => item.state !== "uploaded")
      publishButton.disabled = savedCount === 0 || uploading || unfinished
    }
  }

  function renderQueue() {
    queueElement.replaceChildren()
    queue.forEach((item) => {
      const row = document.createElement("li")
      row.className =
        "flex min-w-0 gap-3 rounded-lg border border-border bg-background p-3"
      row.dataset.uploadName = item.file.name
      row.dataset.uploadId = item.uploadId

      const preview = document.createElement("img")
      preview.src = item.preview
      preview.alt = ""
      preview.className =
        "size-20 shrink-0 rounded-md border border-border bg-muted object-cover"

      const details = document.createElement("div")
      details.className = "min-w-0 flex-1 space-y-2"
      const name = document.createElement("p")
      name.className = "truncate text-sm font-medium"
      name.textContent = item.file.name
      const altLabel = document.createElement("label")
      altLabel.className = "block text-xs text-muted-foreground"
      altLabel.textContent = "Alternative text"
      const altInput = document.createElement("input")
      altInput.type = "text"
      altInput.maxLength = 500
      altInput.required = true
      altInput.value = item.altText
      altInput.disabled = uploading || item.state === "uploaded"
      altInput.setAttribute(
        "aria-label",
        `Alternative text for ${item.file.name}`,
      )
      altInput.className =
        "mt-1 h-9 w-full rounded-md border border-input bg-background px-3 text-sm"
      altInput.addEventListener("input", () => {
        item.altText = altInput.value
      })
      altLabel.append(altInput)
      const message = document.createElement("p")
      message.dataset.uploadMessage = ""
      message.className =
        item.state === "failed"
          ? "text-sm text-destructive"
          : "text-xs text-muted-foreground"
      message.setAttribute("role", item.state === "failed" ? "alert" : "status")
      message.textContent =
        item.error ||
        (item.state === "uploaded"
          ? "Uploaded"
          : item.state === "uploading"
            ? `Uploading… ${item.progress}%`
            : "Ready to upload")
      details.append(name, altLabel, message)

      if (item.state === "uploading") {
        const progress = document.createElement("progress")
        progress.dataset.uploadProgress = ""
        progress.max = 100
        progress.value = item.progress
        progress.className = "h-2 w-full accent-primary"
        details.append(progress)
      }

      const controls = document.createElement("div")
      controls.className = "flex shrink-0 flex-col gap-2"
      if (item.state === "failed") {
        const retry = document.createElement("button")
        retry.type = "button"
        retry.textContent = "Retry"
        retry.className =
          "rounded-md border border-border px-3 py-2 text-sm font-medium hover:bg-accent disabled:opacity-50"
        retry.disabled = uploading
        retry.addEventListener("click", uploadPending)
        controls.append(retry)
      }
      if (item.state !== "uploading" && item.state !== "uploaded") {
        const remove = document.createElement("button")
        remove.type = "button"
        remove.textContent = "Remove"
        remove.className =
          "rounded-md px-3 py-2 text-sm text-muted-foreground hover:bg-accent"
        remove.disabled = uploading
        remove.addEventListener("click", () => {
          URL.revokeObjectURL(item.preview)
          queue = queue.filter((queued) => queued.uploadId !== item.uploadId)
          renderQueue()
        })
        controls.append(remove)
      }
      row.append(preview, details, controls)
      queueElement.append(row)
    })
    updateButtons()
  }

  function addFiles(files) {
    for (const file of files) {
      if (
        queue.some(
          (item) =>
            item.file.name === file.name &&
            item.file.size === file.size &&
            item.file.lastModified === file.lastModified,
        )
      )
        continue
      const item = {
        file,
        uploadId: crypto.randomUUID(),
        preview: URL.createObjectURL(file),
        altText: "",
        state: file.size > 10 * 1024 * 1024 ? "failed" : "queued",
        progress: 0,
        error:
          file.size > 10 * 1024 * 1024
            ? "This file is over the 10 MB limit."
            : "",
      }
      queue.push(item)
    }
    input.value = ""
    renderQueue()
    setStatus(
      `${queue.length} image${queue.length === 1 ? "" : "s"} in the upload queue.`,
    )
  }

  function uploadItem(item) {
    if (uploading || item.state === "uploaded") return Promise.resolve(false)
    if (!item.altText.trim()) {
      item.state = "failed"
      item.error = "Add alternative text before uploading."
      renderQueue()
      return Promise.resolve(false)
    }
    if (item.file.size > 10 * 1024 * 1024) {
      item.state = "failed"
      item.error = "This file is over the 10 MB limit."
      renderQueue()
      return Promise.resolve(false)
    }

    uploading = true
    item.state = "uploading"
    item.error = ""
    item.progress = 0
    renderQueue()
    setStatus(`Uploading ${item.file.name}…`)

    return new Promise((resolve) => {
      const request = new XMLHttpRequest()
      request.open("POST", form.action)
      request.upload.addEventListener("progress", (event) => {
        if (event.lengthComputable) {
          item.progress = Math.round((event.loaded / event.total) * 100)
          const row = queueElement.querySelector(
            `[data-upload-id="${item.uploadId}"]`,
          )
          row.querySelector("[data-upload-progress]").value = item.progress
          row.querySelector("[data-upload-message]").textContent =
            `Uploading… ${item.progress}%`
        }
      })
      request.addEventListener("load", () => {
        uploading = false
        const redirectedToLogin =
          new URL(request.responseURL).pathname === "/studio/login"
        if (
          request.status >= 200 &&
          request.status < 300 &&
          !redirectedToLogin
        ) {
          item.state = "uploaded"
          item.progress = 100
          savedCount += 1
          setStatus(`${item.file.name} uploaded successfully.`)
          resolve(true)
        } else {
          item.state = "failed"
          const response = new DOMParser().parseFromString(
            request.responseText,
            "text/html",
          )
          item.error = redirectedToLogin
            ? "Your Studio session expired. Sign in again, then retry this image."
            : response.querySelector('[role="alert"]')?.textContent.trim() ||
              `Upload failed (${request.status}). Retry this image.`
          setStatus(
            `${item.file.name} failed. Your other images are unchanged; retry it below.`,
          )
          resolve(false)
        }
        renderQueue()
      })
      request.addEventListener("error", () => {
        uploading = false
        item.state = "failed"
        item.error =
          "Network error. Retry this image; completed uploads are preserved."
        setStatus(item.error)
        renderQueue()
        resolve(false)
      })
      const data = new FormData()
      data.append("image", item.file, item.file.name)
      data.append("alt_text", item.altText.trim())
      data.append("upload_id", item.uploadId)
      request.send(data)
    })
  }

  input.addEventListener("change", () => addFiles(input.files))
  dropzone.hidden = false
  dropzone.classList.remove("hidden")
  form.addEventListener("dragover", (event) => event.preventDefault())
  form.addEventListener("drop", (event) => {
    event.preventDefault()
    addFiles(event.dataTransfer.files)
  })
  async function uploadPending() {
    if (uploading) return
    const pending = queue.filter(
      (item) => item.state === "queued" || item.state === "failed",
    )
    for (const item of pending) {
      const success = await uploadItem(item)
      if (!success) return
    }
    updateButtons()
    if (queue.length > 0 && queue.every((item) => item.state === "uploaded")) {
      setStatus(
        "All selected images uploaded. Refreshing the artwork image list…",
      )
      window.location.reload()
    }
  }

  form.addEventListener("submit", (event) => {
    event.preventDefault()
    uploadPending()
  })

  updateButtons()
}

themeMediaQuery.addEventListener("change", () => {
  if (currentTheme() === "system") applyTheme("system")
})

function initializePage() {
  initThemeToggle()
  initArtworkGallery()
  initBatchImageUploader()
}

// Deferred scripts can load after htmx has processed the initial document.
initializePage()
document.addEventListener("htmx:after:process", initializePage)

document.addEventListener("htmx:before:response", (event) => {
  const { ctx } = event.detail
  // History refetches can redirect to sign-in after the session expires.
  if (
    ctx.request.headers["HX-History-Restore-Request"] &&
    ctx.response.raw.redirected
  ) {
    ctx.replace = "true"
  }
})
