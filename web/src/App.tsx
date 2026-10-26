import { useState, useEffect } from 'react'
import { Bot, Package, BookOpen, ShieldCheck, LogOut, Settings2, Library } from 'lucide-react'
import { cn } from '@/lib/utils'
import { BotsPage } from '@/pages/BotsPage'
import { ItemsPage } from '@/pages/ItemsPage'
import { DocsPage } from '@/pages/DocsPage'
import { OptionsPage } from '@/pages/OptionsPage'
import { ScriptsPage } from '@/pages/ScriptsPage'
import { ProxyPage } from '@/pages/ProxyPage'
import { SetupPage } from '@/pages/SetupPage'
import { LoginPage } from '@/pages/LoginPage'
import { authStore } from '@/lib/auth'
import { api } from '@/lib/api'
import { APP_VERSION } from '@/lib/version'

type Page = 'bots' | 'items' | 'proxy' | 'docs' | 'options' | 'scripts'
type AuthState = 'loading' | 'setup' | 'login' | 'authenticated'

export default function App() {
  const [authState, setAuthState] = useState<AuthState>('loading')
  const [page, setPage] = useState<Page>('bots')

  // On mount: determine auth state
  useEffect(() => {
    api.authStatus().then(({ registered }) => {
      if (!registered) {
        setAuthState('setup')
      } else if (authStore.isLoggedIn()) {
        setAuthState('authenticated')
      } else {
        setAuthState('login')
      }
    }).catch(() => {
      // If we can't reach the server, still show login
      setAuthState('login')
    })

    // Listen for 401s from the API client
    const onUnauthorized = () => {
      setAuthState('login')
    }
    window.addEventListener('mori:unauthorized', onUnauthorized)
    return () => window.removeEventListener('mori:unauthorized', onUnauthorized)
  }, [])

  async function handleLogout() {
    await api.authLogout().catch(() => {})
    authStore.clearToken()
    setAuthState('login')
  }

  if (authState === 'loading') {
    return (
      <div className="h-full flex items-center justify-center text-muted-foreground text-xs">
        Loading…
      </div>
    )
  }

  if (authState === 'setup') {
    return <SetupPage onDone={() => setAuthState('authenticated')} />
  }

  if (authState === 'login') {
    return <LoginPage onDone={() => setAuthState('authenticated')} />
  }

  return (
    <div className="h-full flex flex-col antialiased text-sm">
      <header className="shrink-0 h-10 flex items-center justify-between px-4 bg-card border-b border-border">
        <span className="font-bold tracking-wide text-foreground text-xs flex items-center gap-2">
          Mori<span className="text-primary">.</span>
          <span className="rounded border border-border px-1.5 py-0.5 text-[10px] font-medium text-muted-foreground">
            v{APP_VERSION}
          </span>
        </span>
        <nav className="flex gap-0.5">
          <NavBtn active={page === 'bots'} onClick={() => setPage('bots')} icon={<Bot className="w-3 h-3" />} label="Bots" />
          <NavBtn active={page === 'items'} onClick={() => setPage('items')} icon={<Package className="w-3 h-3" />} label="Items" />
          <NavBtn active={page === 'proxy'} onClick={() => setPage('proxy')} icon={<ShieldCheck className="w-3 h-3" />} label="Proxy" />
          <NavBtn active={page === 'options'} onClick={() => setPage('options')} icon={<Settings2 className="w-3 h-3" />} label="Options" />
          <NavBtn active={page === 'scripts'} onClick={() => setPage('scripts')} icon={<Library className="w-3 h-3" />} label="Scripts" />
          <NavBtn active={page === 'docs'} onClick={() => setPage('docs')} icon={<BookOpen className="w-3 h-3" />} label="Docs" />
          <button
            onClick={handleLogout}
            title="Logout"
            className="px-3 py-1 text-xs rounded font-medium flex items-center gap-1.5 transition-colors text-muted-foreground hover:text-foreground"
          >
            <LogOut className="w-3 h-3" />
          </button>
        </nav>
      </header>
      <div className="flex-1 overflow-hidden">
        {page === 'bots' && <BotsPage />}
        {page === 'items' && <ItemsPage />}
        {page === 'proxy' && <ProxyPage />}
        {page === 'options' && <OptionsPage />}
        {page === 'scripts' && <ScriptsPage />}
        {page === 'docs' && <DocsPage />}
      </div>
    </div>
  )
}

function NavBtn({
  active,
  onClick,
  icon,
  label,
}: {
  active: boolean
  onClick: () => void
  icon: React.ReactNode
  label: string
}) {
  return (
    <button
      onClick={onClick}
      className={cn(
        'px-3 py-1 text-xs rounded font-medium flex items-center gap-1.5 transition-colors',
        active ? 'text-primary bg-muted' : 'text-muted-foreground hover:text-foreground',
      )}
    >
      {icon}
      {label}
    </button>
  )
}
