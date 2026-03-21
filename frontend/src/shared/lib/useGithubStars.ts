import { useEffect, useState } from 'react'

type GithubStarsState = {
  stars: number
  isLoading: boolean
}

const initialState: GithubStarsState = {
  stars: 0,
  isLoading: true,
}

export function formatStarCount(stars: number) {
  return new Intl.NumberFormat('en', {
    notation: 'compact',
    maximumFractionDigits: 1,
  }).format(stars)
}

export function useGithubStars(owner: string, repo: string) {
  const [state, setState] = useState<GithubStarsState>(initialState)

  useEffect(() => {
    const controller = new AbortController()

    async function fetchStars() {
      try {
        const response = await fetch(`https://api.github.com/repos/${owner}/${repo}`, {
          headers: {
            Accept: 'application/vnd.github+json',
          },
          signal: controller.signal,
        })

        if (!response.ok) {
          throw new Error('Unable to load GitHub repository metadata')
        }

        const payload = (await response.json()) as { stargazers_count?: number }

        setState({
          stars: typeof payload.stargazers_count === 'number' ? payload.stargazers_count : 0,
          isLoading: false,
        })
      } catch {
        if (controller.signal.aborted) {
          return
        }

        setState({
          stars: 0,
          isLoading: false,
        })
      }
    }

    void fetchStars()

    return () => controller.abort()
  }, [owner, repo])

  return state
}