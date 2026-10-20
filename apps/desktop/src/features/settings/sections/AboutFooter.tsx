// App identity footer: name/version/tagline + repository link.
import { APP_NAME, APP_TAGLINE, APP_VERSION, REPO_URL } from "../../../lib/branding";

export function AboutFooter() {
  return (
    <footer className="mt-10 border-t border-border pt-4 text-xs text-text-dim">
      <p>
        {APP_NAME} v{APP_VERSION} · {APP_TAGLINE}
      </p>
      <p className="mt-1">
        <a href={REPO_URL} className="underline hover:text-text" target="_blank" rel="noreferrer">
          {REPO_URL}
        </a>
      </p>
    </footer>
  );
}
