import { metrics } from '../../../content/site-content'
import { Container } from '../../../shared/ui/Container'

export function MetricsBar() {
  return (
    <section className="py-16 sm:py-20 border-y border-[var(--border)]">
      <Container>
        <div className="grid grid-cols-1 gap-10 sm:grid-cols-3">
          {metrics.map((metric) => (
            <div key={metric.label} className="border-l-2 border-[var(--border-strong)] pl-6">
              <p className="text-[2.2rem] sm:text-[2.5rem] font-['Newsreader'] italic font-semibold leading-none tracking-[-0.03em] text-[var(--text-strong)]">
                {metric.value}
              </p>
              <p className="mt-3 font-['IBM_Plex_Mono'] text-[0.72rem] font-semibold uppercase tracking-[0.12em] text-[var(--text-strong)]">
                {metric.label}
              </p>
              <p className="mt-2 text-sm leading-relaxed text-[var(--muted)]">
                {metric.note}
              </p>
            </div>
          ))}
        </div>
      </Container>
    </section>
  )
}
