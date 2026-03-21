import { detectionFamilies } from '../../../content/site-content'

export function FeatureGrid() {
  return (
    <div className="mt-14 grid gap-5 md:grid-cols-2 xl:grid-cols-3">
      {detectionFamilies.map((family) => {
        const Icon = family.icon

        return (
          <article key={family.title} className="glass-panel rounded-[2rem] p-7 sm:p-8">
            <div className="flex items-start gap-4">
              <span className="icon-badge">
                <Icon className="h-6 w-6" aria-hidden="true" />
              </span>

              <div className="min-w-0 flex-1">
                <h3 className="text-[1.95rem] leading-tight font-bold">{family.title}</h3>
              </div>
            </div>

            <p className="mt-5 text-base leading-8 text-[var(--muted)]">{family.description}</p>

            <ul className="mt-8 flex flex-wrap gap-2.5 border-t border-[var(--border)] pt-6">
              {family.rules.map((rule) => (
                <li
                  key={rule}
                  className="surface-chip px-3.5 py-1.5 text-[0.78rem]"
                >
                  {rule}
                </li>
              ))}
            </ul>
          </article>
        )
      })}
    </div>
  )
}