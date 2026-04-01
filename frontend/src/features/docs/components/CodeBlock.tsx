import { useState } from 'react'
import { CheckIcon, ClipboardIcon } from '@heroicons/react/24/outline'

export function CodeBlock({ code }: { code: string }) {
  const [copied, setCopied] = useState(false)

  const handleCopy = () => {
    navigator.clipboard.writeText(code).then(() => {
      setCopied(true)
      setTimeout(() => setCopied(false), 2000)
    })
  }

  return (
    <div className="docs-code-block-wrapper">
      <pre className="docs-code-block">
        <code>{code}</code>
      </pre>
      <button
        type="button"
        onClick={handleCopy}
        className="docs-copy-btn"
        aria-label={copied ? 'Copied!' : 'Copy code to clipboard'}
        title={copied ? 'Copied!' : 'Copy'}
      >
        {copied ? (
          <CheckIcon className="h-3.5 w-3.5" aria-hidden="true" />
        ) : (
          <ClipboardIcon className="h-3.5 w-3.5" aria-hidden="true" />
        )}
        <span>{copied ? 'Copied' : 'Copy'}</span>
      </button>
    </div>
  )
}
