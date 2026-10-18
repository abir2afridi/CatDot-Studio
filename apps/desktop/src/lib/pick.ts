// Shared native file-picker helper (Part E4: dialog plugin, pick-only).
// Used by the Explorer, the Dashboard, and command-palette import commands so
// the dialog filters live in one place.
import { open as openFileDialog } from "@tauri-apps/plugin-dialog";

export const ANDROID_FILTERS = [
  { name: "Android packages", extensions: ["apk", "xapk", "apks", "apkm"] },
  { name: "All files", extensions: ["*"] },
];

/** Returns the picked absolute path, or null when cancelled. */
export async function pickApkPath(title: string): Promise<string | null> {
  const picked = await openFileDialog({
    multiple: false,
    directory: false,
    title,
    filters: ANDROID_FILTERS,
  });
  return typeof picked === "string" && picked !== "" ? picked : null;
}
