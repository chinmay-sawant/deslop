import { quickStartItems } from '../../../content/site-content'

export function QuickStart() {
  return (
    <div className="mt-14 grid gap-5 lg:grid-cols-3">
      {quickStartItems.map((item, index) => (
        <article key={item.label} className="glass-panel rounded-[2rem] p-7 sm:p-8">
          <div className="flex items-center justify-between gap-4">
            <span className="eyebrow">Step 0{index + 1}</span>
            <span className="text-sm text-[var(--muted)]">CLI</span>
          </div>

          <h3 className="mt-6 text-[1.95rem] leading-tight font-bold">{item.label}</h3>
          <p className="mt-4 text-base leading-8 text-[var(--muted)]">{item.description}</p>

          <div className="grid-panel mt-8 overflow-hidden rounded-[1.6rem] p-5">
            <div className="terminal-line font-['IBM_Plex_Mono'] text-[0.78rem] sm:text-[0.9rem]">
              <span className="terminal-prompt">$</span>
              <span className="terminal-copy break-all">{item.command}</span>
            </div>
          </div>
        </article>
      ))}
    </div>
  )
}