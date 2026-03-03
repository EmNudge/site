import type { WebHaptics } from "web-haptics";

let haptics: WebHaptics | null = null;

async function getHaptics() {
  if (haptics) return haptics;

  const { WebHaptics } = await import("web-haptics");
  if (!WebHaptics.isSupported) return null;

  haptics = new WebHaptics();
  return haptics;
}

document.addEventListener("click", (e) => {
  const link = (e.target as HTMLElement).closest("nav a, header a");
  if (!link) return;

  getHaptics().then((h) => h?.trigger("nudge"));
});
