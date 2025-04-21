# Event Gateway UI


## Features

- **Dashboard**: Overview of events, rules, and system metrics
- **Routing Rules**: Create, edit, and manage event routing rules
- **Topic Validations**: Define and manage validation schemas for topics
- **Modern UI**: Built with React, TypeScript, and Tailwind CSS

## Tech Stack

- **React**: UI library
- **TypeScript**: Type safety
- **React Router**: Client-side routing
- **Tailwind CSS**: Utility-first CSS framework
- **Radix UI**: Accessible UI components
- **Lucide Icons**: Beautiful icons

## Project Structure

```
ui/
├── public/              # Static assets
├── src/
│   ├── app/             # App configuration
│   ├── components/      # Reusable UI components
│   │   ├── ui/          # Base UI components
│   │   └── ...          # Feature-specific components
│   ├── hooks/           # Custom React hooks
│   ├── lib/             # Utility functions
│   ├── pages/           # Page components
│   ├── services/        # API services
│   ├── types/           # TypeScript type definitions
│   ├── App.tsx          # Main App component
│   ├── main.tsx         # Application entry point
│   └── index.css        # Global styles
├── package.json         # Dependencies and scripts
└── tsconfig.json        # TypeScript configuration
```

## Getting Started

### Prerequisites

- Node.js (v16 or higher)
- npm or yarn

### Installation

1. Clone the repository
2. Navigate to the UI directory:
   ```
   cd ui
   ```
3. Install dependencies:
   ```
   npm install
   # or
   yarn install
   ```

### Development

Start the development server:

```
npm run dev
# or
yarn dev
```

The application will be available at [http://localhost:5173](http://localhost:5173).

### Building for Production

Build the application for production:

```
npm run build
# or
yarn build
```

The built files will be in the `dist` directory.

## Available Scripts

- `dev`: Start development server
- `build`: Build for production
- `preview`: Preview production build
- `lint`: Run ESLint
- `type-check`: Run TypeScript type checking

## UI Components

The application uses a combination of custom components and Radix UI primitives:

- **Button**: Various button styles and variants
- **Card**: Container for content with header, content, and footer sections
- **Dialog**: Modal dialogs for forms and confirmations
- **Input**: Text input fields
- **Label**: Form labels
- **Select**: Dropdown selection
- **Table**: Data tables with sorting and pagination
- **Toast**: Notification system

## State Management

The application uses React's built-in state management with hooks:

- `useState`: Local component state
- `useEffect`: Side effects and data fetching
- `useToast`: Toast notifications

## Routing

The application uses React Router for client-side routing:

- `/`: Dashboard
- `/events`: Events page (coming soon)
- `/rules`: Routing rules management
- `/validations`: Topic validations management

## Contributing

1. Fork the repository
2. Create a feature branch
3. Commit your changes
4. Push to the branch
5. Create a pull request

## License

This project is licensed under the MIT License.
