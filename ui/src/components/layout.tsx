import { Link, useLocation } from 'react-router-dom';
import { cn } from '../lib/utils';

interface LayoutProps {
  children: React.ReactNode;
}

export function Layout({ children }: LayoutProps) {
  const location = useLocation();

  const isActive = (path: string) => {
    return location.pathname === path;
  };

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
              to="/routing-rules"
              className={`flex items-center px-4 py-2 text-sm font-medium rounded-md ${
                isActive('/routing-rules')
                  ? 'bg-accent text-foreground'
                  : 'text-muted-foreground hover:text-foreground hover:bg-accent'
              }`}
            >
              Routing Rules
            </Link>
            <Link
              to="/topic-validations"
              className={`flex items-center px-4 py-2 text-sm font-medium rounded-md ${
                isActive('/topic-validations')
                  ? 'bg-accent text-foreground'
                  : 'text-muted-foreground hover:text-foreground hover:bg-accent'
              }`}
            >
              Topic Validations
            </Link>
            <Link
              to="/playground"
              className={`flex items-center px-4 py-2 text-sm font-medium rounded-md ${
                isActive('/playground')
                  ? 'bg-accent text-foreground'
                  : 'text-muted-foreground hover:text-foreground hover:bg-accent'
              }`}
            >
              Playground
            </Link>
          </nav>
        </aside>

        {/* Main content */}
        <main className="flex-1 p-8">
          {children}
        </main>
      </div>
    </div>
  );
} 