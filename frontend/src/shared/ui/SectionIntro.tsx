import { cn } from '../lib/cn'

type SectionIntroProps = {
  eyebrow: string
  title: string
  description: string
  className?: string
}

export function SectionIntro({ eyebrow, title, description, className }: SectionIntroProps) {
  return (
    <div className={cn('max-w-4xl', className)}>
      <span className="eyebrow">{eyebrow}</span>
      <h2 className="mt-6 text-4xl leading-[1.02] font-bold sm:text-5xl lg:text-[3.65rem]">{title}</h2>
      <p className="mt-5 max-w-3xl text-base leading-8 text-[var(--muted)] sm:text-lg">{description}</p>
    </div>
  )
}