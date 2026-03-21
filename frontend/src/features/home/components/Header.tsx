import { Disclosure, DisclosureButton, DisclosurePanel } from '@headlessui/react'
import { ArrowUpRightIcon, Bars3Icon, MoonIcon, SunIcon, XMarkIcon } from '@heroicons/react/24/outline'

import { navigation, siteMetadata } from '../../../content/site-content'
import { cn } from '../../../shared/lib/cn'
import { useGithubStars } from '../../../shared/lib/useGithubStars'
import type { Theme } from '../../../shared/lib/useTheme'
import { Container } from '../../../shared/ui/Container'
import { GitHubStarsBadge } from '../../../shared/ui/GitHubStarsBadge'

function Logo() {
  return (
    <a href="#top" className="flex items-center gap-3">
      <span className="flex h-11 w-11 items-center justify-center rounded-2xl border border-[var(--border)] bg-[var(--accent-soft)] font-['Space_Grotesk'] text-lg font-bold text-[var(--text)]">
        d/
      </span>
      <span className="flex flex-col leading-none">
        <span className="font-['Space_Grotesk'] text-lg font-bold tracking-[-0.05em] text-[var(--text)]">deslop</span>
        <span className="text-[0.76rem] tracking-[0.02em] text-[var(--muted)]">
          Static analysis platform
        </span>
      </span>
    </a>
  )
}

const navLinkClassName =
  'rounded-full px-4 py-2 text-sm font-medium text-[var(--muted)] transition hover:text-[var(--text)] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent)]'

type HeaderProps = {
  theme: Theme
  onToggleTheme: () => void
}

type ThemeToggleButtonProps = HeaderProps & {
  compact?: boolean
}

function ThemeToggleButton({ theme, onToggleTheme, compact = false }: ThemeToggleButtonProps) {
  const nextTheme = theme === 'dark' ? 'light' : 'dark'
  const Icon = theme === 'dark' ? SunIcon : MoonIcon

  return (
    <button
      type="button"
      onClick={onToggleTheme}
      className={cn(
        'inline-flex min-h-11 items-center justify-center gap-2 rounded-full border border-[var(--border)] bg-[var(--panel)] text-[var(--text)] transition hover:border-[var(--border-strong)] hover:bg-[var(--accent-soft)] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent)]',
        compact ? 'h-11 w-11 px-0' : 'px-4',
      )}
      aria-label={`Switch to ${nextTheme} mode`}
      title={`Switch to ${nextTheme} mode`}
    >
      <Icon className="h-5 w-5" aria-hidden="true" />
      {!compact && <span className="hidden lg:inline">{nextTheme === 'light' ? 'Light mode' : 'Dark mode'}</span>}
    </button>
  )
}

export function Header({ theme, onToggleTheme }: HeaderProps) {
  const { stars, isLoading } = useGithubStars(siteMetadata.github.owner, siteMetadata.github.repo)

  return (
    <Disclosure as="header" className="sticky top-0 z-50 border-b border-[var(--border)] bg-[var(--header-bg)] backdrop-blur-xl">
      {({ open }) => (
        <>
          <Container className="flex items-center justify-between gap-4 py-4">
            <div className="flex items-center gap-3">
              <GitHubStarsBadge
                href={siteMetadata.github.url}
                stars={stars}
                isLoading={isLoading}
                className="hidden sm:inline-flex"
              />
              <Logo />
            </div>

            <nav className="hidden items-center gap-1 md:flex">
              {navigation.map((item) => (
                <a key={item.href} href={item.href} className={navLinkClassName}>
                  {item.label}
                </a>
              ))}
            </nav>

            <div className="hidden items-center gap-3 md:flex">
              <ThemeToggleButton theme={theme} onToggleTheme={onToggleTheme} />
              <a href="#quickstart" className="button-primary">
                Quick start
                <ArrowUpRightIcon className="h-4 w-4" aria-hidden="true" />
              </a>
            </div>

            <div className="flex items-center gap-2 md:hidden">
              <ThemeToggleButton theme={theme} onToggleTheme={onToggleTheme} compact />
              <DisclosureButton
                className="flex h-11 w-11 items-center justify-center rounded-full border border-[var(--border)] bg-[var(--panel)] text-[var(--text)] transition hover:bg-[var(--accent-soft)]"
                aria-label={open ? 'Close navigation' : 'Open navigation'}
              >
                {open ? <XMarkIcon className="h-5 w-5" aria-hidden="true" /> : <Bars3Icon className="h-5 w-5" aria-hidden="true" />}
              </DisclosureButton>
            </div>
          </Container>

          <DisclosurePanel className="border-t border-[var(--border)] md:hidden">
            <Container className="pb-5">
              <div className="glass-panel rounded-3xl p-3">
                <GitHubStarsBadge
                  href={siteMetadata.github.url}
                  stars={stars}
                  isLoading={isLoading}
                  className="mb-3 w-full justify-center"
                />
                <div className="flex flex-col gap-1">
                  {navigation.map((item) => (
                    <a key={item.href} href={item.href} className={cn(navLinkClassName, 'px-4 py-3')}>
                      {item.label}
                    </a>
                  ))}
                </div>
                <a href="#quickstart" className="button-primary mt-3 w-full">
                  Quick start
                </a>
              </div>
            </Container>
          </DisclosurePanel>
        </>
      )}
    </Disclosure>
  )
}