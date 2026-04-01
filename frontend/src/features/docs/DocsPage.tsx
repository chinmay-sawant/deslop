import { useState } from 'react'

import { DocsLayout } from './components/DocsLayout'
import { type Language, type SectionId } from './docs-content'

export function DocsPage() {
  const [activeLang, setActiveLang] = useState<Language>('go')
  const [activeSection, setActiveSection] = useState<SectionId>('overview')

  return (
    <DocsLayout
      activeLang={activeLang}
      activeSection={activeSection}
      onLangChange={setActiveLang}
      onSectionChange={setActiveSection}
    />
  )
}
