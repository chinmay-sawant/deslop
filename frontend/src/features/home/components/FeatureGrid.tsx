import { detectionFamilies } from '../../../content/site-content'

export function FeatureGrid() {
  return (
    <div className="mt-20 flex flex-col gap-24">
      {detectionFamilies.map((family, index) => {
        const numberStr = String(index + 1).padStart(2, '0')

        return (
          <article key={family.title} className="flex flex-col border-t border-[var(--border-strong)] pt-12 md:flex-row md:items-start md:gap-16">
            <span className="font-['Newsreader'] text-6xl italic text-[var(--muted)] opacity-50 mb-6 md:mb-0">
              {numberStr}
            </span>

            <div className="flex-1">
              <div className="min-w-0">
                <h3 className="text-[2.5rem] leading-tight font-medium text-[var(--text-strong)]">{family.title}</h3>
              </div>

              <p className="mt-5 max-w-2xl text-lg leading-relaxed text-[var(--muted)]">{family.description}</p>

              <br/>
              <ul className="mt-10 flex flex-wrap gap-3">
                {family.rules.map((rule) => (
                  <li
                    key={rule}
                    className="font-['IBM_Plex_Mono'] text-[0.8rem] uppercase tracking-[0.15em] text-[var(--text-strong)] border border-[var(--border)] px-4 py-2"
                  >
                    {rule}
                  </li>
                ))}
              </ul>
            </div>
          </article>
        )
      })}
    </div>
  )
}