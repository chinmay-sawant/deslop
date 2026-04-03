import { useLayoutEffect, useRef, useState } from 'react'

import { DocsLayout } from './components/DocsLayout'
import { type Language, type SectionId } from './docs-content'

export function DocsPage() {
  const [activeLang, setActiveLang] = useState<Language>('go')
  const [activeSection, setActiveSection] = useState<SectionId>('overview')
  const didMountRef = useRef(false)

  useLayoutEffect(() => {
    if (!didMountRef.current) {
      didMountRef.current = true
      return
    }

    window.scrollTo({ top: 0, left: 0, behavior: 'auto' })
  }, [activeLang, activeSection])

  return (
    <DocsLayout
      activeLang={activeLang}
      activeSection={activeSection}
      onLangChange={setActiveLang}
      onSectionChange={setActiveSection}
    />
  )
}
