import { useEffect, useMemo, useState } from 'react'
import { Library, Search, Trash2 } from 'lucide-react'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { ScrollArea } from '@/components/ui/scroll-area'
import { api, type SavedScript } from '@/lib/api'
import { LuaEditor } from '@/components/lua-editor'

export function ScriptsPage() {
  const [library, setLibrary] = useState<SavedScript[]>([])
  const [query, setQuery] = useState('')
  const [selectedId, setSelectedId] = useState('')
  const [name, setName] = useState('')
  const [content, setContent] = useState('')
  const [libStatus, setLibStatus] = useState('')

  useEffect(() => {
    api.listScripts().then(setLibrary).catch(() => {})
  }, [])

  const filtered = useMemo(() => {
    const term = query.trim().toLowerCase()
    return library
      .filter((s) => !term || s.name.toLowerCase().includes(term))
      .sort((a, b) => a.name.localeCompare(b.name))
  }, [library, query])

  const selected = library.find((s) => s.id === selectedId)

  function pick(script: SavedScript) {
    setSelectedId(script.id)
    setName(script.name)
    setContent(script.content)
  }

  async function refreshLibrary(preferId?: string) {
    const list = await api.listScripts()
    setLibrary(list)
    if (preferId && list.some((s) => s.id === preferId)) {
      setSelectedId(preferId)
    }
  }

  async function saveScript() {
    const trimmed = name.trim()
    if (!trimmed) {
      setLibStatus('Name required')
      return
    }
    try {
      if (selectedId) {
        const updated = await api.updateScript(selectedId, { name: trimmed, content })
        await refreshLibrary(updated.id)
      } else {
        const created = await api.createScript({ name: trimmed, content })
        await refreshLibrary(created.id)
        setSelectedId(created.id)
      }
      setLibStatus('Saved')
      setTimeout(() => setLibStatus(''), 2000)
    } catch {
      setLibStatus('Error')
    }
  }

  async function saveAsNew() {
    const trimmed = name.trim()
    if (!trimmed) {
      setLibStatus('Name required')
      return
    }
    try {
      const created = await api.createScript({ name: trimmed, content })
      await refreshLibrary(created.id)
      setSelectedId(created.id)
      setLibStatus('Saved')
      setTimeout(() => setLibStatus(''), 2000)
    } catch {
      setLibStatus('Error')
    }
  }

  async function removeScript() {
    if (!selectedId) return
    try {
      await api.deleteScript(selectedId)
      setSelectedId('')
      setName('')
      setContent('')
      await refreshLibrary()
      setLibStatus('Deleted')
      setTimeout(() => setLibStatus(''), 2000)
    } catch {
      setLibStatus('Error')
    }
  }

  return (
    <div className="h-full overflow-hidden bg-background">
      <div className="border-b border-border bg-card/80 backdrop-blur">
        <div className="mx-auto flex max-w-7xl flex-col gap-3 px-6 py-5">
          <div className="flex flex-wrap items-center gap-2">
            <Badge variant="outline" className="gap-1.5">
              <Library className="size-3" />
              Scripts
            </Badge>
            <Badge variant="secondary">Library</Badge>
          </div>
          <h1 className="text-2xl font-semibold tracking-tight text-foreground">
            Script library
          </h1>
          <p className="max-w-3xl text-sm leading-6 text-muted-foreground">
            Saved scripts are shared across all bots. Call them from Lua with{' '}
            <code className="text-foreground">runScript("name")</code>.
          </p>
        </div>
      </div>

      <ScrollArea className="h-[calc(100%-9rem)]">
        <div className="mx-auto max-w-7xl px-6 py-6">
          <section className="rounded-xl border border-border bg-card p-4 flex flex-col gap-3 min-h-[32rem]">
            <div className="flex items-center gap-2">
              <Library className="size-4 text-muted-foreground" />
              <h2 className="text-sm font-semibold">Saved scripts</h2>
              <span className="text-xs text-muted-foreground">{library.length}</span>
            </div>
            <div className="relative">
              <Search className="pointer-events-none absolute left-3 top-1/2 size-3.5 -translate-y-1/2 text-muted-foreground" />
              <Input
                value={query}
                onChange={(e) => setQuery(e.target.value)}
                placeholder="Search saved scripts..."
                className="h-8 pl-9 text-xs"
              />
            </div>
            <div className="grid gap-3 md:grid-cols-[14rem_1fr] flex-1 min-h-0">
              <ScrollArea className="h-72 rounded-lg border border-border">
                <div className="p-1">
                  {filtered.length === 0 && (
                    <p className="px-2 py-6 text-xs text-muted-foreground">No saved scripts yet.</p>
                  )}
                  {filtered.map((script) => (
                    <button
                      key={script.id}
                      onClick={() => pick(script)}
                      className={`w-full rounded-md px-2 py-1.5 text-left text-xs truncate ${
                        selectedId === script.id
                          ? 'bg-muted text-foreground'
                          : 'text-muted-foreground hover:text-foreground hover:bg-muted/60'
                      }`}
                    >
                      {script.name}
                    </button>
                  ))}
                </div>
              </ScrollArea>
              <div className="flex flex-col gap-2 min-h-0">
                <Input
                  value={name}
                  onChange={(e) => setName(e.target.value)}
                  placeholder="Script name"
                  className="h-8 text-xs"
                />
                <LuaEditor
                  value={content}
                  onChange={setContent}
                  className="min-h-48"
                  placeholder={'print("hello")'}
                />
                <div className="flex flex-wrap items-center gap-2">
                  <Button size="sm" className="text-xs" onClick={saveScript}>
                    {selected ? 'Update' : 'Save'}
                  </Button>
                  <Button size="sm" variant="outline" className="text-xs" onClick={saveAsNew}>
                    Save as new
                  </Button>
                  <Button
                    size="sm"
                    variant="destructive"
                    className="text-xs"
                    onClick={removeScript}
                    disabled={!selectedId}
                  >
                    <Trash2 className="size-3" />
                    Delete
                  </Button>
                  {libStatus && <span className="text-xs text-muted-foreground">{libStatus}</span>}
                </div>
              </div>
            </div>
          </section>
        </div>
      </ScrollArea>
    </div>
  )
}
