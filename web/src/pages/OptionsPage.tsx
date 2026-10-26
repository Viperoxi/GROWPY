import { useEffect, useState, type ReactNode } from 'react'
import { Settings2 } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { Switch } from '@/components/ui/switch'
import { api, type GlobalOptions } from '@/lib/api'

const EMPTY_OPTIONS: GlobalOptions = {
  place_ms: 180,
  walk_ms: 80,
  auto_collect: true,
  collect_range: 5,
  collect_path_check: false,
  ignore_gems: false,
  ignore_essences: false,
  punch_reach: 2,
  farm_holes: 2,
  pickup_range: 3,
}

export function OptionsPage() {
  const [opts, setOpts] = useState<GlobalOptions>(EMPTY_OPTIONS)
  const [status, setStatus] = useState('')

  useEffect(() => {
    api.getOptions().then(setOpts).catch(() => {})
  }, [])

  function patch<K extends keyof GlobalOptions>(key: K, value: GlobalOptions[K]) {
    setOpts((prev) => ({ ...prev, [key]: value }))
  }

  async function saveOptions() {
    try {
      const saved = await api.saveOptions(opts)
      setOpts(saved)
      setStatus('Saved')
      setTimeout(() => setStatus(''), 2000)
    } catch {
      setStatus('Error')
    }
  }

  return (
    <div className="h-full overflow-hidden bg-background">
      <div className="border-b border-border bg-card/80 backdrop-blur">
        <div className="mx-auto flex max-w-7xl flex-col gap-3 px-6 py-5">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline" className="gap-1.5">
              <Settings2 className="size-3" />
              Options
            </Badge>
            <Badge variant="secondary">Global</Badge>
          </div>
          <h1 className="text-2xl font-semibold tracking-tight text-foreground">
            Bot defaults
          </h1>
          <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
            These values apply to every bot when you press Run. A Lua script can still override them
            with <code className="text-foreground">bot.place_delay</code> or local variables.
          </p>
        </div>
      </div>

      <ScrollArea className="h-[calc(100%-9rem)]">
        <div className="mx-auto max-w-xl px-6 py-6">
          <section className="rounded-xl border border-border bg-card p-4 flex flex-col gap-4">
            <h2 className="text-sm font-semibold">Bot defaults</h2>
            <Field label="Place / punch (ms)">
              <Input type="number" min={0} value={opts.place_ms} onChange={(e) => patch('place_ms', Number(e.target.value) || 0)} className="h-7 text-xs" />
            </Field>
            <Field label="Walk / pathfind (ms)">
              <Input type="number" min={0} value={opts.walk_ms} onChange={(e) => patch('walk_ms', Number(e.target.value) || 0)} className="h-7 text-xs" />
            </Field>
            <Toggle label="Auto collect" checked={opts.auto_collect} onChange={(v) => patch('auto_collect', v)} />
            <Field label="Collect range (tiles)">
              <Input type="number" min={1} max={5} value={opts.collect_range} onChange={(e) => patch('collect_range', Number(e.target.value) || 1)} className="h-7 text-xs" />
            </Field>
            <Toggle label="Collect path check" checked={opts.collect_path_check} onChange={(v) => patch('collect_path_check', v)} />
            <Toggle label="Ignore gems" checked={opts.ignore_gems} onChange={(v) => patch('ignore_gems', v)} />
            <Toggle label="Ignore essences" checked={opts.ignore_essences} onChange={(v) => patch('ignore_essences', v)} />
            <h2 className="text-sm font-semibold pt-2">Script config</h2>
            <p className="text-[11px] text-muted-foreground -mt-2">
              Exposed in Lua as <code className="text-foreground">config</code>.
            </p>
            <Field label="Punch reach (tiles)">
              <Input type="number" min={1} max={4} value={opts.punch_reach} onChange={(e) => patch('punch_reach', Number(e.target.value) || 1)} className="h-7 text-xs" />
            </Field>
            <Field label="Farm holes per side">
              <Input type="number" min={1} max={8} value={opts.farm_holes} onChange={(e) => patch('farm_holes', Number(e.target.value) || 1)} className="h-7 text-xs" />
            </Field>
            <Field label="Pickup range (tiles)">
              <Input type="number" min={1} max={8} value={opts.pickup_range} onChange={(e) => patch('pickup_range', Number(e.target.value) || 1)} className="h-7 text-xs" />
            </Field>
            <div className="flex items-center gap-2 pt-1">
              <Button size="sm" className="text-xs" onClick={saveOptions}>Save defaults</Button>
              {status && <span className="text-xs text-muted-foreground">{status}</span>}
            </div>
          </section>
        </div>
      </ScrollArea>
    </div>
  )
}

function Field({ label, children }: { label: string; children: ReactNode }) {
  return (
    <label className="flex flex-col gap-1">
      <span className="text-xs text-muted-foreground">{label}</span>
      {children}
    </label>
  )
}

function Toggle({
  label,
  checked,
  onChange,
}: {
  label: string
  checked: boolean
  onChange: (value: boolean) => void
}) {
  return (
    <label className="flex items-center justify-between gap-3">
      <span className="text-xs text-muted-foreground">{label}</span>
      <Switch checked={checked} onCheckedChange={onChange} size="sm" />
    </label>
  )
}
