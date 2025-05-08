import { useEffect, useState } from 'react';
import { Button } from '@/components/ui/button';
import {
  Table,
  TableBody,
  TableCell,
  TableHead,
  TableHeader,
  TableRow,
} from '@/components/ui/table';
import { ChevronLeft, ChevronRight, ChevronDown, ChevronRight as ChevronRightIcon, Search } from 'lucide-react';
import axios from 'axios';
import { Input } from '@/components/ui/input';

interface Event {
  id: string;
  eventType: string;
  eventVersion: string;
  metadata: Record<string, any>;
  data: Record<string, any>;
  timestamp?: string;
  origin?: string;
}

interface EventsResponse {
  events: Event[];
  total: number;
}

const PAGE_SIZE = 10;

export default function EventsPage() {
  const [events, setEvents] = useState<Event[]>([]);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);
  const [currentPage, setCurrentPage] = useState(1);
  const [totalEvents, setTotalEvents] = useState(0);
  const [searchQuery, setSearchQuery] = useState('');
  const [expandedRows, setExpandedRows] = useState<Set<string>>(new Set());

  useEffect(() => {
    loadEvents();
  }, [currentPage]);

  async function loadEvents() {
    try {
      setLoading(true);
      const offset = (currentPage - 1) * PAGE_SIZE;
      console.log('Fetching events with params:', { limit: PAGE_SIZE, offset });
      const response = await axios.get<EventsResponse>(`/api/v1/events/samples?limit=${PAGE_SIZE}&offset=${offset}`);
      console.log('Received response:', response.data);
      setEvents(response.data.events);
      setTotalEvents(response.data.total);
      setError(null);
    } catch (err) {
      console.error('Error loading events:', err);
      setError('Failed to load events');
    } finally {
      setLoading(false);
    }
  }

  const toggleRow = (id: string) => {
    setExpandedRows(prev => {
      const newSet = new Set(prev);
      if (newSet.has(id)) {
        newSet.delete(id);
      } else {
        newSet.add(id);
      }
      return newSet;
    });
  };

  const filteredEvents = events.filter(event => {
    if (!searchQuery) return true;
    const searchLower = searchQuery.toLowerCase();
    
    // Search through all relevant fields
    return (
      event.id.toLowerCase().includes(searchLower) ||
      event.eventType.toLowerCase().includes(searchLower) ||
      event.eventVersion.toLowerCase().includes(searchLower) ||
      (event.timestamp?.toLowerCase().includes(searchLower) ?? false) ||
      (event.origin?.toLowerCase().includes(searchLower) ?? false) ||
      JSON.stringify(event.data).toLowerCase().includes(searchLower) ||
      JSON.stringify(event.metadata).toLowerCase().includes(searchLower)
    );
  });

  const totalPages = Math.ceil(totalEvents / PAGE_SIZE);

  return (
    <div className="space-y-4">
      <div className="flex flex-col gap-4">
        <h1 className="text-2xl font-bold">Events</h1>
        <div className="relative w-full max-w-md mx-auto">
          <Search className="absolute left-2 top-2.5 h-4 w-4 text-muted-foreground" />
          <Input
            placeholder="Search events..."
            value={searchQuery}
            onChange={(e) => setSearchQuery(e.target.value)}
            className="pl-8"
          />
        </div>
      </div>

      {error && (
        <div className="bg-destructive/15 text-destructive px-4 py-2 rounded-md">
          {error}
        </div>
      )}

      <div className="border rounded-lg overflow-x-auto">
        <Table>
          <TableHeader>
            <TableRow>
              <TableHead className="w-[50px]"></TableHead>
              <TableHead className="w-[200px]">ID</TableHead>
              <TableHead className="w-[150px]">Type</TableHead>
              <TableHead className="w-[100px]">Version</TableHead>
              <TableHead className="w-[200px]">Timestamp</TableHead>
              <TableHead className="w-[100px]">Origin</TableHead>
              <TableHead>Data</TableHead>
            </TableRow>
          </TableHeader>
          <TableBody>
            {loading ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center py-4">
                  Loading...
                </TableCell>
              </TableRow>
            ) : filteredEvents.length === 0 ? (
              <TableRow>
                <TableCell colSpan={7} className="text-center py-4">
                  No events found
                </TableCell>
              </TableRow>
            ) : (
              filteredEvents.map((event) => (
                <TableRow key={`${event.id}-${event.timestamp}`}>
                  <TableCell>
                    <Button
                      variant="ghost"
                      size="icon"
                      onClick={() => toggleRow(event.id)}
                      className="h-8 w-8"
                    >
                      {expandedRows.has(event.id) ? (
                        <ChevronDown className="h-4 w-4" />
                      ) : (
                        <ChevronRightIcon className="h-4 w-4" />
                      )}
                    </Button>
                  </TableCell>
                  <TableCell className="font-mono whitespace-nowrap">
                    {searchQuery && event.id.toLowerCase().includes(searchQuery.toLowerCase()) ? (
                      <span dangerouslySetInnerHTML={{
                        __html: event.id.replace(
                          new RegExp(searchQuery, 'gi'),
                          match => `<span class="bg-green-200 dark:bg-green-800 px-1 rounded">${match}</span>`
                        )
                      }} />
                    ) : event.id}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    {searchQuery && event.eventType.toLowerCase().includes(searchQuery.toLowerCase()) ? (
                      <span dangerouslySetInnerHTML={{
                        __html: event.eventType.replace(
                          new RegExp(searchQuery, 'gi'),
                          match => `<span class="bg-green-200 dark:bg-green-800 px-1 rounded">${match}</span>`
                        )
                      }} />
                    ) : event.eventType}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    {searchQuery && event.eventVersion.toLowerCase().includes(searchQuery.toLowerCase()) ? (
                      <span dangerouslySetInnerHTML={{
                        __html: event.eventVersion.replace(
                          new RegExp(searchQuery, 'gi'),
                          match => `<span class="bg-green-200 dark:bg-green-800 px-1 rounded">${match}</span>`
                        )
                      }} />
                    ) : event.eventVersion}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    {searchQuery && event.timestamp?.toLowerCase().includes(searchQuery.toLowerCase()) ? (
                      <span dangerouslySetInnerHTML={{
                        __html: event.timestamp.replace(
                          new RegExp(searchQuery, 'gi'),
                          match => `<span class="bg-green-200 dark:bg-green-800 px-1 rounded">${match}</span>`
                        )
                      }} />
                    ) : event.timestamp}
                  </TableCell>
                  <TableCell className="whitespace-nowrap">
                    {searchQuery && event.origin?.toLowerCase().includes(searchQuery.toLowerCase()) ? (
                      <span dangerouslySetInnerHTML={{
                        __html: (event.origin || '-').replace(
                          new RegExp(searchQuery, 'gi'),
                          match => `<span class="bg-green-200 dark:bg-green-800 px-1 rounded">${match}</span>`
                        )
                      }} />
                    ) : (event.origin || '-')}
                  </TableCell>
                  <TableCell>
                    {expandedRows.has(event.id) && (
                      <pre className="text-xs overflow-auto max-w-md">
                        {searchQuery ? (
                          <span dangerouslySetInnerHTML={{
                            __html: JSON.stringify(event.data, null, 2)
                              .replace(
                                new RegExp(searchQuery, 'gi'),
                                match => `<span class="bg-green-200 dark:bg-green-800 px-1 rounded">${match}</span>`
                              )
                          }} />
                        ) : (
                          JSON.stringify(event.data, null, 2)
                        )}
                      </pre>
                    )}
                  </TableCell>
                </TableRow>
              ))
            )}
          </TableBody>
        </Table>
      </div>

      <div className="flex justify-between items-center">
        <div className="text-sm text-muted-foreground">
          Showing {filteredEvents.length} of {totalEvents} events
        </div>
        <div className="flex gap-2">
          <Button
            variant="outline"
            size="sm"
            onClick={() => setCurrentPage(p => Math.max(1, p - 1))}
            disabled={currentPage === 1}
          >
            <ChevronLeft className="h-4 w-4" />
            Previous
          </Button>
          <Button
            variant="outline"
            size="sm"
            onClick={() => setCurrentPage(p => Math.min(totalPages, p + 1))}
            disabled={currentPage === totalPages}
          >
            Next
            <ChevronRight className="h-4 w-4" />
          </Button>
        </div>
      </div>
    </div>
  );
} 