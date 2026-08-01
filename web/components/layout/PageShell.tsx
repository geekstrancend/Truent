import clsx from 'clsx'

interface PageShellProps {
  children: React.ReactNode
  /** Radial wash behind the panel. Each screen tunes its own spread. */
  glow?: string
  /** 404 fills the viewport so its footer sits at the bottom. */
  fullHeight?: boolean
  className?: string
}

const DEFAULT_GLOW = 'radial-gradient(1100px 600px at 50% -100px, rgba(52,211,153,0.14), rgba(6,9,8,0) 60%)'

/**
 * The inset, rounded panel every marketing screen sits inside: a 10px gutter of
 * page background around a hairline-bordered card carrying the green wash.
 */
export function PageShell({ children, glow = DEFAULT_GLOW, fullHeight = false, className }: PageShellProps) {
  return (
    <div className="min-h-screen bg-bg p-2.5">
      <div
        className={clsx(
          'relative overflow-clip rounded-[22px] border border-white/[0.05]',
          fullHeight && 'flex min-h-[calc(100vh-20px)] flex-col',
          className,
        )}
        style={{ background: `${glow}, #060908` }}
      >
        {children}
      </div>
    </div>
  )
}
