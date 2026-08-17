const { invoke } = window.__TAURI__.core;
const convertFileSrc = window.__TAURI__.core.convertFileSrc;
const getCurrentWindow = window.__TAURI__.window.getCurrentWindow;

const breakInfo = window.__BREAK__ || {
  breakId: null,
  typeLabel: "Break",
  message: "Take a break now",
  imageFilename: null,
  durationMinutes: 5,
};

const messageEl = document.getElementById("message");
const countdownValueEl = document.getElementById("countdown-value");
const closeBtn = document.getElementById("close-btn");
const skipBtn = document.getElementById("skip-btn");
const postponeBtn = document.getElementById("postpone-btn");
const postponeMenu = document.getElementById("postpone-menu");

messageEl.textContent = breakInfo.message || "Take a break now";

// The countdown is recomputed from timestamps each tick (rather than
// decremented) so it stays correct even if the webview's JS timers get
// throttled for a moment.
const endsAt = Date.now() + breakInfo.durationMinutes * 60 * 1000;

function formatCountdown(msRemaining) {
  const totalSeconds = Math.max(0, Math.round(msRemaining / 1000));
  const minutes = Math.floor(totalSeconds / 60);
  const seconds = totalSeconds % 60;
  return `${minutes}:${String(seconds).padStart(2, "0")}`;
}

function tick() {
  const remaining = endsAt - Date.now();
  countdownValueEl.textContent = formatCountdown(remaining);
  if (remaining <= 0) {
    clearInterval(intervalId);
    getCurrentWindow().close();
  }
}

const intervalId = setInterval(tick, 250);
tick();

if (breakInfo.imageFilename) {
  invoke("get_image_path", { filename: breakInfo.imageFilename }).then((path) => {
    document.body.style.backgroundImage = `url("${convertFileSrc(path)}")`;
  });
}

async function skipBreak() {
  await invoke("cancel_break", { breakId: breakInfo.breakId });
}

postponeBtn.addEventListener("click", () => {
  postponeMenu.classList.toggle("hidden");
});

postponeMenu.addEventListener("click", async (e) => {
  const minutesAttr = e.target.getAttribute("data-minutes");
  if (!minutesAttr) return;
  await invoke("postpone_break", { breakId: breakInfo.breakId, minutes: Number(minutesAttr) });
});

skipBtn.addEventListener("click", skipBreak);
closeBtn.addEventListener("click", skipBreak);
