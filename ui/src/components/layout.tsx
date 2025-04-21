import { Link, Outlet } from 'react-router-dom';
import { cn } from '../lib/utils';

export function Layout() {
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
              className={cn(
                'flex items-center px-4 py-2 text-sm font-medium rounded-md',
                'text-muted-foreground hover:text-foreground hover:bg-accent'
              )}
            >
              Dashboard
            </Link>
            <Link
              to="/events"
              className={cn(
                'flex items-center px-4 py-2 text-sm font-medium rounded-md',
                'text-muted-foreground hover:text-foreground hover:bg-accent'
              )}
            >
              Events
            </Link>
            <Link
              to="/rules"
              className={cn(
                'flex items-center px-4 py-2 text-sm font-medium rounded-md',
                'text-muted-foreground hover:text-foreground hover:bg-accent'
              )}
            >
              Routing Rules
            </Link>
            <Link
              to="/validations"
              className={cn(
                'flex items-center px-4 py-2 text-sm font-medium rounded-md',
                'text-muted-foreground hover:text-foreground hover:bg-accent'
              )}
            >
              Topic Validations
            </Link>
          </nav>
        </aside>

        {/* Main content */}
        <main className="flex-1 p-8">
          <Outlet />
        </main>
      </div>
    </div>
  );
} 