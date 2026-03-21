import { formatStarCount } from '../lib/useGithubStars'
import { cn } from '../lib/cn'

type GitHubStarsBadgeProps = {
  href: string
  stars: number
  isLoading: boolean
  className?: string
}

function GitHubMark({ className }: { className?: string }) {
  return (
    <svg viewBox="0 0 24 24" fill="currentColor" className={className} aria-hidden="true">
      <path d="M12 1.25a10.75 10.75 0 0 0-3.4 20.95c.54.1.73-.23.73-.52 0-.25-.01-1.08-.02-1.96-2.97.65-3.6-1.26-3.6-1.26-.49-1.23-1.18-1.56-1.18-1.56-.96-.66.07-.64.07-.64 1.06.08 1.62 1.1 1.62 1.1.95 1.62 2.47 1.16 3.07.89.09-.69.37-1.16.67-1.42-2.37-.27-4.86-1.19-4.86-5.28 0-1.16.41-2.11 1.09-2.86-.11-.27-.47-1.36.1-2.83 0 0 .89-.29 2.93 1.09a10.1 10.1 0 0 1 5.34 0c2.03-1.38 2.92-1.09 2.92-1.09.58 1.47.22 2.56.11 2.83.68.75 1.09 1.7 1.09 2.86 0 4.1-2.5 5-4.88 5.27.39.34.73 1 .73 2.03 0 1.47-.01 2.66-.01 3.02 0 .29.19.63.74.52A10.75 10.75 0 0 0 12 1.25Z" />
    </svg>
  )
}

export function GitHubStarsBadge({ href, stars, isLoading, className }: GitHubStarsBadgeProps) {
  const displayStars = isLoading ? '...' : formatStarCount(stars)
  const ariaLabel = isLoading ? 'GitHub stars are loading' : `View the repository on GitHub. ${stars} stars.`

  return (
    <a
      href={href}
      target="_blank"
      rel="noreferrer"
      className={cn('github-badge', className)}
      aria-label={ariaLabel}
    >
      <GitHubMark className="h-4 w-4" />
      <span className="font-['IBM_Plex_Mono'] text-[0.82rem] font-medium tracking-[0.04em]">{displayStars}</span>
    </a>
  )
}