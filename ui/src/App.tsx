import { Routes, Route, Link } from 'react-router-dom'
import './App.css'
import DashboardPage from './pages/dashboard'
import RoutingRulesPage from './pages/routing-rules'

function App() {
  return (
    <div className="min-h-screen bg-background">
      <div className="flex">
        {/* Sidebar */}
        <aside className="w-64 min-h-screen bg-card border-r">
          <div className="p-6">
            <h1 className="text-2xl font-bold">Event Gateway</h1>
          </div>
          <nav className="space-y-1 px-4">
            <Link
              to="/"
              className="flex items-center px-4 py-2 text-sm font-medium rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              Dashboard
            </Link>
            <Link
              to="/events"
              className="flex items-center px-4 py-2 text-sm font-medium rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              Events
            </Link>
            <Link
              to="/rules"
              className="flex items-center px-4 py-2 text-sm font-medium rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              Routing Rules
            </Link>
            <Link
              to="/validations"
              className="flex items-center px-4 py-2 text-sm font-medium rounded-md text-muted-foreground hover:text-foreground hover:bg-accent"
            >
              Topic Validations
            </Link>
          </nav>
        </aside>

        {/* Main content */}
        <main className="flex-1 p-8">
          <Routes>
            <Route path="/" element={<DashboardPage />} />
            <Route path="/events" element={<div>Events Page (Coming Soon)</div>} />
            <Route path="/rules" element={<RoutingRulesPage />} />
            <Route path="/validations" element={<div>Topic Validations Page (Coming Soon)</div>} />
          </Routes>
        </main>
      </div>
    </div>
  )
}

export default App
