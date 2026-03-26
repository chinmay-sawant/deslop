import { Disclosure, DisclosureButton, DisclosurePanel } from '@headlessui/react'
import { ArrowUpRightIcon, Bars3Icon, MoonIcon, SunIcon, XMarkIcon } from '@heroicons/react/24/outline'
import { Link, useLocation, useNavigate } from 'react-router-dom'

import { navigation, siteMetadata } from '../../../content/site-content'
import { cn } from '../../../shared/lib/cn'
import { useGithubStars } from '../../../shared/lib/useGithubStars'
import type { Theme } from '../../../shared/lib/useTheme'
import { Container } from '../../../shared/ui/Container'
import { GitHubStarsBadge } from '../../../shared/ui/GitHubStarsBadge'

function Logo() {
  return (
    <Link to="/" className="flex items-center gap-3">
      <span className="flex h-11 w-11 items-center justify-center border-2 border-[var(--text)] bg-[var(--text)] font-['Newsreader'] italic text-2xl font-bold text-[var(--bg)]">
        d/
      </span>
      <span className="flex flex-col leading-none">
        <span className="font-['Newsreader'] italic text-2xl font-bold tracking-[-0.02em] text-[var(--text)]">deslop</span>
        <span className="text-[0.76rem] tracking-[0.02em] text-[var(--muted)] mt-1">
          Go code review signals
        </span>
      </span>
    </Link>
  )
}

const navLinkClassName =
  'px-4 py-2 text-sm font-medium text-[var(--muted)] transition hover:text-[var(--text)] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent)]'

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
        'inline-flex min-h-11 items-center justify-center gap-2 border-2 border-[var(--border)] bg-transparent text-[var(--text)] transition hover:border-[var(--text)] hover:bg-[var(--text)] hover:text-[var(--bg)] focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-[var(--accent)]',
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
  const location = useLocation()
  const navigate = useNavigate()

  const handleInstallClick = (e: React.MouseEvent<HTMLAnchorElement>) => {
    e.preventDefault()
    if (location.pathname === '/' || location.pathname === '' || location.pathname === '/deslop' || location.pathname === '/deslop/') {
      const el = document.getElementById('install-run')
      if (el) {
        el.scrollIntoView({ behavior: 'smooth' })
        window.history.pushState(null, '', '/deslop/#install-run')
      }
    } else {
      // If we are on /docs, navigate back to home
      navigate('/')
      // Small delay to allow react to render the homepage before scrolling natively
      setTimeout(() => {
        document.getElementById('install-run')?.scrollIntoView({ behavior: 'smooth' })
        window.history.pushState(null, '', '/deslop/#install-run')
      }, 100)
    }
  }

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
                item.href.includes('#') ? (
                  <a key={item.href} href={item.href} className={navLinkClassName} onClick={handleInstallClick}>
                    {item.label}
                  </a>
                ) : (
                  <Link key={item.href} to={item.href} className={navLinkClassName}>
                    {item.label}
                  </Link>
                )
              ))}
            </nav>

            <div className="hidden items-center gap-3 md:flex">
              <ThemeToggleButton theme={theme} onToggleTheme={onToggleTheme} />
              <a 
                href="/#install-run" 
                className="button-primary"
                onClick={handleInstallClick}
              >
                Install and run
                <ArrowUpRightIcon className="h-4 w-4" aria-hidden="true" />
              </a>
            </div>

            <div className="flex items-center gap-2 md:hidden">
              <ThemeToggleButton theme={theme} onToggleTheme={onToggleTheme} compact />
              <DisclosureButton
                className="flex h-11 w-11 items-center justify-center border-2 border-[var(--border)] bg-transparent text-[var(--text)] transition hover:border-[var(--text)] hover:bg-[var(--text)] hover:text-[var(--bg)]"
                aria-label={open ? 'Close navigation' : 'Open navigation'}
              >
                {open ? <XMarkIcon className="h-5 w-5" aria-hidden="true" /> : <Bars3Icon className="h-5 w-5" aria-hidden="true" />}
              </DisclosureButton>
            </div>
          </Container>

          <DisclosurePanel className="border-t border-[var(--border)] md:hidden">
            <Container className="pb-5">
              <div className="glass-panel p-3 border-x-0 border-b-2">
                <GitHubStarsBadge
                  href={siteMetadata.github.url}
                  stars={stars}
                  isLoading={isLoading}
                  className="mb-3 w-full justify-center"
                />
                <div className="flex flex-col gap-1">
                  {navigation.map((item) => (
                    item.href.includes('#') ? (
                      <a key={item.href} href={item.href} className={cn(navLinkClassName, 'px-4 py-3')} onClick={handleInstallClick}>
                        {item.label}
                      </a>
                    ) : (
                      <Link key={item.href} to={item.href} className={cn(navLinkClassName, 'px-4 py-3')}>
                        {item.label}
                      </Link>
                    )
                  ))}
                </div>
                <a 
                  href="/#install-run" 
                  className="button-primary mt-3 w-full"
                  onClick={handleInstallClick}
                >
                  Install and run
                </a>
              </div>
            </Container>
          </DisclosurePanel>
        </>
      )}
    </Disclosure>
  )
}