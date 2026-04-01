import { footerLinks } from '../../../content/site-content'
import { sitePath } from '../../../shared/lib/sitePath'
import { Container } from '../../../shared/ui/Container'

export function Footer() {
  return (
    <footer className="border-t border-[var(--border)] pb-16 pt-14 sm:pb-20 sm:pt-18">
      <Container>
        {/* Main footer grid */}
        <div className="grid gap-12 lg:grid-cols-[1.5fr_1fr] items-start">
          {/* Left: Brand + description + language badges */}
          <div>
            <p className="font-['Newsreader'] italic text-[2.6rem] font-semibold leading-none tracking-[-0.03em] text-[var(--text-strong)]">
              deslop
            </p>
            <p className="mt-4 max-w-md font-['Newsreader'] italic text-[1.15rem] leading-[1.75] text-[var(--muted)]">
              Static analysis for low-context code, focused on readable, repository-aware findings across Go, Python, and Rust repositories.
            </p>

            {/* Language badges */}
            <div className="mt-6 flex flex-wrap gap-2">
              <span className="px-3 py-1 text-[0.7rem] font-medium font-['IBM_Plex_Mono'] tracking-[0.12em] uppercase border border-[var(--lang-go-badge)] text-[var(--lang-go)] bg-[var(--lang-go-soft)]">
                Go
              </span>
              <span className="px-3 py-1 text-[0.7rem] font-medium font-['IBM_Plex_Mono'] tracking-[0.12em] uppercase border border-[var(--lang-python-badge)] text-[var(--lang-python)] bg-[var(--lang-python-soft)]">
                Python
              </span>
              <span className="px-3 py-1 text-[0.7rem] font-medium font-['IBM_Plex_Mono'] tracking-[0.12em] uppercase border border-[var(--lang-rust-badge)] text-[var(--lang-rust)] bg-[var(--lang-rust-soft)]">
                Rust
              </span>
            </div>
          </div>

          {/* Right: Navigation links */}
          <div className="flex flex-col gap-4 lg:items-end lg:pt-1">
            {footerLinks.map((link) => (
              <a
                key={link.href}
                href={link.href.startsWith('http') ? link.href : sitePath(link.href)}
                className="group flex items-center gap-2 text-sm text-[var(--muted)] transition-colors duration-150 hover:text-[var(--text-strong)]"
              >
                <span>{link.label}</span>
                <span className="inline-block transition-transform duration-150 group-hover:translate-x-0.5 opacity-40 group-hover:opacity-100">→</span>
              </a>
            ))}
          </div>
        </div>

        {/* Bottom bar */}
        <div className="mt-14 border-t border-[var(--border)] pt-8 flex flex-col gap-3 sm:flex-row sm:items-center sm:justify-between">
          <p className="text-xs text-[var(--muted)] font-['IBM_Plex_Mono'] tracking-[0.06em]">
            © 2026 deslop · MIT License · Open-source from day one
          </p>
          <p className="text-sm text-[var(--muted)]">
            Built &amp; vibecoded by{' '}
            <a
              href="https://github.com/chinmay-sawant"
              target="_blank"
              rel="noreferrer"
              className="text-[var(--text)] underline decoration-[var(--border)] underline-offset-4 transition-colors duration-150 hover:decoration-[var(--text)]"
            >
              Chinmay Sawant
            </a>{' '}
            with ❤️
          </p>
        </div>
      </Container>
    </footer>
  )
}
