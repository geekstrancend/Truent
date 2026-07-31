interface AsciiLogoProps {
  className?: string
  glow?: boolean
}

const LOGO = `  ███████╗███████╗███╗   ██╗████████╗██████╗ ██╗
  ██╔════╝██╔════╝████╗  ██║╚══██╔══╝██╔══██╗██║
  ███████╗█████╗  ██╔██╗ ██║   ██║   ██████╔╝██║
  ╚════██║██╔══╝  ██║╚██╗██║   ██║   ██╔══██╗██║
  ███████║███████╗██║ ╚████║   ██║   ██║  ██║██║
  ╚══════╝╚══════╝╚═╝  ╚═══╝   ╚═╝   ╚═╝  ╚═╝╚═╝`

export function AsciiLogo({ className, glow = false }: AsciiLogoProps) {
  const baseClasses = 'font-mono leading-[1.05] whitespace-pre select-none'
  const colorClasses = glow ? 'text-acc-text drop-shadow-[0_0_24px_var(--secondary)]' : 'text-acc-text'
  const finalClassName = `${baseClasses} ${colorClasses} ${className || ''}`

  return (
    <pre
      className={finalClassName}
      aria-label="Truent"
      role="img"
    >
      {LOGO}
    </pre>
  )
}
