'use client'

import { useEffect, useRef } from 'react'

/**
 * Scroll-scrubbed particle hero.
 *
 * One `progress` value (0 → 1, driven by scroll through the pinned section)
 * choreographs the whole thing:
 *
 * - **0** — particles are scattered across the full viewport as ambient dust,
 *   and the headline is set huge, fit to the measure.
 * - **→1** — the dust converges into the wordmark while the headline shrinks
 *   and rises out of its way.
 * - **1** — the wordmark is resolved and the supporting copy has faded in.
 *
 * Everything animates imperatively inside a single rAF loop — the headline is
 * written via `style`, never React state, so scrolling never triggers a
 * re-render.
 *
 * The headline is a real `<h1>`, scaled rather than drawn, so it stays
 * selectable and legible to crawlers and screen readers; the canvas is purely
 * decorative and marked `aria-hidden`.
 */

type Particle = {
  /** scattered origin, in CSS px */
  hx: number
  hy: number
  /** glyph-pixel target */
  tx: number
  ty: number
  amp: number
  driftPhase: number
  driftSpeed: number
  twPhase: number
  twSpeed: number
}

interface ParticleHeroProps {
  /** Word the particles resolve into. */
  wordmark?: string
  eyebrow?: string
  headline: string
  /** Mono line revealed once the wordmark resolves. */
  subline?: React.ReactNode
  /** Revealed alongside the subline. */
  actions?: React.ReactNode
  accent?: string
  base?: string
}

const rand = (min: number, max: number) => min + Math.random() * (max - min)
const clamp = (v: number, lo: number, hi: number) => Math.max(lo, Math.min(hi, v))
/** power3.out — quick departure, long settle. */
const easeOutCubic = (x: number) => 1 - Math.pow(1 - x, 3)

export function ParticleHero({
  wordmark = 'TRUENT',
  eyebrow,
  headline,
  subline,
  actions,
  accent = '#34D399',
  base = '#9ab4a9',
}: ParticleHeroProps) {
  const sectionRef = useRef<HTMLElement | null>(null)
  const stageRef = useRef<HTMLDivElement | null>(null)
  const canvasRef = useRef<HTMLCanvasElement | null>(null)
  const headlineRef = useRef<HTMLHeadingElement | null>(null)
  const copyRef = useRef<HTMLDivElement | null>(null)
  const revealRef = useRef<HTMLDivElement | null>(null)

  useEffect(() => {
    const section = sectionRef.current
    const stage = stageRef.current
    const canvas = canvasRef.current
    const h1 = headlineRef.current
    const copy = copyRef.current
    const reveal = revealRef.current
    const ctx = canvas?.getContext('2d')
    if (!section || !stage || !canvas || !ctx || !h1 || !copy || !reveal) return

    const reduced = window.matchMedia('(prefers-reduced-motion: reduce)').matches
    const dpr = Math.min(window.devicePixelRatio || 1, 2)

    let width = 0
    let height = 0
    let dust: Particle[] = []
    let sparks: Particle[] = []
    let raf = 0
    let visible = true
    let disposed = false
    /** Headline size at each end of the tween. */
    let bigSize = 0
    let smallSize = 0

    const displayFont = () =>
      getComputedStyle(document.documentElement).getPropertyValue('--font-display').trim() ||
      'serif'

    /** Fit the headline to the measure, and record both tween endpoints. */
    const measureHeadline = () => {
      const probe = document.createElement('canvas').getContext('2d')
      if (!probe) return
      const font = displayFont()
      const trial = 100
      probe.font = `700 ${trial}px ${font}`
      const w = probe.measureText(headline).width
      if (w <= 0) return
      const maxWidth = Math.min(width * 0.92, 1240)
      // Cap so a short headline doesn't become absurd on a wide screen.
      bigSize = Math.min((maxWidth / w) * trial, height * 0.3, 190)
      smallSize = clamp(width * 0.038, 30, 56)
    }

    /** Rasterise the wordmark and sample it into particle targets. */
    const build = () => {
      // The nav is sticky and in-flow, so it occupies the top of the viewport
      // in both states. Fit the stage to what is actually visible beneath it,
      // measured rather than hardcoded, or the hero overflows the fold and its
      // contents sit low by the nav's height.
      const nav = document.querySelector('nav')
      const navH = nav ? Math.round(nav.getBoundingClientRect().height) : 0
      stage.style.height = `calc(100vh - ${navH}px)`
      stage.style.top = `${navH}px`

      const rect = stage.getBoundingClientRect()
      width = rect.width
      height = rect.height
      if (width <= 0 || height <= 0) return

      canvas.width = Math.floor(width * dpr)
      canvas.height = Math.floor(height * dpr)
      canvas.style.width = `${width}px`
      canvas.style.height = `${height}px`
      ctx.setTransform(dpr, 0, 0, dpr, 0, 0)

      measureHeadline()

      const off = document.createElement('canvas')
      off.width = Math.floor(width)
      off.height = Math.floor(height)
      const octx = off.getContext('2d', { willReadFrequently: true })
      if (!octx) return

      // Place the wordmark in the gap actually left between the resolved
      // headline and the supporting copy, measured rather than assumed: on a
      // narrow screen the copy wraps to several lines and a fixed fraction
      // would drive the word straight through it.
      const prevSize = h1.style.fontSize
      h1.style.fontSize = `${smallSize}px`
      const copyHeight = copy.offsetHeight
      h1.style.fontSize = prevSize

      const gap = Math.max(height * 0.03, 16)
      // At rest the headline block is centred, then lifted by 17% of height.
      const headlineBottom = height * 0.33 + copyHeight / 2
      const revealTop = height - height * 0.12 - reveal.offsetHeight
      const bandTop = headlineBottom + gap
      const bandBottom = Math.max(revealTop - gap, bandTop + 40)
      const bandHeight = bandBottom - bandTop
      const centreY = (bandTop + bandBottom) / 2

      const font = displayFont()
      // Font size bounds neither rendered width nor height, so fit both.
      let size = Math.min(bandHeight * 0.92, height * 0.3, 300)
      octx.font = `700 ${size}px ${font}`
      const measured = octx.measureText(wordmark).width
      const maxWordmark = width * (width < 640 ? 0.84 : 0.56)
      if (measured > maxWordmark && measured > 0) {
        size = Math.max((maxWordmark / measured) * size, 12)
      }
      octx.fillStyle = '#fff'
      octx.font = `700 ${size}px ${font}`
      octx.textAlign = 'center'
      octx.textBaseline = 'middle'
      octx.fillText(wordmark, width / 2, centreY)

      const data = octx.getImageData(0, 0, off.width, off.height).data
      // 3px sampling: a serif face has thin strokes that sample too sparsely
      // at 4px, leaving the resolved word looking dotted-out rather than set.
      const hits: Array<[number, number]> = []
      for (let y = 0; y < off.height; y += 3) {
        for (let x = 0; x < off.width; x += 3) {
          if (data[(y * off.width + x) * 4 + 3] > 128) hits.push([x, y])
        }
      }

      const cap = window.innerWidth < 768 ? 2000 : 5400
      const stride = hits.length > cap ? hits.length / cap : 1
      const count = Math.min(hits.length, cap)

      dust = []
      sparks = []
      for (let i = 0; i < count; i++) {
        const [tx, ty] = hits[Math.floor(i * stride)]
        const p: Particle = {
          // Scattered across the whole stage: at rest this reads as ambient
          // dust filling the viewport, not a cloud waiting to become a word.
          hx: Math.random() * width,
          hy: Math.random() * height,
          tx,
          ty,
          amp: rand(6, 20),
          driftPhase: rand(0, Math.PI * 2),
          driftSpeed: rand(0.25, 0.8),
          twPhase: rand(0, Math.PI * 2),
          twSpeed: rand(1, 3),
        }
        if (i % 7 === 0) sparks.push(p)
        else dust.push(p)
      }
    }

    const paint = (list: Particle[], colour: string, t: number, p: number, dot: number) => {
      ctx.fillStyle = colour
      const inv = 1 - p
      for (let i = 0; i < list.length; i++) {
        const s = list[i]
        // Drift dies away as the word resolves.
        const dx = s.hx + s.amp * Math.sin(t * s.driftSpeed + s.driftPhase) * inv
        const dy = s.hy + s.amp * Math.cos(t * s.driftSpeed * 0.85 + s.driftPhase) * inv
        const x = dx + (s.tx - dx) * p
        const y = dy + (s.ty - dy) * p
        const tw = 0.4 + 0.6 * (0.5 + 0.5 * Math.sin(t * s.twSpeed + s.twPhase))
        ctx.globalAlpha = tw + (1 - tw) * p
        ctx.fillRect(x, y, dot, dot)
      }
    }

    const render = (now: number, p: number) => {
      const dot = window.innerWidth < 768 ? 1.7 : 2
      const t = now * 0.001
      ctx.clearRect(0, 0, width, height)
      paint(dust, base, t, p, dot)
      paint(sparks, accent, t, p, dot)
      ctx.globalAlpha = 1
    }

    /** Drive the DOM copy from the same progress value. */
    const layout = (p: number) => {
      const size = bigSize + (smallSize - bigSize) * p
      h1.style.fontSize = `${size}px`
      // The block is centred by a -50% translate, and the lift rides on top of
      // it. Both have to live in this one declaration: writing only the lift
      // would replace the centring and drop the headline half its own height.
      copy.style.transform = `translateY(calc(-50% - ${p * height * 0.17}px))`
      // Supporting copy arrives only once the word is legible.
      reveal.style.opacity = String(clamp((p - 0.72) / 0.28, 0, 1))
    }

    /** Scroll progress through the pinned section. */
    const progressAt = () => {
      const rect = section.getBoundingClientRect()
      const runway = Math.max(rect.height - window.innerHeight, 1)
      const scrolled = clamp(-rect.top, 0, runway)
      // Resolve before the runway ends, so the formed state gets a beat.
      return easeOutCubic(clamp(scrolled / (runway * 0.7), 0, 1))
    }

    build()

    if (reduced) {
      // No motion: present the resolved state.
      layout(1)
      render(0, 1)
    } else {
      layout(0)
      const loop = (now: number) => {
        if (visible) {
          const p = progressAt()
          render(now, p)
          layout(p)
        }
        raf = requestAnimationFrame(loop)
      }
      raf = requestAnimationFrame(loop)
    }

    let resizeTimer = 0
    const onResize = () => {
      window.clearTimeout(resizeTimer)
      resizeTimer = window.setTimeout(() => {
        if (disposed) return
        build()
        const p = reduced ? 1 : progressAt()
        layout(p)
        if (reduced) render(0, 1)
      }, 150)
    }
    window.addEventListener('resize', onResize)
    window.addEventListener('orientationchange', onResize)

    // A late-loading face would otherwise leave a stale raster and a
    // mis-measured headline.
    document.fonts?.ready.then(() => {
      if (disposed) return
      build()
      const p = reduced ? 1 : progressAt()
      layout(p)
      if (reduced) render(0, 1)
    })

    const io = new IntersectionObserver(([entry]) => {
      visible = entry?.isIntersecting ?? true
    })
    io.observe(stage)

    return () => {
      disposed = true
      cancelAnimationFrame(raf)
      window.clearTimeout(resizeTimer)
      window.removeEventListener('resize', onResize)
      window.removeEventListener('orientationchange', onResize)
      io.disconnect()
    }
  }, [wordmark, headline, accent, base])

  return (
    // The tall section is the scroll runway; the stage inside it pins.
    <section ref={sectionRef} className="relative h-[240vh]">
      <div ref={stageRef} className="sticky top-0 h-screen overflow-hidden bg-bg will-change-transform">
        <div className="absolute inset-0 bg-grid-pattern opacity-25 pointer-events-none" />
        <canvas ref={canvasRef} className="absolute inset-0 h-full w-full" aria-hidden="true" />

        {/* Headline block, lifted as the wordmark resolves. */}
        <div
          ref={copyRef}
          className="absolute inset-x-0 top-1/2 -translate-y-1/2 px-7 will-change-transform"
        >
          {eyebrow && (
            <p className="mb-5 text-center font-mono text-[11px] uppercase tracking-[0.3em] text-sec sm:text-xs">
              {eyebrow}
            </p>
          )}
          <h1
            ref={headlineRef}
            className="mx-auto max-w-site text-center font-display font-[700] leading-[0.95] tracking-[-0.03em] text-text"
            style={{ fontSize: '9vw' }}
          >
            {headline}
          </h1>
        </div>

        {/* Revealed once the word is legible. */}
        <div
          ref={revealRef}
          className="absolute inset-x-0 bottom-[12vh] flex flex-col items-center gap-7 px-7 opacity-0"
          style={{ transition: 'opacity 120ms linear' }}
        >
          {subline && (
            <p className="max-w-narrow text-center font-mono text-sm leading-relaxed text-sec">
              {subline}
            </p>
          )}
          {actions && <div className="flex flex-col gap-3 sm:flex-row">{actions}</div>}
        </div>

        <div className="pointer-events-none absolute inset-x-0 bottom-0 h-28 bg-gradient-to-b from-transparent to-bg" />
      </div>
    </section>
  )
}
