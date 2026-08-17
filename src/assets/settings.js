const { invoke } = window.__TAURI__.core;
const convertFileSrc = window.__TAURI__.core.convertFileSrc;
const { listen } = window.__TAURI__.event;

const DAYS = ["Mon", "Tue", "Wed", "Thu", "Fri", "Sat", "Sun"];

const pageList = document.getElementById("page-list");
const pageEdit = document.getElementById("page-edit");

const breaksListEl = document.getElementById("breaks-list");
const addBreakBtn = document.getElementById("add-break-btn");

const activeToggle = document.getElementById("active-toggle");
const defaultDisplayModeSelect = document.getElementById("default-display-mode");
const cancelOnCallToggle = document.getElementById("cancel-on-call-toggle");
const showOnAllScreensToggle = document.getElementById("show-on-all-screens-toggle");
const autostartToggle = document.getElementById("autostart-toggle");

const editTitle = document.getElementById("break-edit-title");
const typeSelect = document.getElementById("break-type");
const typeCustomGroup = document.getElementById("break-type-custom-group");
const typeCustomInput = document.getElementById("break-type-custom");
const startTimeInput = document.getElementById("break-start-time");
const durationInput = document.getElementById("break-duration");
const daysContainer = document.getElementById("break-days");
const displayModeSelect = document.getElementById("break-display-mode");
const imagePreview = document.getElementById("break-image-preview");
const imagePickBtn = document.getElementById("break-image-pick-btn");
const imageClearBtn = document.getElementById("break-image-clear-btn");
const messageInput = document.getElementById("break-message");
const enabledInput = document.getElementById("break-enabled");
const backBtn = document.getElementById("break-cancel-btn");

let breaks = [];
let editingId = null;
let selectedDays = new Set();
let selectedImageFilename = null;
let saveTimer = null;

// A blocking alert() here would freeze the whole window — if a burst of
// rapid clicks (e.g. toggling a checkbox repeatedly) produced several
// errors at once, each alert() has to be dismissed before the next runs,
// which reads as the app having hung. A transient on-page toast never
// blocks input.
function reportError(context, err) {
  console.error(context, err);
  const toast = document.createElement("div");
  toast.className = "toast";
  toast.textContent = `${context}: ${err}`;
  document.body.appendChild(toast);
  setTimeout(() => toast.remove(), 4000);
}

// Every #<field>.addEventListener("change", saveBreakNow) fires independently,
// so rapidly toggling a checkbox a few times queues up that many concurrent
// invoke() calls. Chaining them through this queue makes them run one at a
// time instead — each still saves the current (latest) form state when its
// turn comes, so nothing is lost, but there's no longer a pile of overlapping
// backend writes in flight at once.
let saveQueue = Promise.resolve();
function enqueue(fn) {
  const result = saveQueue.then(fn, fn);
  saveQueue = result;
  return result;
}

function dayButtons() {
  daysContainer.innerHTML = "";
  for (const day of DAYS) {
    const btn = document.createElement("button");
    btn.type = "button";
    btn.className = "day-toggle";
    btn.textContent = day;
    btn.addEventListener("click", () => {
      if (selectedDays.has(day)) {
        selectedDays.delete(day);
      } else {
        selectedDays.add(day);
      }
      syncDayButtons();
      saveBreakNow();
    });
    daysContainer.appendChild(btn);
  }
  syncDayButtons();
}

function syncDayButtons() {
  [...daysContainer.children].forEach((btn, i) => {
    btn.classList.toggle("active", selectedDays.has(DAYS[i]));
  });
}

function breakTypeLabel(breakType) {
  if (breakType.kind === "Custom") return breakType.value;
  return breakType.kind;
}

function formatDays(days) {
  return DAYS.filter((d) => days.includes(d)).join(", ");
}

function applySettingsToUI(settings) {
  // Inverted on purpose: the checkbox means "active", the stored value is
  // "paused" — binding it directly (checked = settings.paused) previously
  // meant checking "Breaks are active" actually paused everything.
  activeToggle.checked = !settings.paused;
  defaultDisplayModeSelect.value = settings.default_display_mode;
  cancelOnCallToggle.checked = settings.cancel_on_call;
  showOnAllScreensToggle.checked = settings.show_on_all_screens;
  autostartToggle.checked = settings.autostart;
}

async function loadSettings() {
  try {
    applySettingsToUI(await invoke("get_settings"));
  } catch (err) {
    reportError("Failed to load settings", err);
  }
}

// Settings can also change from outside this window — e.g. Pause/Resume
// from the tray menu — so keep the toggles in sync if that happens while
// this window is open, rather than only reading settings once at load.
listen("settings-changed", (event) => applySettingsToUI(event.payload));

// Queued via `enqueue` for the same reason as saveBreakNow: rapidly
// toggling a switch a few times should save one at a time, not concurrently.
function saveSettings() {
  return enqueue(async () => {
    try {
      const settings = {
        paused: !activeToggle.checked,
        default_display_mode: defaultDisplayModeSelect.value,
        cancel_on_call: cancelOnCallToggle.checked,
        show_on_all_screens: showOnAllScreensToggle.checked,
        autostart: autostartToggle.checked,
      };
      await invoke("update_settings", { settings });
    } catch (err) {
      reportError("Failed to save settings", err);
    }
  });
}

async function loadBreaks() {
  try {
    breaks = await invoke("list_breaks");
    renderBreaks();
  } catch (err) {
    reportError("Failed to load breaks", err);
  }
}

function renderBreaks() {
  breaksListEl.innerHTML = "";
  if (breaks.length === 0) {
    const empty = document.createElement("div");
    empty.className = "empty-state";
    empty.textContent = "No breaks scheduled yet.";
    breaksListEl.appendChild(empty);
    return;
  }

  for (const b of breaks) {
    const row = document.createElement("div");
    row.className = "break-row" + (b.enabled ? "" : " disabled");

    const meta = document.createElement("div");
    meta.className = "meta";
    const typeLabel = document.createElement("div");
    typeLabel.className = "type-label";
    typeLabel.textContent = breakTypeLabel(b.break_type);
    const detail = document.createElement("div");
    detail.className = "detail";
    detail.textContent = `${b.start_time.slice(0, 5)} · ${b.duration_minutes}m · ${formatDays(b.days)}`;
    meta.appendChild(typeLabel);
    meta.appendChild(detail);

    const actions = document.createElement("div");
    actions.className = "actions";
    const editBtn = document.createElement("button");
    editBtn.className = "icon-btn";
    editBtn.title = "Edit";
    editBtn.innerHTML = '<span class="icon icon-pencil"></span>';
    editBtn.addEventListener("click", () => openEditPage(b));
    const deleteBtn = document.createElement("button");
    deleteBtn.className = "icon-btn danger";
    deleteBtn.title = "Delete";
    deleteBtn.innerHTML = '<span class="icon icon-trash"></span>';
    deleteBtn.addEventListener("click", () => deleteBreak(b.id));
    actions.appendChild(editBtn);
    actions.appendChild(deleteBtn);

    row.appendChild(meta);
    row.appendChild(actions);
    breaksListEl.appendChild(row);
  }
}

async function deleteBreak(id) {
  if (!confirm("Delete this break?")) return;
  try {
    await invoke("delete_break", { id });
    await loadBreaks();
  } catch (err) {
    reportError("Failed to delete break", err);
  }
}

async function setImagePreview(filename) {
  selectedImageFilename = filename;
  if (!filename) {
    imagePreview.style.backgroundImage = "";
    return;
  }
  try {
    const path = await invoke("get_image_path", { filename });
    imagePreview.style.backgroundImage = `url("${convertFileSrc(path)}")`;
  } catch (err) {
    reportError("Failed to load image preview", err);
  }
}

function openEditPage(existing) {
  clearTimeout(saveTimer);
  saveTimer = null;

  editingId = existing ? existing.id : null;
  editTitle.textContent = existing ? "Edit break" : "Add break";

  const kind = existing ? existing.break_type.kind : "Hydration";
  typeSelect.value = kind;
  typeCustomInput.value = existing && kind === "Custom" ? existing.break_type.value : "";
  typeCustomGroup.style.display = kind === "Custom" ? "flex" : "none";

  startTimeInput.value = existing ? existing.start_time.slice(0, 5) : "09:00";
  durationInput.value = existing ? existing.duration_minutes : 5;

  selectedDays = new Set(existing ? existing.days : ["Mon", "Tue", "Wed", "Thu", "Fri"]);
  dayButtons();

  displayModeSelect.value = existing && existing.display_mode ? existing.display_mode : "";
  messageInput.value = existing ? existing.message : "Take a break now";
  enabledInput.checked = existing ? existing.enabled : true;

  setImagePreview(existing ? existing.image_filename : null);

  pageList.classList.add("hidden");
  pageEdit.classList.remove("hidden");
}

async function backToList() {
  await flushPendingSave();
  pageEdit.classList.add("hidden");
  pageList.classList.remove("hidden");
  await loadBreaks();
}

function buildBreakPayload() {
  const breakType =
    typeSelect.value === "Custom"
      ? { kind: "Custom", value: typeCustomInput.value || "Custom" }
      : { kind: typeSelect.value };

  // The backend always assigns a fresh id on create and ignores this value
  // in that case — it just needs to be a syntactically valid UUID.
  return {
    id: editingId ?? "00000000-0000-0000-0000-000000000000",
    break_type: breakType,
    start_time: `${startTimeInput.value || "09:00"}:00`,
    duration_minutes: Number(durationInput.value) || 1,
    days: [...selectedDays],
    display_mode: displayModeSelect.value || null,
    image_filename: selectedImageFilename,
    message: messageInput.value || "Take a break now",
    enabled: enabledInput.checked,
  };
}

// There's no Save button — every change persists on its own. Discrete
// controls (selects, checkboxes, day toggles, image picks) save right away;
// free-text fields debounce briefly so we're not writing on every keystroke.
// Queued via `enqueue` so rapid repeated changes save one at a time.
function saveBreakNow() {
  clearTimeout(saveTimer);
  saveTimer = null;

  return enqueue(async () => {
    const payload = buildBreakPayload();
    try {
      if (editingId) {
        await invoke("update_break", { updated: payload });
      } else {
        const created = await invoke("create_break", { newBreak: payload });
        editingId = created.id;
      }
    } catch (err) {
      reportError("Failed to save break", err);
    }
  });
}

function scheduleBreakSave() {
  clearTimeout(saveTimer);
  saveTimer = setTimeout(saveBreakNow, 500);
}

async function flushPendingSave() {
  if (saveTimer) {
    await saveBreakNow();
  }
}

typeSelect.addEventListener("change", () => {
  typeCustomGroup.style.display = typeSelect.value === "Custom" ? "flex" : "none";
  saveBreakNow();
});
typeCustomInput.addEventListener("input", scheduleBreakSave);
startTimeInput.addEventListener("change", saveBreakNow);
durationInput.addEventListener("input", scheduleBreakSave);
displayModeSelect.addEventListener("change", saveBreakNow);
messageInput.addEventListener("input", scheduleBreakSave);
enabledInput.addEventListener("change", saveBreakNow);

imagePickBtn.addEventListener("click", async () => {
  try {
    const filename = await invoke("pick_image");
    if (filename) {
      await setImagePreview(filename);
      await saveBreakNow();
    }
  } catch (err) {
    reportError("Failed to pick image", err);
  }
});

imageClearBtn.addEventListener("click", async () => {
  await setImagePreview(null);
  await saveBreakNow();
});

addBreakBtn.addEventListener("click", () => openEditPage(null));
backBtn.addEventListener("click", backToList);

activeToggle.addEventListener("change", saveSettings);
defaultDisplayModeSelect.addEventListener("change", saveSettings);
cancelOnCallToggle.addEventListener("change", saveSettings);
showOnAllScreensToggle.addEventListener("change", saveSettings);
autostartToggle.addEventListener("change", saveSettings);

loadSettings();
loadBreaks();
